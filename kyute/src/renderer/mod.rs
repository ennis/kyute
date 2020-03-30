mod colors;
mod metrics;
mod renderer;

pub use colors::Colors;
pub use renderer::Painter;
pub use renderer::Renderer;
pub use renderer::TextLayout;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ButtonState {
    pub disabled: bool,
    pub clicked: bool,
    pub hot: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TextState {
    Default,
    Disabled,
}

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
