use ffmpeg::format::Pixel;
use ffmpeg_next as ffmpeg;
use libc::EAGAIN;
use shrimply_math_core::Fraction;
use shrimply_project_document::project::{fraction_denominator, fraction_numerator};
use std::path::Path;

#[derive(Clone, Copy)]
pub enum HevcBackend {
    Nvenc,
    X265,
}

impl HevcBackend {
    fn name(self) -> &'static str {
        match self {
            Self::Nvenc => "hevc_nvenc",
            Self::X265 => "libx265",
        }
    }
    fn pixel(self) -> Pixel {
        match self {
            Self::Nvenc => Pixel::NV12,
            Self::X265 => Pixel::YUV420P,
        }
    }
}

#[derive(Clone, Copy)]
pub enum RgbaAlpha {
    Premultiplied,
    Straight,
}

/// Two HEVC streams: straight color followed by a lossless alpha luma plane.
pub struct VideoCacheEncoder {
    output: ffmpeg::format::context::Output,
    color: ffmpeg::codec::encoder::video::Encoder,
    alpha: ffmpeg::codec::encoder::video::Encoder,
    color_stream: usize,
    alpha_stream: usize,
    color_time_base: ffmpeg::Rational,
    alpha_time_base: ffmpeg::Rational,
    scaler: ffmpeg::software::scaling::context::Context,
    width: u32,
    height: u32,
    pixel: Pixel,
}

impl VideoCacheEncoder {
    pub fn new(
        path: &Path,
        width: u32,
        height: u32,
        fps: Fraction,
        color_qp: u32,
        backend: HevcBackend,
    ) -> Result<Self, String> {
        ffmpeg::init().map_err(|error| format!("could not initialize FFmpeg: {error}"))?;
        let time_base = ffmpeg::Rational(
            i32::try_from(fraction_denominator(fps))
                .map_err(|_| "cache FPS denominator is too large".to_string())?,
            i32::try_from(fraction_numerator(fps))
                .map_err(|_| "cache FPS numerator is too large".to_string())?,
        );
        let frame_rate = ffmpeg::Rational(time_base.1, time_base.0);
        let mut output = ffmpeg::format::output(path)
            .map_err(|error| format!("could not create visual cache media: {error}"))?;
        let global_header = output
            .format()
            .flags()
            .contains(ffmpeg::format::Flags::GLOBAL_HEADER);
        let format = EncoderFormat {
            width,
            height,
            time_base,
            frame_rate,
            global_header,
            backend,
        };
        let color = open_encoder(&format, color_qp, false)?;
        let alpha = open_encoder(&format, 0, true)?;
        let color_stream = {
            let mut stream = output
                .add_stream_with(color.as_ref())
                .map_err(|error| format!("could not add visual cache color stream: {error}"))?;
            stream.set_time_base(time_base);
            stream.index()
        };
        let alpha_stream = {
            let mut stream = output
                .add_stream_with(alpha.as_ref())
                .map_err(|error| format!("could not add visual cache alpha stream: {error}"))?;
            stream.set_time_base(time_base);
            stream.index()
        };
        output
            .write_header()
            .map_err(|error| format!("could not write visual cache header: {error}"))?;
        let color_time_base = output
            .stream(color_stream)
            .expect("visual color stream disappeared")
            .time_base();
        let alpha_time_base = output
            .stream(alpha_stream)
            .expect("visual alpha stream disappeared")
            .time_base();
        let scaler = ffmpeg::software::scaling::context::Context::get(
            Pixel::RGBA,
            width,
            height,
            backend.pixel(),
            width,
            height,
            ffmpeg::software::scaling::flag::Flags::BILINEAR,
        )
        .map_err(|error| format!("could not create visual cache color converter: {error}"))?;
        Ok(Self {
            output,
            color,
            alpha,
            color_stream,
            alpha_stream,
            color_time_base,
            alpha_time_base,
            scaler,
            pixel: backend.pixel(),
            width,
            height,
        })
    }

    pub fn write(
        &mut self,
        source: &ffmpeg::frame::Video,
        frame_index: u64,
        alpha_mode: RgbaAlpha,
    ) -> Result<(), String> {
        if source.format() != Pixel::RGBA
            || source.width() > self.width
            || source.height() > self.height
        {
            return Err("visual cache requires an RGBA frame within the coded dimensions".into());
        }
        let mut rgba = ffmpeg::frame::Video::new(Pixel::RGBA, self.width, self.height);
        rgba.data_mut(0).fill(0);
        let source_stride = source.stride(0);
        let destination_stride = rgba.stride(0);
        let source_width = source.width() as usize;
        for row in 0..source.height() as usize {
            let source_row = &source.data(0)[row * source_stride..][..source_width * 4];
            let destination_row =
                &mut rgba.data_mut(0)[row * destination_stride..][..source_width * 4];
            for (source, destination) in source_row
                .chunks_exact(4)
                .zip(destination_row.chunks_exact_mut(4))
            {
                let alpha = u16::from(source[3]);
                destination[3] = source[3];
                destination[..3].fill(0);
                if matches!(alpha_mode, RgbaAlpha::Straight) {
                    destination[..3].copy_from_slice(&source[..3]);
                } else if alpha != 0 {
                    for channel in 0..3 {
                        destination[channel] = (u16::from(source[channel]) * 255 + alpha / 2)
                            .checked_div(alpha)
                            .expect("nonzero alpha must divide")
                            .min(255) as u8;
                    }
                }
            }
        }
        let mut color = ffmpeg::frame::Video::new(self.pixel, self.width, self.height);
        self.scaler
            .run(&rgba, &mut color)
            .map_err(|error| format!("could not convert visual cache color frame: {error}"))?;
        let mut alpha = ffmpeg::frame::Video::new(self.pixel, self.width, self.height);
        alpha.set_color_range(ffmpeg::color::Range::JPEG);
        alpha.data_mut(0).fill(0);
        for plane in 1..alpha.planes() {
            alpha.data_mut(plane).fill(128);
        }
        let rgba_stride = rgba.stride(0);
        let alpha_stride = alpha.stride(0);
        for row in 0..self.height as usize {
            for column in 0..self.width as usize {
                alpha.data_mut(0)[row * alpha_stride + column] =
                    rgba.data(0)[row * rgba_stride + column * 4 + 3];
            }
        }
        let pts = i64::try_from(frame_index).map_err(|_| "cache is too long".to_string())?;
        color.set_pts(Some(pts));
        color.set_kind(ffmpeg::util::picture::Type::I);
        alpha.set_pts(Some(pts));
        alpha.set_kind(ffmpeg::util::picture::Type::I);
        self.color
            .send_frame(&color)
            .map_err(|error| format!("could not encode visual cache color: {error}"))?;
        receive_packets(
            &mut self.color,
            &mut self.output,
            self.color_stream,
            self.color_time_base,
        )?;
        self.alpha
            .send_frame(&alpha)
            .map_err(|error| format!("could not encode visual cache alpha: {error}"))?;
        receive_packets(
            &mut self.alpha,
            &mut self.output,
            self.alpha_stream,
            self.alpha_time_base,
        )
    }

    pub fn finish(mut self) -> Result<(), String> {
        self.color
            .send_eof()
            .map_err(|error| format!("could not finalize visual cache color: {error}"))?;
        receive_packets(
            &mut self.color,
            &mut self.output,
            self.color_stream,
            self.color_time_base,
        )?;
        self.alpha
            .send_eof()
            .map_err(|error| format!("could not finalize visual cache alpha: {error}"))?;
        receive_packets(
            &mut self.alpha,
            &mut self.output,
            self.alpha_stream,
            self.alpha_time_base,
        )?;
        self.output
            .write_trailer()
            .map_err(|error| format!("could not finalize visual cache media: {error}"))
    }
}

struct EncoderFormat {
    width: u32,
    height: u32,
    time_base: ffmpeg::Rational,
    frame_rate: ffmpeg::Rational,
    global_header: bool,
    backend: HevcBackend,
}

fn open_encoder(
    format: &EncoderFormat,
    qp: u32,
    alpha: bool,
) -> Result<ffmpeg::codec::encoder::video::Encoder, String> {
    let EncoderFormat {
        width,
        height,
        time_base,
        frame_rate,
        global_header,
        backend,
    } = *format;
    let name = backend.name();
    let codec = ffmpeg::codec::encoder::find_by_name(name)
        .ok_or_else(|| format!("FFmpeg encoder {name} was not found"))?;
    let mut encoder = ffmpeg::codec::Context::new_with_codec(codec)
        .encoder()
        .video()
        .map_err(|error| format!("could not configure {name}: {error}"))?;
    encoder.set_width(width);
    encoder.set_height(height);
    encoder.set_time_base(time_base);
    encoder.set_frame_rate(Some(frame_rate));
    encoder.set_gop(2);
    encoder.set_max_b_frames(0);
    encoder.set_format(backend.pixel());
    if alpha {
        // Alpha is stored directly as 0..255 luma, not studio-range video.
        encoder.set_color_range(ffmpeg::color::Range::JPEG);
    }
    if global_header {
        unsafe {
            (*encoder.as_mut_ptr()).flags |= ffmpeg::sys::AV_CODEC_FLAG_GLOBAL_HEADER as i32;
        }
    }
    let mut options = ffmpeg::Dictionary::new();
    options.set("forced-idr", "1");
    match backend {
        HevcBackend::Nvenc => {
            options.set("preset", if qp == 0 { "lossless" } else { "p4" });
            options.set("rc", "constqp");
            options.set("qp", &qp.to_string());
            options.set("bf", "0");
            options.set("rc-lookahead", "0");
            options.set("tune", if qp == 0 { "lossless" } else { "hq" });
        }
        HevcBackend::X265 => {
            options.set("preset", "fast");
            options.set("tune", "zerolatency");
            options.set("qp", &qp.to_string());
            // x265's QP 0 is not lossless. Preserve alpha samples explicitly.
            if qp == 0 {
                options.set("x265-params", "lossless=1");
            }
        }
    }
    encoder
        .open_as_with(codec, options)
        .map_err(|error| format!("could not open {name}: {error}"))
}

fn receive_packets(
    encoder: &mut ffmpeg::codec::encoder::video::Encoder,
    output: &mut ffmpeg::format::context::Output,
    stream_index: usize,
    stream_time_base: ffmpeg::Rational,
) -> Result<(), String> {
    loop {
        let mut packet = ffmpeg::Packet::empty();
        match encoder.receive_packet(&mut packet) {
            Ok(()) => {
                packet.set_stream(stream_index);
                packet.rescale_ts(encoder.time_base(), stream_time_base);
                packet
                    .write_interleaved(output)
                    .map_err(|error| format!("could not write visual cache packet: {error}"))?;
            }
            Err(ffmpeg::Error::Other { errno }) if errno == EAGAIN => return Ok(()),
            Err(ffmpeg::Error::Eof) => return Ok(()),
            Err(error) => return Err(format!("could not receive visual cache packet: {error}")),
        }
    }
}
