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
mod popup;
mod scroll_area;
mod selectable;
mod table;
mod thumb;
mod titled_pane;

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
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use grid::{Grid, GridLength, GridRow, GridSpan};
pub use image::{Image, Scaling};
pub use label::Label;
pub use layout_wrapper::LayoutInspector;
pub use menu::{Action, ContextMenu, Menu, MenuItem, Shortcut};
pub use null::Null;
pub use padding::Padding;
pub use popup::Popup;
pub use scroll_area::ScrollArea;
pub use selectable::Selectable;
pub use separator::separator;
pub use slider::Slider;
pub use table::{ColumnHeaders, TableRow, TableSelection, TableView, TableViewParams};
pub use text::Text;
pub use text_edit::{Formatter, TextEdit, TextInput, ValidationResult};
pub use thumb::{DragController, Thumb};
pub use titled_pane::TitledPane;

pub use kyute_macros::WidgetWrapper;

use crate::{
    animation::LayerHandle, BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, PaintCtx, Widget,
    WidgetId,
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

    fn layer(&self) -> &LayerHandle {
        self.inner().layer()
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
}

impl<T: WidgetWrapper> Widget for T {
    fn widget_id(&self) -> Option<WidgetId> {
        WidgetWrapper::widget_id(self)
    }

    fn layer(&self) -> &LayerHandle {
        WidgetWrapper::layer(self)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        WidgetWrapper::layout(self, ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        WidgetWrapper::event(self, ctx, event, env)
    }

    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        WidgetWrapper::route_event(self, ctx, event, env)
    }
}

/// Prelude containing commonly used items for writing custom widgets.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        animation::{Layer, LayerDelegate, LayerHandle},
        cache::Signal,
        composable, Alignment, BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, Offset,
        Orientation, PaintCtx, Point, Rect, Size, Transform, Widget, WidgetId, WidgetPod,
    };
}
