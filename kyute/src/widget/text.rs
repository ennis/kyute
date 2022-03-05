use crate::{
    composable, drawing::ToSkia, BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, PaintCtx,
    Point, Rect, Widget, WidgetId,
};
use kyute_common::{Color, RectI, Transform, UnknownUnit};
use kyute_text::{
    FormattedText, GlyphMaskData, GlyphMaskFormat, GlyphRun, GlyphRunDrawingEffects, Paragraph, ParagraphStyle,
    RasterizationOptions,
};
use skia_safe as sk;
use std::{
    cell::{Ref, RefCell},
    ptr,
    sync::Arc,
};

/// Displays formatted text.
#[derive(Clone)]
pub struct Text {
    /// Input formatted text.
    formatted_text: FormattedText,
    /// The formatted paragraph, calculated during layout. `None` if not yet calculated.
    paragraph: RefCell<Option<Paragraph>>,
    run_masks: RefCell<Option<Arc<Vec<GlyphMaskImage>>>>,
}

impl Text {
    /// Creates a new text element.
    #[composable]
    pub fn new(formatted_text: impl Into<FormattedText>) -> Text {
        let formatted_text = formatted_text.into();
        Text {
            formatted_text,
            paragraph: RefCell::new(None),
            run_masks: RefCell::new(None),
        }
    }

    /// Returns a reference to the formatted text paragraph.
    pub fn paragraph(&self) -> Ref<kyute_text::Paragraph> {
        Ref::map(self.paragraph.borrow(), |x| {
            x.as_ref().expect("`Text::paragraph` called before layout")
        })
    }
}

struct GlyphMaskImage {
    // pixel bounds
    bounds: RectI,
    mask: sk::Image,
}

impl GlyphMaskImage {
    pub fn new(bounds: RectI, data: GlyphMaskData) -> GlyphMaskImage {
        let (_src_bpp, dst_bpp) = match data.format() {
            GlyphMaskFormat::Rgb8 => (3usize, 4usize),
            GlyphMaskFormat::Alpha8 => (1usize, 1usize),
        };

        let n = (bounds.width() * bounds.height()) as usize;
        let row_bytes = bounds.width() as usize * dst_bpp;
        let mut rgba_buf: Vec<u8> = Vec::with_capacity(n * dst_bpp);

        for i in 0..n {
            let src = data.data();
            match data.format() {
                GlyphMaskFormat::Rgb8 => unsafe {
                    // SAFETY: rgba_buf and src sized accordingly
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4 + 0), src[i * 3 + 0]);
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4 + 1), src[i * 3 + 1]);
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4 + 2), src[i * 3 + 2]);
                    ptr::write(rgba_buf.as_mut_ptr().add(i * 4 + 3), 255);
                },
                GlyphMaskFormat::Alpha8 => unsafe {
                    // SAFETY: rgba_buf and src sized accordingly
                    ptr::write(rgba_buf.as_mut_ptr().add(i), src[i]);
                },
            }
        }

        unsafe {
            rgba_buf.set_len(n * 4);
        }

        // upload RGBA data to a skia image
        let alpha_data = sk::Data::new_copy(&rgba_buf);
        let mask = match data.format() {
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
            GlyphMaskFormat::Alpha8 => sk::Image::from_raster_data(
                &sk::ImageInfo::new(
                    sk::ISize::new(bounds.width(), bounds.height()),
                    sk::ColorType::Alpha8,
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

impl<'a, 'b> kyute_text::Renderer for Renderer<'a, 'b> {
    fn draw_glyph_run(&mut self, glyph_run: &GlyphRun, drawing_effects: &GlyphRunDrawingEffects) {
        let analysis = glyph_run.create_glyph_run_analysis(self.ctx.scale_factor, &self.ctx.window_transform);
        let raster_opts = RasterizationOptions::Subpixel;
        let bounds = analysis.raster_bounds(raster_opts);
        if let Some(mask) = analysis.rasterize(raster_opts) {
            let mask_image = GlyphMaskImage::new(bounds, mask);
            let color = drawing_effects.color;

            let apply_mask_effect = sk::RuntimeEffect::make_for_blender(LCD_MASK_BLENDER_SKSL, None).unwrap();

            let mask_blender = {
                // set color uniform
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
                let uniform_data = sk::Data::new_copy(&uniform_data);
                apply_mask_effect
                    .make_blender(uniform_data, None)
                    .expect("make_blender failed")
            };

            let mut paint = sk::Paint::new(color.to_skia(), None);
            paint.set_blender(mask_blender);

            self.ctx.canvas.save();
            self.ctx.canvas.reset_matrix();
            //let inv_scale_factor = 1.0 / self.ctx.scale_factor as f32;
            //self.ctx.canvas.scale((inv_scale_factor, inv_scale_factor));
            self.ctx.canvas.draw_image(
                &mask_image.mask,
                sk::Point::new(bounds.origin.x as sk::scalar, bounds.origin.y as sk::scalar),
                Some(&paint),
            );
            self.ctx.canvas.restore();
        }
    }

    fn transform(&self) -> Transform {
        // trace!("window transform: {:?}", self.ctx.window_transform);
        self.ctx.window_transform
    }

    fn scale_factor(&self) -> f64 {
        self.ctx.scale_factor
    }
}

impl Widget for Text {
    fn widget_id(&self) -> Option<WidgetId> {
        // no need for a stable identity
        None
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: BoxConstraints, _env: &Environment) -> Measurements {
        let paragraph = self
            .formatted_text
            .create_paragraph(constraints.max, &ParagraphStyle::default());

        // measure the paragraph
        let metrics = paragraph.metrics();
        let baseline = paragraph
            .line_metrics()
            .first()
            .map(|line| line.baseline)
            .unwrap_or(0.0);
        let size = constraints.constrain(metrics.bounds.size);

        // stash the laid out paragraph for rendering
        self.paragraph.replace(Some(paragraph));

        Measurements {
            bounds: size.into(),
            baseline: Some(baseline),
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, _bounds: Rect, _env: &Environment) {
        //----------------------------------
        let mut paragraph = self.paragraph.borrow_mut();
        let paragraph = paragraph.as_mut().expect("paint called before layout");

        // FIXME: actually cache run masks somehow
        let runs = self.run_masks.borrow_mut();

        if runs.is_none() {
            let mut renderer = Renderer { ctx, masks: vec![] };
            // FIXME: should be a point in absolute coords?
            paragraph.draw(
                Point::origin(),
                &mut renderer,
                &GlyphRunDrawingEffects {
                    color: Color::from_hex("#FFFFFF"),
                },
            );
        }
    }
}
