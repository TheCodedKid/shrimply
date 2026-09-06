use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use imagesize::ImageType;
use shrimply_project::project::project_directory;

use super::*;

const CLIPBOARD_MEDIA_DIR: &str = "media/clipboard";
const MAX_CLIPBOARD_IMAGE_BYTES: usize = 100 * 1024 * 1024;

pub fn store_clipboard_image(bytes: &[u8]) -> Result<PathBuf, String> {
    store_clipboard_raster(bytes, None)
}

pub fn store_clipboard_image_with_extension(
    bytes: &[u8],
    extension_hint: &str,
) -> Result<PathBuf, String> {
    if extension_hint.eq_ignore_ascii_case("svg") {
        validate_clipboard_image_length(bytes.len())?;
        let text = std::str::from_utf8(bytes)
            .map_err(|error| format!("clipboard SVG is not UTF-8: {error}"))?;
        if !text.contains("<svg") {
            return Err("clipboard SVG does not contain an SVG document".into());
        }
        return store_clipboard_bytes(bytes, "svg");
    }
    store_clipboard_raster(bytes, Some(extension_hint))
}

pub fn image_extension_for_content_type(content_type: &str) -> Option<&'static str> {
    match content_type.split(';').next()?.trim() {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/avif" => Some("avif"),
        "image/svg+xml" => Some("svg"),
        _ => None,
    }
}

pub fn image_extension_for_url(url: &str) -> Option<&'static str> {
    let extension = url
        .split(['?', '#'])
        .next()?
        .rsplit_once('.')?
        .1
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("png"),
        "jpg" | "jpeg" => Some("jpg"),
        "gif" => Some("gif"),
        "webp" => Some("webp"),
        "avif" => Some("avif"),
        "svg" => Some("svg"),
        _ => None,
    }
}

pub fn validate_clipboard_image_length(length: usize) -> Result<(), String> {
    if length == 0 {
        return Err("clipboard image is empty".into());
    }
    if length > MAX_CLIPBOARD_IMAGE_BYTES {
        return Err("clipboard image is larger than 100 MiB".into());
    }
    Ok(())
}

pub fn store_clipboard_visual_file(path: &Path) -> Result<Option<PathBuf>, String> {
    let Some(kind) = crate::import::file_kind(path) else {
        return Ok(None);
    };
    if !matches!(
        kind,
        crate::import::FileKind::Image
            | crate::import::FileKind::Gif
            | crate::import::FileKind::Svg
            | crate::import::FileKind::Pdf
    ) {
        return Ok(None);
    }
    let directory = project_directory().join(CLIPBOARD_MEDIA_DIR);
    if path.starts_with(&directory) {
        return Ok(Some(path.to_owned()));
    }
    let bytes = read_clipboard_file(path)?;
    if matches!(
        kind,
        crate::import::FileKind::Image | crate::import::FileKind::Gif
    ) {
        return store_clipboard_raster(
            &bytes,
            path.extension().and_then(|extension| extension.to_str()),
        )
        .map(Some);
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or("clipboard visual file has no extension")?;
    if bytes.is_empty() {
        return Err("clipboard visual file is empty".into());
    }
    store_clipboard_bytes(&bytes, extension).map(Some)
}

fn read_clipboard_file(path: &Path) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|error| {
            format!(
                "could not open clipboard visual {}: {error}",
                path.display()
            )
        })?
        .take((MAX_CLIPBOARD_IMAGE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "could not read clipboard visual {}: {error}",
                path.display()
            )
        })?;
    validate_clipboard_image_length(bytes.len())?;
    Ok(bytes)
}

fn store_clipboard_raster(bytes: &[u8], extension_hint: Option<&str>) -> Result<PathBuf, String> {
    validate_clipboard_image_length(bytes.len())?;

    let extension = match imagesize::image_type(bytes)
        .map_err(|error| format!("clipboard image cannot be parsed: {error}"))?
    {
        ImageType::Png => "png",
        ImageType::Jpeg => "jpg",
        ImageType::Webp => "webp",
        ImageType::Gif => "gif",
        ImageType::Heif(_)
            if extension_hint.is_some_and(|extension| extension.eq_ignore_ascii_case("avif")) =>
        {
            "avif"
        }
        _ => return Err("clipboard image uses an unsupported format".into()),
    };
    let size = imagesize::blob_size(bytes)
        .map_err(|error| format!("clipboard image is invalid: {error}"))?;
    if size.width == 0 || size.height == 0 {
        return Err("clipboard image has no pixels".into());
    }

    store_clipboard_bytes(bytes, extension)
}

fn store_clipboard_bytes(bytes: &[u8], extension: &str) -> Result<PathBuf, String> {
    let directory = project_directory().join(CLIPBOARD_MEDIA_DIR);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("could not create clipboard media directory: {error}"))?;
    let path = directory.join(format!("{}.{}", uuid::Uuid::new_v4(), extension));
    fs::write(&path, bytes).map_err(|error| format!("could not store clipboard image: {error}"))?;
    Ok(path)
}

#[derive(Clone)]
pub struct TextPreview {
    pub text: String,
    pub kind: TrackKind,
    pub track_index: usize,
    pub start: Time,
    pub end: Time,
}
