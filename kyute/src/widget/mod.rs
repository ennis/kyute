//! built-in widgets.
mod align;
mod baseline;
mod button;
mod clickable;
mod constrained;
mod container;
pub mod drop_down;
mod flex;
pub mod grid;
mod image;
mod label;
mod layout_wrapper;
mod menu;
mod null;
mod padding;
mod separator;
mod slider;
mod text;
mod text_edit;
//mod text_v1;
mod border;
mod canvas;
mod color_picker;
//mod layer_widget;
mod env_override;
mod formatter;
mod popup;
mod scroll_area;
mod selectable;
mod stepper;
mod table;
mod text_input;
mod thumb;
mod titled_pane;
mod widget_pod;

pub use align::Align;
pub use baseline::Baseline;
pub use border::Border;
pub use button::Button;
pub use canvas::{Canvas, Viewport};
pub use clickable::Clickable;
pub use color_picker::{ColorPaletteItem, ColorPicker, ColorPickerMode, ColorPickerParams, HsvColorSquare};
pub use constrained::ConstrainedBox;
pub use container::Container;
pub use drop_down::DropDown;
pub use env_override::EnvOverride;
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use formatter::{DisplayFormatter, FloatingPointNumberFormatter, Formatter, ValidationResult};
pub use grid::{Grid, GridLength};
pub use image::{Image, Scaling};
pub use kyute_macros::WidgetWrapper;
pub use label::Label;
//pub use layer_widget::LayerWidget;
pub use layout_wrapper::LayoutInspector;
pub use menu::{Action, ContextMenu, Menu, MenuItem, Shortcut};
pub use null::Null;
pub use padding::Padding;
pub use popup::Popup;
pub use scroll_area::ScrollArea;
pub use selectable::Selectable;
pub use separator::separator;
pub use slider::Slider;
pub use stepper::Stepper;
pub use table::{ColumnHeaders, TableRow, TableSelection, TableView, TableViewParams};
pub use text::Text;
pub use text_edit::TextEdit;
pub use text_input::{StepperTextInput, TextInput};
pub use thumb::{DragController, Thumb};
pub use titled_pane::TitledPane;
pub use widget_pod::WidgetPod;

use crate::{
    animation::PaintCtx, BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, Widget, WidgetId,
};

// TODO move somewhere else
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    pub fn cross_orientation(self) -> Orientation {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

/// Widgets that have only one child and wish to defer to this child's `Widget` implementation.
pub trait WidgetWrapper {
    type Inner: Widget + ?Sized;

    fn inner(&self) -> &Self::Inner;
    fn inner_mut(&mut self) -> &mut Self::Inner;

    fn widget_id(&self) -> Option<WidgetId> {
        self.inner().widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner().event(ctx, event, env)
    }

    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner().route_event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner().layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner().paint(ctx)
    }
}

impl<T: WidgetWrapper> Widget for T {
    fn widget_id(&self) -> Option<WidgetId> {
        WidgetWrapper::widget_id(self)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        WidgetWrapper::layout(self, ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        WidgetWrapper::event(self, ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        WidgetWrapper::paint(self, ctx)
    }

    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        WidgetWrapper::route_event(self, ctx, event, env)
    }
}

/// Prelude containing commonly used items for writing custom widgets.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        animation::PaintCtx, cache::Signal, composable, widget::WidgetPod, Alignment, BoxConstraints, Environment,
        Event, EventCtx, LayoutCache, LayoutCtx, Measurements, Offset, Orientation, Point, Rect, Size, Transform,
        Widget, WidgetId,
    };
}
