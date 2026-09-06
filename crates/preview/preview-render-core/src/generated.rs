use super::*;
use shrimply_math_geometry::ComposedTransform2D;
use shrimply_project::project::{CanvasSize, ItemAddress, Project, Time, VideoItem};
use shrimply_video_core::generated::{
    GeneratedFrame, GeneratedVisual, TextMaskOperation, VectorOperation,
};
use shrimply_video_modifiers::ModifierEffect;

pub(super) struct PreparedVector {
    pub frame: GeneratedFrame,
    pub is_vector: bool,
    pub sample_method: shrimply_render_core::VideoSampleMethod,
    pub effects: Vec<shrimply_video_core::raster_modifiers::Modifier>,
    morph_scene: Option<shrimply_video_core::vector_morph::MorphScene>,
}

impl PreparedVector {
    pub fn morph_scene(
        &self,
        native: CanvasSize,
    ) -> Option<shrimply_video_core::vector_morph::MorphScene> {
        (self.is_vector && self.frame.render_size == native)
            .then(|| self.morph_scene.clone())
            .flatten()
            .and_then(|scene| {
                shrimply_video_core::vector_morph::apply_vector_operations(
                    scene,
                    &self.frame.operations,
                )
            })
    }
}

struct TextSource {
    text: shrimply_video_core::text::PreparedText,
    masks: Vec<TextMaskOperation>,
}

struct SvgSource {
    svg: std::sync::Arc<shrimply_video_core::svg::PreparedSvg>,
    root_size: CanvasSize,
    transition: Option<shrimply_video_core::generated::GeneratedTransition>,
}

impl GeneratedVisual for SvgSource {
    fn draw(
        &self,
        canvas: &skia_safe::Canvas,
        _evaluation: &VisualEvaluation,
        _expressions: &mut TransformExpressionCache,
        path_effect: Option<&skia_safe::PathEffect>,
    ) {
        self.svg
            .draw(canvas, self.root_size, self.transition, path_effect);
    }
}

impl GeneratedVisual for TextSource {
    fn draw(
        &self,
        canvas: &skia_safe::Canvas,
        _evaluation: &VisualEvaluation,
        _expressions: &mut TransformExpressionCache,
        path_effect: Option<&skia_safe::PathEffect>,
    ) {
        self.text.draw_with_masks(canvas, path_effect, &self.masks);
    }
}

impl Scene {
    pub(super) fn vector(
        &mut self,
        project: &Project,
        address: &ItemAddress,
        item: &VideoItem,
        position: Time,
        evaluation: VisualEvaluation,
        native: CanvasSize,
        transform: ComposedTransform2D,
        transition: Option<shrimply_video_core::generated::GeneratedTransition>,
        svg: Option<media::SvgFrame>,
    ) -> Result<PreparedVector, String> {
        let render_size = shrimply_video_core::generated::render_canvas(
            item,
            native,
            &evaluation,
            &mut self.expressions,
        );
        let mut operations = Vec::new();
        let mut effects = Vec::new();
        let mut is_vector = true;
        let mut sample_method = item.sample_method.value_at(evaluation.local_time());
        for (modifier_index, modifier) in item
            .modifiers
            .iter()
            .enumerate()
            .filter(|(_, modifier)| modifier.enabled)
        {
            if is_vector
                && modifier
                    .alpha_mask
                    .as_ref()
                    .is_some_and(|mask| mask.enabled)
            {
                return Err("Vector modifier masks are not yet connected to Metal".into());
            }
            match &modifier.effect {
                ModifierEffect::Vectorize(_) if matches!(item.content, VideoItemContent::Image) => {
                }
                ModifierEffect::Vector(effect) if is_vector => {
                    operations.extend(shrimply_video_core::vector_modifiers::operation(
                        effect,
                        &evaluation,
                        &mut self.expressions,
                    ));
                }
                ModifierEffect::Rasterize(effect) if is_vector => {
                    is_vector = false;
                    sample_method = effect.sample_method.value_at(evaluation.local_time());
                }
                ModifierEffect::Raster(_) if !is_vector => {
                    effects.push(
                        shrimply_video_core::raster_modifiers::modifier(
                            shrimply_video_core::raster_modifiers::ModifierRequest {
                                project,
                                address,
                                item,
                                position,
                                modifier_index,
                                require_complete_assets: false,
                            },
                            &evaluation,
                            &mut self.expressions,
                            self.requested_accuracy.content_accurate(),
                        )?
                        .ok_or("Vector source's raster modifier is not yet connected to Metal")?,
                    );
                }
                _ => {
                    return Err(
                        "Unsupported generated modifier or invalid vector/raster modifier order"
                            .into(),
                    );
                }
            }
        }
        let masks = shrimply_video_core::text::take_masks(&mut operations);
        let (visual, source_offset, morph_scene): (Box<dyn GeneratedVisual>, _, _) =
            match item.content {
                VideoItemContent::Shape(_) => {
                    if !masks.is_empty() {
                        return Err("TextMask requires a text source".into());
                    }
                    let shape = shrimply_video_core::shape::prepare(
                        native,
                        render_size,
                        item,
                        evaluation.clone(),
                        transition,
                        &mut self.expressions,
                    );
                    let offset = ComposedTransform2D {
                        matrix: glam::Mat3::from_translation(-shape.content_offset),
                    };
                    let morph_scene = shape.morph_scene();
                    (Box::new(shape), offset, morph_scene)
                }
                VideoItemContent::Text(_) => {
                    let text = shrimply_video_core::text::prepare(
                        native,
                        render_size,
                        item,
                        evaluation.clone(),
                        transition,
                        &mut self.expressions,
                    );
                    let offset = text.source_offset;
                    let morph_scene = masks.is_empty().then(|| text.morph_scene()).flatten();
                    (Box::new(TextSource { text, masks }), offset, morph_scene)
                }
                VideoItemContent::Paint(_) => {
                    if !masks.is_empty() {
                        return Err("TextMask requires a text source".into());
                    }
                    let paint = shrimply_video_core::paint::prepare(
                        native,
                        render_size,
                        item,
                        evaluation.clone(),
                        transition,
                        &mut self.expressions,
                        self.paint_caches.entry(item.id).or_default().clone(),
                    )?;
                    let morph_scene = paint.morph_scene();
                    (
                        Box::new(paint),
                        ComposedTransform2D {
                            matrix: glam::Mat3::IDENTITY,
                        },
                        morph_scene,
                    )
                }
                VideoItemContent::Svg | VideoItemContent::Image => {
                    if !masks.is_empty() {
                        return Err("TextMask requires a text source".into());
                    }
                    let svg = svg.ok_or("Vector source was not prepared")?;
                    let root_size = svg.size;
                    let morph_scene = svg.prepared.morph_scene(root_size, native, &evaluation);
                    (
                        Box::new(SvgSource {
                            svg: svg.prepared,
                            root_size,
                            transition,
                        }),
                        ComposedTransform2D {
                            matrix: glam::Mat3::IDENTITY,
                        },
                        morph_scene,
                    )
                }
                _ => return Err("Unsupported generated vector source".into()),
            };
        operations.insert(
            0,
            VectorOperation::Transform(transform.compose(source_offset)),
        );
        Ok(PreparedVector {
            frame: GeneratedFrame {
                visual,
                evaluation,
                operations,
                render_size,
                canvas_size: native,
                drawing_strategy: item.skia_drawing_strategy,
            },
            is_vector,
            sample_method,
            effects,
            morph_scene,
        })
    }
}
