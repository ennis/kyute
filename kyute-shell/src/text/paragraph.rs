use crate::{
    backend,
    text::{FormattedText, GlyphMaskData, ParagraphStyle, RasterizationOptions, TextPosition},
    Error,
};
use kyute_common::{Color, Data, Point, Rect, RectI, Size, Transform};
use std::ops::Range;

/// Text hit-test metrics.
#[derive(Copy, Clone, Debug, PartialEq, Data)]
pub struct HitTestMetrics {
    /// Text position in UTF-8 code units (bytes).
    pub text_position: TextPosition,
    pub length: usize,
    pub bounds: Rect,
}

/// Return value of [TextLayout::hit_test_point].
#[derive(Copy, Clone, Debug, PartialEq, Data)]
pub struct HitTestPoint {
    pub is_inside: bool,
    // use idx instead of position to better disambiguate "character index in the text string" and "position on screen"
    pub idx: usize,
}

/// Return value of [TextLayout::hit_test_text_position].
#[derive(Copy, Clone, Debug, PartialEq, Data)]
pub struct HitTestTextPosition {
    pub point: Point,
    pub metrics: HitTestMetrics,
}

#[derive(Copy, Clone, Debug, PartialEq, Data)]
pub struct TextMetrics {
    pub bounds: Rect,
    pub width_including_trailing_whitespace: f32,
    pub line_count: u32,
    pub max_bidi_reordering_depth: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Data)]
pub struct LineMetrics {
    pub length: u32,
    pub trailing_whitespace_length: u32,
    pub newline_length: u32,
    pub height: f64,
    pub baseline: f64,
    pub is_trimmed: bool,
}

/// Information about a glyph run: glyph indices, advances and so on.
#[derive(Debug)]
pub struct GlyphRun<'a>(pub(crate) backend::text::GlyphRun<'a>);

impl<'a> GlyphRun<'a> {
    pub fn create_glyph_run_analysis(&self, scale_factor: f64, transform: &Transform) -> GlyphRunAnalysis {
        GlyphRunAnalysis(self.0.create_glyph_run_analysis(scale_factor, transform))
    }
}

/// Information needed to draw a glyph run.
///
/// Contains rendering information calculated after taking into account a text transform and the
/// render target scale factor.
#[derive(Clone)]
pub struct GlyphRunAnalysis(pub(crate) backend::text::GlyphRunAnalysis);

impl GlyphRunAnalysis {
    /// Returns the bounds of rasterized glyph run.
    pub fn raster_bounds(&self, options: RasterizationOptions) -> RectI {
        self.0.raster_bounds(options)
    }

    /// Rasterizes the glyph run.
    ///
    /// The glyph run may be empty (contains no glyphs), in which case this function returns `None`.
    /// Apparently DirectWrite sometimes produces runs with no glyphs in them. Maybe they are whitespace runs?
    pub fn rasterize(&self, options: RasterizationOptions) -> Option<GlyphMaskData> {
        self.0.rasterize(options)
    }
}

/// Drawing parameters passed to `draw_glyph_run`.
#[derive(Clone, Debug)]
pub struct GlyphRunDrawingEffects {
    /// The color of the glyph run.
    pub color: Color,
    // TODO application-defined drawing effects
}

impl Default for GlyphRunDrawingEffects {
    fn default() -> Self {
        GlyphRunDrawingEffects {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

/// Trait for rendering a series of glyph runs.
pub trait Renderer {
    /// Draw a glyph run.
    // TODO error handling?
    fn draw_glyph_run(&mut self, glyph_run: &GlyphRun, drawing_effects: &GlyphRunDrawingEffects);

    /// Returns the current text transformation.
    fn transform(&self) -> Transform;

    /// Returns the scale factor (Physical pixels per DIP).
    fn scale_factor(&self) -> f64;
}

/// A laid-out block of text.
#[derive(Clone)]
pub struct Paragraph(backend::text::Paragraph);

impl Paragraph {
    pub fn new(
        formatted_text: &FormattedText,
        layout_box_size: Size,
        default_paragraph_style: &ParagraphStyle,
    ) -> Paragraph {
        Paragraph(backend::text::Paragraph::new(
            formatted_text,
            layout_box_size,
            default_paragraph_style,
        ))
    }

    pub fn hit_test_point(&self, point: Point) -> HitTestPoint {
        self.0.hit_test_point(point)
    }

    pub fn max_size(&self) -> Size {
        self.0.max_size()
    }

    pub fn hit_test_text_position(&self, text_position: TextPosition) -> HitTestTextPosition {
        self.0.hit_test_text_position(text_position)
    }

    pub fn hit_test_text_range(&self, text_range: Range<usize>, origin: Point) -> Vec<HitTestMetrics> {
        self.0.hit_test_text_range(text_range, origin)
    }

    pub fn metrics(&self) -> TextMetrics {
        self.0.metrics()
    }

    pub fn line_metrics(&self) -> Vec<LineMetrics> {
        self.0.line_metrics()
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
        self.0.draw(origin, renderer, default_drawing_effects)
    }
}
