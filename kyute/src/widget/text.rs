use crate::{
    composable,
    core::DebugNode,
    drawing::{PaintCtx, ToSkia},
    make_uniform_data, theme, BoxLayout, Color, Data, EnvRef, Environment, Event, EventCtx, Font, LayoutCache,
    LayoutCtx, LayoutParams, Measurements, Point, RectI, RoundToPixel, Transform, Widget, WidgetId,
};
use kyute_shell::text::{
    FormattedText, GlyphMaskData, GlyphMaskFormat, GlyphRun, GlyphRunDrawingEffects, Paragraph, ParagraphStyle,
    RasterizationOptions,
};
use lazy_static::lazy_static;
use skia_safe as sk;
use std::{cell::Ref, ptr};
use threadbound::ThreadBound;

////////////////////////////////////////////////////////////////////////////////////////////////////

struct GlyphMaskImage {
    // pixel bounds
    bounds: RectI,
    mask: sk::Image,
}

impl GlyphMaskImage {
    pub fn new(bounds: RectI, data: GlyphMaskData) -> GlyphMaskImage {
        let _span = trace_span!("Create glyph mask image").entered();
        let (_src_bpp, dst_bpp) = match data.format {
            GlyphMaskFormat::Rgb8 => (3usize, 4usize),
            GlyphMaskFormat::Gray8 => (1usize, 1usize),
        };

        let n = (bounds.width() * bounds.height()) as usize;
        let row_bytes = bounds.width() as usize * dst_bpp;
        let mut rgba_buf: Vec<u8> = Vec::with_capacity(n * dst_bpp);

        for i in 0..n {
            let src = &data.data;
            match data.format {
                GlyphMaskFormat::Rgb8 => unsafe {
                    // SAFETY: rgba_buf and src sized accordingly
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4), src[i * 3]);
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4 + 1), src[i * 3 + 1]);
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4 + 2), src[i * 3 + 2]);
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4 + 3), 255);
                },
                GlyphMaskFormat::Gray8 => unsafe {
                    // SAFETY: rgba_buf and src sized accordingly
                    ptr::write(rgba_buf.as_mut_ptr().add(i), src[i]);
                },
            }
        }

        unsafe {
            rgba_buf.set_len(n * dst_bpp);
        }

        // upload RGBA data to a skia image
        let alpha_data = sk::Data::new_copy(&rgba_buf);
        let mask = match data.format {
            GlyphMaskFormat::Rgb8 => sk::Image::from_raster_data(
                &sk::ImageInfo::new(
                    sk::ISize::new(bounds.width(), bounds.height()),
                    sk::ColorType::RGB888x,
                    sk::AlphaType::Unknown,
                    None,
                ),
                alpha_data,
                row_bytes,
            )
            .expect("ImageInfo::new failed"),
            GlyphMaskFormat::Gray8 => sk::Image::from_raster_data(
                &sk::ImageInfo::new(
                    sk::ISize::new(bounds.width(), bounds.height()),
                    sk::ColorType::Gray8,
                    sk::AlphaType::Unknown,
                    None,
                ),
                alpha_data,
                row_bytes,
            )
            .expect("ImageInfo::new failed"),
        };

        GlyphMaskImage { bounds, mask }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// `kyute_text::Renderer` implementation
////////////////////////////////////////////////////////////////////////////////////////////////////

struct Renderer<'a, 'b> {
    ctx: &'a mut PaintCtx<'b>,
    masks: Vec<(RectI, GlyphMaskData)>,
}

const LCD_MASK_BLENDER_SKSL: &str = r#"
layout(color) uniform half4 color;

half4 main(vec4 src, vec4 dst) {
    half4 mask = pow(src, float4(1.0/2.2)); 
    mask *= color.a;

    return half4(
            color.rgb * mask.rgb + dst.rgb * (1.0 - mask.rgb),
            min(1.0, dst.a + max(max(mask.r, mask.g), mask.b)));
}
"#;

lazy_static! {
    static ref APPLY_MASK_EFFECT: ThreadBound<sk::RuntimeEffect> =
        ThreadBound::new(sk::RuntimeEffect::make_for_blender(LCD_MASK_BLENDER_SKSL, None).unwrap());
}

impl<'a, 'b> kyute_shell::text::Renderer for Renderer<'a, 'b> {
    fn draw_glyph_run(&mut self, glyph_run: &GlyphRun, drawing_effects: &GlyphRunDrawingEffects) {
        let analysis = {
            let _span = trace_span!("Analyze glyph run").entered();
            glyph_run.create_glyph_run_analysis(self.ctx.scale_factor, &self.ctx.layer_transform())
        };
        let raster_opts = RasterizationOptions::Subpixel;
        let bounds = analysis.raster_bounds(raster_opts);
        let mask = {
            let _span = trace_span!("Rasterize glyph run").entered();
            analysis.rasterize(raster_opts)
        };
        if let Some(mask) = mask {
            let mask_image = GlyphMaskImage::new(bounds, mask);
            let color = drawing_effects.color;

            let apply_mask_effect = APPLY_MASK_EFFECT.get_ref().unwrap();

            let mask_blender = {
                let (r, g, b, a) = color.to_rgba();
                let uniform_data = make_uniform_data!([apply_mask_effect]
                    color: [f32; 4] = [r, g, b, a];
                );
                apply_mask_effect
                    .make_blender(uniform_data.0, None)
                    .expect("make_blender failed")
                /*// set color uniform
                let mut u_offset = None;
                let mut u_size = None;
                for u in apply_mask_effect.uniforms() {
                    if u.name() == "color" {
                        u_offset = Some(u.offset());
                        u_size = Some(u.size_in_bytes());
                    }
                }
                let u_offset = u_offset.unwrap();
                let u_size = u_size.unwrap();

                let uniform_size = apply_mask_effect.uniform_size();
                assert!(u_offset < uniform_size);
                let mut uniform_data: Vec<u8> = Vec::with_capacity(uniform_size);
                unsafe {
                    let uniform_ptr = uniform_data.as_mut_ptr();
                    let (r, g, b, a) = color.to_rgba();
                    ptr::write(uniform_ptr.add(u_offset).cast::<[f32; 4]>(), [r, g, b, a]);
                    uniform_data.set_len(uniform_size);
                }
                let uniform_data = sk::Data::new_copy(&uniform_data);*/
            };

            let mut paint = sk::Paint::new(color.to_skia(), None);
            paint.set_blender(mask_blender);

            {
                let _span = trace_span!("Draw glyph mask image").entered();
                let canvas = self.ctx.surface.canvas();
                canvas.save();
                //let inv_scale_factor = 1.0 / self.ctx.scale_factor as f32;
                //self.ctx.canvas.scale((inv_scale_factor, inv_scale_factor));
                canvas.reset_matrix();
                canvas.draw_image(
                    &mask_image.mask,
                    sk::Point::new(bounds.origin.x as sk::scalar, bounds.origin.y as sk::scalar),
                    Some(&paint),
                );
                canvas.restore();
            }
        }
    }

    fn transform(&self) -> Transform {
        self.ctx.layer_transform().clone()
    }

    fn scale_factor(&self) -> f64 {
        self.ctx.scale_factor
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Text widget
////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
struct TextLayoutResult {
    paragraph: Paragraph,
    measurements: Measurements,
    font: Font,
    color: Color,
}

/// Displays formatted text.
pub struct Text {
    /// Input formatted text.
    formatted_text: FormattedText,
    /// Font.
    font: EnvRef<Font>,
    /// Text color.
    color: EnvRef<Color>,
    /// The formatted paragraph, calculated during layout. `None` if not yet calculated.
    cached_layout: LayoutCache<TextLayoutResult>,
}

impl Text {
    /// Creates a new text element.
    #[composable]
    pub fn new(formatted_text: impl Into<FormattedText>) -> Text {
        let formatted_text = formatted_text.into();
        //trace!("Text::new {:?}", formatted_text.plain_text);
        Text {
            formatted_text,
            font: EnvRef::Env(theme::DEFAULT_FONT),
            color: EnvRef::Env(theme::TEXT_COLOR),
            cached_layout: Default::default(),
        }
    }

    pub fn font(mut self, font: impl Into<EnvRef<Font>>) -> Self {
        self.font = font.into();
        self
    }

    pub fn color(mut self, color: impl Into<EnvRef<Color>>) -> Self {
        self.color = color.into();
        self
    }

    /// Returns a reference to the formatted text paragraph.
    pub fn paragraph(&self) -> Ref<kyute_shell::text::Paragraph> {
        Ref::map(self.cached_layout.get_cached(), |layout| &layout.paragraph)
    }
}

impl Widget for Text {
    fn widget_id(&self) -> Option<WidgetId> {
        // no need for a stable identity
        None
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> BoxLayout {
        let layout = self.cached_layout.update(ctx, constraints, |ctx| {
            trace!("Text::layout {:?}", self.formatted_text.plain_text);

            let font = self.font.resolve_or_default(env);
            let color = self.color.resolve_or_default(env);
            let font_size = env.get(&theme::FONT_SIZE).unwrap_or(16.0);

            let paragraph_style = ParagraphStyle {
                text_alignment: None,
                font_style: Some(font.style),
                font_weight: Some(font.weight),
                font_size: Some(font_size),
                font_family: Some(font.family.to_string()),
            };
            let paragraph = Paragraph::new(&self.formatted_text, constraints.max, &paragraph_style);

            // measure the paragraph
            let metrics = paragraph.metrics();
            let baseline = paragraph
                .line_metrics()
                .first()
                .map(|line| line.baseline)
                .unwrap_or(0.0);
            let size = constraints.constrain(metrics.bounds.size.round_to_pixel(ctx.scale_factor));

            TextLayoutResult {
                paragraph,
                measurements: Measurements {
                    size,
                    // TODO clip bounds
                    clip_bounds: None,
                    baseline: Some(baseline),
                },
                color,
                font,
            }
        });

        BoxLayout {
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: layout.measurements,
        }
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, ctx: &mut PaintCtx) {
        let _span = trace_span!("Text paint").entered();
        let mut renderer = Renderer { ctx, masks: vec![] };
        // FIXME: should be a point in absolute coords?
        let cached = self.cached_layout.get_cached();
        cached
            .paragraph
            .draw(
                Point::origin(),
                &mut renderer,
                &GlyphRunDrawingEffects { color: cached.color },
            )
            .expect("failed to draw paragraph");
    }

    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("plain text: {:?}", self.formatted_text.plain_text.as_ref()))
    }
}
