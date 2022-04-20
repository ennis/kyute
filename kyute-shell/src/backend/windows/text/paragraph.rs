use crate::{
    application::Application,
    backend::text::{count_until_utf16, count_utf16, dwrite_factory, ToDirectWrite, ToWString},
    text::{
        Attribute, FontStyle, FontWeight, FormattedText, GlyphMaskData, GlyphMaskFormat, GlyphRunDrawingEffects,
        HitTestMetrics, HitTestPoint, HitTestTextPosition, LineMetrics, ParagraphStyle, RasterizationOptions, Renderer,
        TextAffinity, TextAlignment, TextMetrics, TextPosition,
    },
    Error,
};
use kyute_common::{Color, Data, Point, PointI, Rect, RectI, Size, SizeI, Transform};
use std::{cell::RefCell, ffi::c_void, mem, mem::MaybeUninit, ops::Range, ptr, sync::Arc};
use windows::{
    core::{implement, IUnknown, Interface, ToImpl, PCWSTR},
    Win32::{
        Foundation::{BOOL, ERROR_INSUFFICIENT_BUFFER, RECT},
        Graphics::DirectWrite::{
            DWRITE_TEXTURE_ALIASED_1x1, DWRITE_TEXTURE_CLEARTYPE_3x1, IDWriteFontFace, IDWriteGlyphRunAnalysis,
            IDWriteInlineObject, IDWriteNumberSubstitution, IDWriteNumberSubstitution_Impl, IDWritePixelSnapping_Impl,
            IDWriteTextLayout, IDWriteTextRenderer, IDWriteTextRenderer_Impl, DWRITE_FONT_STRETCH_NORMAL,
            DWRITE_GLYPH_RUN, DWRITE_GLYPH_RUN_DESCRIPTION, DWRITE_HIT_TEST_METRICS, DWRITE_LINE_METRICS,
            DWRITE_MATRIX, DWRITE_MEASURING_MODE, DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC, DWRITE_STRIKETHROUGH,
            DWRITE_TEXTURE_TYPE, DWRITE_TEXT_METRICS, DWRITE_TEXT_RANGE, DWRITE_UNDERLINE,
        },
    },
};

/// A laid-out block of text.
#[derive(Clone)]
pub struct Paragraph {
    layout: IDWriteTextLayout,
    text: Arc<str>,
}

/// Returns (start, len).
fn to_dwrite_text_range(text: &str, range: Range<usize>) -> DWRITE_TEXT_RANGE {
    let utf16_start = count_utf16(&text[0..range.start]);
    let utf16_len = count_utf16(&text[range.start..range.end]);

    DWRITE_TEXT_RANGE {
        startPosition: utf16_start as u32,
        length: utf16_len as u32,
    }
}

impl HitTestMetrics {
    pub fn from_dwrite(metrics: &DWRITE_HIT_TEST_METRICS, text: &str, is_trailing: bool) -> HitTestMetrics {
        // convert utf16 code unit offset to utf8
        //dbg!(metrics.textPosition);
        let text_position = count_until_utf16(text, metrics.textPosition as usize);
        let length = count_until_utf16(&text[text_position..], metrics.length as usize);

        let text_position = TextPosition {
            position: text_position,
            affinity: if is_trailing {
                TextAffinity::Downstream
            } else {
                TextAffinity::Upstream
            },
        };

        HitTestMetrics {
            text_position,
            length,
            bounds: Rect::new(
                Point::new(metrics.left as f64, metrics.top as f64),
                Size::new(metrics.width as f64, metrics.height as f64),
            ),
        }
    }
}

impl From<DWRITE_TEXT_METRICS> for TextMetrics {
    fn from(m: DWRITE_TEXT_METRICS) -> Self {
        TextMetrics {
            bounds: Rect::new(
                Point::new(m.left as f64, m.top as f64),
                Size::new(m.width as f64, m.height as f64),
            ),
            width_including_trailing_whitespace: m.widthIncludingTrailingWhitespace,
            max_bidi_reordering_depth: m.maxBidiReorderingDepth,
            line_count: m.lineCount,
        }
    }
}

impl From<DWRITE_LINE_METRICS> for LineMetrics {
    fn from(m: DWRITE_LINE_METRICS) -> Self {
        LineMetrics {
            length: m.length,
            trailing_whitespace_length: m.trailingWhitespaceLength,
            newline_length: m.newlineLength,
            height: m.height as f64,
            baseline: m.baseline as f64,
            is_trimmed: m.isTrimmed.as_bool(),
        }
    }
}

impl Paragraph {
    pub fn hit_test_point(&self, point: Point) -> HitTestPoint {
        unsafe {
            // influenced by piet-direct2d (https://github.com/linebender/piet/blob/f6abb8720f4a5e952c9ed028a6213f6b10974a0b/piet-direct2d/src/text.rs#L381)
            let mut is_trailing_hit = false.into();
            let mut is_inside = false.into();
            let mut metrics = MaybeUninit::<DWRITE_HIT_TEST_METRICS>::uninit();
            self.layout
                .HitTestPoint(
                    point.x as f32,
                    point.y as f32,
                    &mut is_trailing_hit,
                    &mut is_inside,
                    metrics.as_mut_ptr(),
                )
                .expect("HitTestPoint failed");
            let metrics = metrics.assume_init();
            let is_trailing_hit = is_trailing_hit.as_bool();
            let is_inside = is_inside.as_bool();

            // if hit test reports a hit on the trailing side of the grapheme cluster, skip to the next position
            // (we return the cursor position, not the character position)
            let idx_utf16 = if is_trailing_hit {
                metrics.textPosition + metrics.length
            } else {
                metrics.textPosition
            } as usize;

            // utf8 cursor pos
            let idx = count_until_utf16(&self.text, idx_utf16);

            HitTestPoint { is_inside, idx }
        }
    }

    /// Returns the layout maximum size.
    pub fn max_size(&self) -> Size {
        unsafe {
            let w = self.layout.GetMaxWidth();
            let h = self.layout.GetMaxHeight();
            Size::new(w as f64, h as f64)
        }
    }

    pub fn hit_test_text_position(&self, text_position: TextPosition) -> HitTestTextPosition {
        // convert the text position to an utf-16 offset (inspired by piet-direct2d).
        let pos_utf16 = count_utf16(&self.text[0..text_position.position]);

        unsafe {
            let mut point_x = 0.0f32;
            let mut point_y = 0.0f32;
            let mut metrics = MaybeUninit::<DWRITE_HIT_TEST_METRICS>::uninit();
            let is_trailing_hit = match text_position.affinity {
                TextAffinity::Upstream => false,
                TextAffinity::Downstream => true,
            };

            self.layout
                .HitTestTextPosition(
                    pos_utf16 as u32,
                    false,
                    &mut point_x,
                    &mut point_y,
                    metrics.as_mut_ptr(),
                )
                .expect("HitTestTextPosition failed");

            HitTestTextPosition {
                metrics: HitTestMetrics::from_dwrite(&metrics.assume_init(), &self.text, is_trailing_hit),
                point: Point::new(point_x as f64, point_y as f64),
            }
        }
    }

    pub fn hit_test_text_range(&self, text_range: Range<usize>, origin: Point) -> Vec<HitTestMetrics> {
        unsafe {
            // convert range to UTF16
            let text_position = count_utf16(&self.text[0..text_range.start]);
            let text_length = count_utf16(&self.text[text_range]);

            // first call to determine the count
            let text_metrics = self.layout.GetMetrics().expect("GetMetrics failed");

            // "A good value to use as an initial value for maxHitTestMetricsCount
            // may be calculated from the following equation:
            // maxHitTestMetricsCount = lineCount * maxBidiReorderingDepth"
            // (https://docs.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextlayout-hittesttextrange)
            let mut max_metrics_count = text_metrics.lineCount * text_metrics.maxBidiReorderingDepth;
            let mut metrics = vec![Default::default(); max_metrics_count as usize];

            let result = self.layout.HitTestTextRange(
                text_position as u32,
                text_length as u32,
                origin.x as f32,
                origin.y as f32,
                &mut metrics,
                &mut max_metrics_count,
            );

            if let Err(e) = result {
                if e.code() == ERROR_INSUFFICIENT_BUFFER.into() {
                    // reallocate with sufficient space
                    metrics.resize(max_metrics_count as usize, Default::default());
                    self.layout
                        .HitTestTextRange(
                            text_position as u32,
                            text_length as u32,
                            origin.x as f32,
                            origin.y as f32,
                            &mut metrics,
                            &mut max_metrics_count,
                        )
                        .expect("HitTestTextRange failed");
                } else {
                    panic!("HitTestTextRange failed");
                }
            }

            metrics
                .into_iter()
                .take(max_metrics_count as usize)
                .map(|m| HitTestMetrics::from_dwrite(&m, &self.text, true))
                .collect()
        }
    }

    pub fn metrics(&self) -> TextMetrics {
        unsafe {
            let metrics = self.layout.GetMetrics().expect("GetMetrics failed");
            metrics.into()
        }
    }

    pub fn line_metrics(&self) -> Vec<LineMetrics> {
        unsafe {
            let mut line_count = 1;
            let mut metrics = vec![Default::default(); line_count as usize];
            let result = self.layout.GetLineMetrics(&mut metrics, &mut line_count);

            if let Err(e) = result {
                if e.code() == ERROR_INSUFFICIENT_BUFFER.into() {
                    // reallocate with sufficient space
                    metrics.resize(line_count as usize, Default::default());
                    self.layout
                        .GetLineMetrics(&mut metrics, &mut line_count)
                        .expect("GetLineMetrics failed");
                }
            }

            metrics
                .into_iter()
                .take(line_count as usize)
                .map(|m| m.into())
                .collect()
        }
    }

    /// Draws the paragraph with the specified renderer.
    ///
    /// This function calls `draw_glyph_run` on the provided renderer for each glyph run in the paragraph.
    pub fn draw(
        &self,
        origin: Point,
        renderer: &mut dyn Renderer,
        default_drawing_effects: &GlyphRunDrawingEffects,
    ) -> Result<(), Error> {
        unsafe {
            // DANGER ZONE: erase lifetime on renderer
            // TODO: not sure that this is entirely safe
            let renderer = renderer as *mut dyn Renderer;
            let renderer = mem::transmute(renderer);

            let dwrite_renderer: IDWriteTextRenderer = DWriteRendererProxy {
                renderer,
                default_drawing_effects,
            }
            .into();
            self.layout
                .Draw(ptr::null(), dwrite_renderer, origin.x as f32, origin.y as f32)?
        };

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FontFace {
    font_face: IDWriteFontFace,
}

fn to_dwrite_texture_type(rasterization_options: RasterizationOptions) -> DWRITE_TEXTURE_TYPE {
    match rasterization_options {
        RasterizationOptions::Bilevel => DWRITE_TEXTURE_ALIASED_1x1,
        RasterizationOptions::Grayscale => DWRITE_TEXTURE_CLEARTYPE_3x1,
        RasterizationOptions::Subpixel => DWRITE_TEXTURE_CLEARTYPE_3x1,
    }
}

/// Information needed to draw a glyph run.
///
/// Contains rendering information calculated after taking into account a text transform and the
/// render target scale factor.
#[derive(Clone)]
pub struct GlyphRunAnalysis {
    analysis: IDWriteGlyphRunAnalysis,
}

impl GlyphRunAnalysis {
    /// Returns the bounds of rasterized glyph run.
    pub fn raster_bounds(&self, options: RasterizationOptions) -> RectI {
        let texture_type = to_dwrite_texture_type(options);
        unsafe {
            let bounds: RECT = self.analysis.GetAlphaTextureBounds(texture_type).unwrap();
            RectI::new(
                PointI::new(bounds.left, bounds.top),
                SizeI::new(bounds.right - bounds.left, bounds.bottom - bounds.top),
            )
        }
    }

    /// Rasterizes the glyph run.
    ///
    /// The glyph run may be empty (contains no glyphs), in which case this function returns `None`.
    /// Apparently DirectWrite sometimes produces runs with no glyphs in them. Maybe they are whitespace runs?
    pub fn rasterize(&self, options: RasterizationOptions) -> Option<GlyphMaskData> {
        let texture_type = to_dwrite_texture_type(options);

        unsafe {
            let bounds: RECT = self.analysis.GetAlphaTextureBounds(texture_type).unwrap();
            let width = bounds.right - bounds.left;
            let height = bounds.bottom - bounds.top;

            if width == 0 || height == 0 {
                // nothing to render
                return None;
            }

            // create the rendering params (using the default settings for the primary monitor)
            // TODO: per-monitor rendering params
            let rendering_params = dwrite_factory()
                .CreateRenderingParams()
                .expect("CreateRenderingParams failed");

            // fetch gamma params
            let mut blend_gamma = 0.0f32;
            let mut blend_enhanced_contrast = 0.0f32;
            let mut blend_clear_type_level = 0.0f32;
            self.analysis
                .GetAlphaBlendParams(
                    rendering_params,
                    &mut blend_gamma,
                    &mut blend_enhanced_contrast,
                    &mut blend_clear_type_level,
                )
                .unwrap();

            /*eprintln!(
                "alpha blend params {} {} {}",
                blend_gamma, blend_enhanced_contrast, blend_clear_type_level
            );*/

            let buffer_size = match texture_type {
                DWRITE_TEXTURE_ALIASED_1x1 => (width * height) as usize,
                DWRITE_TEXTURE_CLEARTYPE_3x1 => (3 * width * height) as usize,
                _ => unreachable!(),
            };

            let mut data = Vec::with_capacity(buffer_size);
            self.analysis
                .CreateAlphaTexture(texture_type, &bounds, data.as_mut_ptr(), buffer_size as u32)
                .expect("CreateAlphaTexture failed");
            data.set_len(buffer_size);

            let format = match texture_type {
                DWRITE_TEXTURE_ALIASED_1x1 => GlyphMaskFormat::Alpha8,
                DWRITE_TEXTURE_CLEARTYPE_3x1 => GlyphMaskFormat::Rgb8,
                _ => unreachable!(),
            };

            Some(GlyphMaskData {
                size: SizeI::new(width, height),
                format,
                data,
            })
        }
    }
}

/// Information about a glyph run: glyph indices, advances and so on.
#[derive(Clone, Debug)]
pub struct GlyphRun<'a> {
    client_drawing_context: *const c_void,
    baseline_origin_x: f32,
    baseline_origin_y: f32,
    measuring_mode: DWRITE_MEASURING_MODE,
    glyph_run: &'a DWRITE_GLYPH_RUN,
    glyph_run_description: &'a DWRITE_GLYPH_RUN_DESCRIPTION,
    // TODO: analysis cache?
    analysis: RefCell<Option<IDWriteGlyphRunAnalysis>>,
}

impl<'a> GlyphRun<'a> {
    /// Creates a `GlyphRunAnalysis` object containing rendering information for the given scale factor and transformation.
    pub fn create_glyph_run_analysis(&self, scale_factor: f64, transform: &Transform) -> GlyphRunAnalysis {
        let transform = transform.to_dwrite();
        //eprintln!("transform={:?}", transform);
        let analysis: IDWriteGlyphRunAnalysis = unsafe {
            dwrite_factory()
                .CreateGlyphRunAnalysis(
                    self.glyph_run,
                    scale_factor as f32,
                    &transform,
                    // TODO should probably be controlled by the client;
                    // - NATURAL for small fonts, SYMMETRIC for bigger things
                    DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
                    self.measuring_mode,
                    self.baseline_origin_x,
                    self.baseline_origin_y,
                )
                .expect("CreateGlyphRunAnalysis failed")
        };
        GlyphRunAnalysis { analysis }
    }
}

/// Drawing attributes passed to IDWriteTextLayout (via SetDrawingEffect).
// FIXME: `#[implement(IUnknown)]` doesn't work for now, so instead implement a random-ass interface without any methods
#[implement(IDWriteNumberSubstitution)]
struct GlyphRunDrawingEffectsWrapper(GlyphRunDrawingEffects);
impl IDWriteNumberSubstitution_Impl for GlyphRunDrawingEffectsWrapper {}

/// Custom IDWriteTextRenderer. Delegates to a `Renderer` instance.
#[implement(IDWriteTextRenderer)]
struct DWriteRendererProxy {
    default_drawing_effects: *const GlyphRunDrawingEffects,
    renderer: *mut dyn Renderer,
}

impl IDWritePixelSnapping_Impl for DWriteRendererProxy {
    fn IsPixelSnappingDisabled(&self, _clientdrawingcontext: *const c_void) -> windows::core::Result<BOOL> {
        Ok(false.into())
    }

    fn GetCurrentTransform(&self, _clientdrawingcontext: *const c_void) -> windows::core::Result<DWRITE_MATRIX> {
        let transform = unsafe {
            // SAFETY: ensured by lifetime of DWriteRendererProxy in Paragraph::draw
            (&*self.renderer).transform()
        };
        Ok(transform.to_dwrite())
    }

    fn GetPixelsPerDip(&self, _clientdrawingcontext: *const c_void) -> windows::core::Result<f32> {
        let scale_factor = unsafe {
            // SAFETY: ensured by lifetime of DWriteRendererProxy in Paragraph::draw
            (&*self.renderer).scale_factor()
        };
        Ok(scale_factor as f32)
    }
}

impl IDWriteTextRenderer_Impl for DWriteRendererProxy {
    fn DrawGlyphRun(
        &self,
        clientdrawingcontext: *const c_void,
        baselineoriginx: f32,
        baselineoriginy: f32,
        measuringmode: DWRITE_MEASURING_MODE,
        glyphrun: *const DWRITE_GLYPH_RUN,
        glyphrundescription: *const DWRITE_GLYPH_RUN_DESCRIPTION,
        clientdrawingeffect: &Option<IUnknown>,
    ) -> windows::core::Result<()> {
        unsafe {
            let glyph_run = crate::text::GlyphRun(GlyphRun {
                client_drawing_context: clientdrawingcontext,
                baseline_origin_x: baselineoriginx,
                baseline_origin_y: baselineoriginy,
                measuring_mode: measuringmode,
                // SAFETY: only borrowed for the duration of the function; cannot escape through `Renderer::draw_glyph_run` because of lifetime bound.
                glyph_run: &*glyphrun,
                glyph_run_description: &*glyphrundescription,
                analysis: RefCell::new(None),
            });

            if let Some(client_drawing_effect) = clientdrawingeffect {
                // SAFETY: the only drawing effect passed here is an instance of DWriteRendererProxy.
                // TODO erase this disgrace once `implement(IUnknown)` works.
                let whatever: IDWriteNumberSubstitution = client_drawing_effect.cast().unwrap();
                let drawing_effects: &mut GlyphRunDrawingEffectsWrapper = ToImpl::to_impl(&whatever);
                // SAFETY: drawing effect lives as long as the draw call
                (&mut *self.renderer).draw_glyph_run(&glyph_run, &drawing_effects.0);
            } else {
                // SAFETY: drawing effect lives as long as the draw call
                (&mut *self.renderer).draw_glyph_run(&glyph_run, &*self.default_drawing_effects);
            };

            Ok(())
        }
    }

    fn DrawUnderline(
        &self,
        _clientdrawingcontext: *const c_void,
        _baselineoriginx: f32,
        _baselineoriginy: f32,
        _underline: *const DWRITE_UNDERLINE,
        _clientdrawingeffect: &Option<::windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        todo!()
    }

    fn DrawStrikethrough(
        &self,
        _clientdrawingcontext: *const c_void,
        _baselineoriginx: f32,
        _baselineoriginy: f32,
        _strikethrough: *const DWRITE_STRIKETHROUGH,
        _clientdrawingeffect: &Option<::windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        todo!()
    }

    fn DrawInlineObject(
        &self,
        _clientdrawingcontext: *const c_void,
        _originx: f32,
        _originy: f32,
        _inlineobject: &Option<IDWriteInlineObject>,
        _issideways: BOOL,
        _isrighttoleft: BOOL,
        _clientdrawingeffect: &Option<::windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        todo!()
    }
}

impl Paragraph {
    pub fn new(
        formatted_text: &FormattedText,
        layout_box_size: Size,
        default_paragraph_style: &ParagraphStyle,
    ) -> Paragraph {
        unsafe {
            let dwrite_factory = &Application::instance().backend.dwrite_factory.0;
            let text_wide = formatted_text.plain_text.to_wstring();

            // FIXME get last-resort defaults from system settings
            const DEFAULT_FONT_FAMILY: &str = "Segoe UI";
            const DEFAULT_FONT_SIZE: f64 = 14.0;
            let locale_name = "".to_wstring();

            let paragraph_font_family = formatted_text
                .paragraph_style
                .font_family
                .as_deref()
                .or(default_paragraph_style.font_family.as_deref())
                .unwrap_or(DEFAULT_FONT_FAMILY)
                .to_wstring();
            let paragraph_font_style = formatted_text
                .paragraph_style
                .font_style
                .or(default_paragraph_style.font_style)
                .unwrap_or(FontStyle::Normal)
                .to_dwrite();
            let paragraph_font_weight = formatted_text
                .paragraph_style
                .font_weight
                .or(default_paragraph_style.font_weight)
                .unwrap_or(FontWeight::NORMAL)
                .to_dwrite();
            let paragraph_text_alignment = formatted_text
                .paragraph_style
                .text_alignment
                .or(default_paragraph_style.text_alignment)
                .unwrap_or(TextAlignment::Leading)
                .to_dwrite();
            let paragraph_font_size = formatted_text
                .paragraph_style
                .font_size
                .or(default_paragraph_style.font_size)
                .unwrap_or(DEFAULT_FONT_SIZE);

            let format = dwrite_factory
                .CreateTextFormat(
                    PCWSTR(paragraph_font_family.as_ptr()),
                    None,
                    paragraph_font_weight,
                    paragraph_font_style,
                    DWRITE_FONT_STRETCH_NORMAL,
                    paragraph_font_size as f32,
                    PCWSTR(locale_name.as_ptr()),
                )
                .expect("CreateTextFormat failed");

            let layout: IDWriteTextLayout = dwrite_factory
                .CreateTextLayout(
                    &text_wide,
                    format,
                    layout_box_size.width as f32,
                    layout_box_size.height as f32,
                )
                .expect("CreateTextLayout failed");

            layout
                .SetTextAlignment(paragraph_text_alignment)
                .expect("SetTextAlignment failed");

            // apply style ranges
            for run in formatted_text.runs.runs.iter() {
                let mut font_family = None;
                let mut font_weight = None;
                let mut font_style = None;
                //let mut font_stretch = None;
                let mut font_size = None;
                let mut color = None;

                for attr in run.attributes.iter() {
                    match *attr {
                        Attribute::FontSize(size) => font_size = Some(size),
                        Attribute::FontFamily(ref ff) => {
                            font_family = Some(ff);
                        }
                        Attribute::FontStyle(fs) => {
                            font_style = Some(fs);
                        }
                        Attribute::FontWeight(fw) => {
                            font_weight = Some(fw);
                        }
                        Attribute::Color(c) => {
                            color = Some(c);
                        }
                    }
                }

                let range = to_dwrite_text_range(&formatted_text.plain_text, run.range.clone());

                if let Some(ff) = font_family {
                    let ff_name = ff.name().to_wstring();
                    layout
                        .SetFontFamilyName(PCWSTR(ff_name.as_ptr()), range)
                        .expect("SetFontFamilyName failed");
                }

                if let Some(fs) = font_size {
                    layout.SetFontSize(fs as f32, range).expect("SetFontSize failed");
                }

                if let Some(fw) = font_weight {
                    layout
                        .SetFontWeight(fw.to_dwrite(), range)
                        .expect("SetFontWeight failed");
                }

                if let Some(fs) = font_style {
                    layout.SetFontStyle(fs.to_dwrite(), range).expect("SetFontStyle failed");
                }

                if let Some(color) = color {
                    let effect: IUnknown = GlyphRunDrawingEffectsWrapper(GlyphRunDrawingEffects { color }).into();
                    layout.SetDrawingEffect(effect, range).expect("SetDrawingEffect failed");
                }
            }

            Paragraph {
                layout,
                text: formatted_text.plain_text.clone(),
            }
        }
    }
}
