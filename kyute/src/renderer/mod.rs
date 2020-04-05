mod colors;
mod renderer;
mod text;

pub use colors::Colors;
pub use renderer::Theme;

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

// Renderer refactor:
