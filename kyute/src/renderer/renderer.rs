use crate::layout::{Bounds, Point, Size};
use crate::renderer::colors::DEFAULT_COLORS;
use crate::renderer::ButtonState;
use euclid::default::Rect;
use kyute_shell::drawing::{Color, RenderTarget};
use kyute_shell::platform::Platform;
use kyute_shell::text::{TextFormat, TextFormatBuilder, TextLayout};
use kyute_shell::window::DrawContext;
use palette::Srgba;
use std::ops::Range;
use std::rc::Rc;

/// Metrics of a button frame renderered by [`Renderer::draw_button`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ButtonMetrics {
    pub min_width: f64,
    pub min_height: f64,
    pub label_padding: f64,
}

/// Contains the metrics of the primitive elements of a renderer.
///
/// The `Renderer` decides how to render the primitive elements of the application,
/// such as button frames, panel backgrounds, scroll bars, .
/// As such,
///
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WidgetMetrics {
    pub button_metrics: ButtonMetrics,
}

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
#[derive(Clone)]
pub struct Theme {
    /// The text format used for labels on buttons and such.
    pub label_text_format: TextFormat,
    pub button_metrics: ButtonMetrics,
}

impl Theme {
    /// Creates the default UI theme
    pub fn new(platform: &Platform) -> Theme {
        let label_text_format = TextFormatBuilder::new(platform)
            .family("Segoe UI")
            .size(14.0)
            .build()
            .unwrap(); // TODO try something else on failure

        Theme {
            label_text_format,
            button_metrics: DEFAULT_BUTTON_METRICS,
        }
    }

    pub fn button_metrics(&self) -> &ButtonMetrics {
        &self.button_metrics
    }

    /// Draws a button frame in the specified bounds.
    pub fn draw_button_frame(
        &self,
        target: &mut RenderTarget,
        bounds: Bounds,
        _state: &ButtonState,
    ) {
        target.fill_rectangle(bounds, DEFAULT_COLORS.button);
        let stroke_size = 1.0;
        target.draw_rectangle(
            bounds.inflate(-0.5 * stroke_size, -0.5 * stroke_size),
            DEFAULT_COLORS.border,
            1.0,
        );
    }

    /// Draws a panel background in the specified bounds.
    pub fn draw_panel_background(&self, _target: &mut RenderTarget, _bounds: Bounds) {}
}

/// Default button metrics.
const DEFAULT_BUTTON_METRICS: ButtonMetrics = ButtonMetrics {
    min_width: 10.0,
    min_height: 10.0,
    label_padding: 4.0,
};
