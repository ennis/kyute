use kyute_shell::platform::{Direct2dDrawContext, PlatformWindow};
use crate::layout::{Bounds, Point, Size};
use crate::renderer::colors::DEFAULT_COLORS;
use crate::renderer::metrics::WIDGET_METRICS;
use crate::renderer::ButtonState;
use crate::renderer::WidgetMetrics;

use std::rc::Rc;

use direct2d::brush::SolidColorBrush;
use direct2d::enums::DrawTextOptions;
use direct2d::render_target::IRenderTarget;
use euclid::default::Rect;
use math2d::{Point2f, RoundedRect};
use palette::Srgba;
use kyute_shell::platform::Platform;

/// Internal shared state of the renderer.
pub(super) struct RendererState {
    /// DirectWrite factory
    pub(super) dwrite: directwrite::Factory,
    /// Default DirectWrite TextFormat for text elements
    pub(super) text_format: directwrite::TextFormat,
}

/// The type of text layouts returned by [`Renderer::layout_text`].
pub type TextLayout = directwrite::TextLayout;

/// GUI renderer.
///
/// The [`Renderer`] and the [`Painter`] types are in charge of rendering the primitive elements
/// of a user interface, such as labels, button frames, panel backgrounds, slider knows,
/// scroll bar handles, etc.
///
/// Note that it is the renderer that decides the appearance of those primitive elements:
/// as such, it is more akin to a "style" or "theme" than a low-level drawing API.
/// The renderer specifies the metrics of the primitive elements, such as their minimum or preferred
/// sizes, which you can query with the [`widget_metrics`](Renderer::widget_metrics) method.
///
/// The [`Renderer`] contains the renderer state common to all windows.
/// The related [`Painter`] type represents a painting context of the renderer on a particular
/// window.
///
/// Bikeshedding:
/// First, this type should really be a trait. Possible names:
///  - Theme
///  - Style
///  - Skin
///  - Look / LookAndFeel
///
/// Alternative design:
/// - actually provide a low-level renderer, let the widgets render themselves, customize with
///  ThemeData for the widgets (ButtonTheme, SliderTheme, panel background color)
///     -> better, but the widgets will contain raw d2d code
#[derive(Clone)]
pub struct Renderer(pub(super) Rc<RendererState>);

impl Renderer {
    /// Creates a new GUI renderer from a window context.
    pub fn new(platform: &Platform) -> Renderer {
        let dwrite = platform.directwrite().clone();
        let system_fonts = directwrite::FontCollection::system_font_collection(&dwrite, false)
            .expect("failed to get system font collection");

        let text_format = directwrite::TextFormat::create(&dwrite)
            .with_collection(&system_fonts)
            .with_family("Segoe UI")
            .with_size(14.0)
            .build()
            .expect("failed to build text format");

        Renderer(Rc::new(RendererState {
            dwrite,
            text_format,
        }))
    }

    /// Layouts the specified text using the default text format.
    pub fn layout_text(&self, text: &str, layout_box_size: Size) -> directwrite::TextLayout {
        let dwrite = self.directwrite();
        let format = self.text_format();

        let layout = directwrite::TextLayout::create(dwrite)
            .with_str(text)
            .with_size(layout_box_size.width as f32, layout_box_size.height as f32)
            .with_format(format)
            .build()
            .expect("failed to layout text");
        layout
    }

    /// Returns the [`WidgetMetrics`] of the primitive elements drawn by this renderer.
    ///
    /// This is used in layout calculations.
    pub fn widget_metrics(&self) -> &WidgetMetrics {
        &WIDGET_METRICS
    }

    pub(super) fn text_format(&self) -> &directwrite::TextFormat {
        &self.0.text_format
    }

    pub(super) fn directwrite(&self) -> &directwrite::Factory {
        &self.0.dwrite
    }
}

/// Helper to convert a color to the math2d format used by the direct2d crate.
fn mkcolor(srgba: &Srgba) -> math2d::Color {
    let (r, g, b, a) = srgba.into_components();
    math2d::Color::new(r, g, b, a)
}

/// Helper to convert a rect to the math2d format used by the direct2d crate.
fn mkrectf(rect: Rect<f64>) -> math2d::Rectf {
    let ((l, t), (r, b)) = (rect.min().to_tuple(), rect.max().to_tuple());
    math2d::Rectf::new(l as f32, t as f32, r as f32, b as f32)
}

fn mk_inset_stroke_rectf(rect: Bounds, stroke_size: f64) -> math2d::Rectf {
    let rect = rect.inflate(-0.5 * stroke_size, -0.5 * stroke_size);
    mkrectf(rect)
}

/// Encapsulates a painting context over a window.
pub struct Painter<'a> {
    renderer: &'a Renderer,
    pub(crate) ctx: Direct2dDrawContext<'a>,
}

impl<'a> Drop for Painter<'a> {
    fn drop(&mut self) {
        self.ctx.render_target_mut().end_draw().unwrap()
    }
}

impl<'a> Painter<'a> {
    /// Creates a new painter that will paint on the specified window.
    ///
    /// There can be only one painter active on a window at a time, enforced by exclusive borrow
    /// of the window.
    ///
    /// See also: [`Renderer`], [`PlaformWindow`].
    pub fn new(renderer: &'a Renderer, window: &'a mut PlatformWindow) -> Painter<'a> {
        let mut ctx = Direct2dDrawContext::new(window);

        // prepare the target for drawing
        ctx.render_target_mut().begin_draw();

        Painter { renderer, ctx }
    }

    /// Returns the size in logical pixels of the area that the painter is drawing into.
    pub fn size(&self) -> Size {
        let s = self.ctx.render_target().size();
        Size::new(s.width as f64, s.height as f64)
    }

    /// Returns a reference to the renderer object.
    pub fn renderer(&self) -> &Renderer {
        self.renderer
    }

    /// Returns the window that is painted on.
    pub fn window(&self) -> &PlatformWindow {
        self.ctx.window()
    }

    /// Draws a [`TextLayout`] at the specified location using the default text format.
    ///
    /// See also [`Renderer::layout_text`] to get a [`TextLayout`] object.
    pub fn draw_text(&mut self, at: Point, text: &TextLayout) {
        self.draw_text_internal(at, text, &DEFAULT_COLORS.text);
    }

    /// Draws a [`TextLayout`] at the specified location using the "disabled" text format.
    ///
    /// See also [`Renderer::layout_text`] to get a [`TextLayout`] object.
    pub fn draw_text_disabled(&mut self, at: Point, text: &TextLayout) {
        self.draw_text_internal(at, text, &DEFAULT_COLORS.text_disabled);
    }

    /// Internal helper
    fn draw_text_internal(&mut self, at: Point, text: &TextLayout, color: &Srgba) {
        let target = self.ctx.render_target_mut();
        let brush = SolidColorBrush::new(target, mkcolor(color)).unwrap();
        let text_options = DrawTextOptions::NONE;
        target.draw_text_layout(
            Point2f::new(at.x as f32, at.y as f32),
            &text,
            &brush,
            text_options,
        );
    }

    /// Draws a button frame in the specified bounds.
    pub fn draw_button(&mut self, bounds: Bounds, _state: &ButtonState) {
        let target = self.ctx.render_target_mut();
        let brush = SolidColorBrush::new(target, mkcolor(&DEFAULT_COLORS.button)).unwrap();
        target.fill_rectangle(mkrectf(bounds), &brush);

        let brush = SolidColorBrush::new(target, mkcolor(&DEFAULT_COLORS.border)).unwrap();
        target.draw_rectangle(mk_inset_stroke_rectf(bounds, 1.0), &brush, 1.0, None);
    }

    /// Draws a panel background in the specified bounds.
    pub fn draw_panel_background(&mut self, _bounds: Bounds) {}
}
