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
mod canvas;
mod popup;
mod scroll_area;
mod thumb;
mod titled_pane;
mod tree;

pub use align::Align;
pub use baseline::Baseline;
pub use button::Button;
pub use canvas::{Canvas, Viewport};
pub use clickable::Clickable;
pub use constrained::ConstrainedBox;
pub use container::Container;
pub use drop_down::DropDown;
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use grid::{Grid, GridLength, GridRow, GridSpan};
pub use image::Image;
pub use label::Label;
pub use layout_wrapper::{LayoutInspector, LayoutWrapper};
pub use menu::{Action, ContextMenu, Menu, MenuItem, Shortcut};
pub use null::Null;
pub use padding::Padding;
pub use popup::Popup;
pub use scroll_area::ScrollArea;
pub use separator::separator;
pub use slider::Slider;
pub use text::Text;
pub use text_edit::{Formatter, TextEdit, TextInput, ValidationResult};
pub use thumb::{DragController, Thumb};
pub use titled_pane::TitledPane;
pub use tree::{TreeGrid, TreeNode};

use crate::{
    BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, PaintCtx, Rect, Transform, Widget, WidgetId,
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

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner().layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.inner().paint(ctx, env);
    }
}

impl<T: WidgetWrapper> Widget for T {
    fn widget_id(&self) -> Option<WidgetId> {
        WidgetWrapper::widget_id(self)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        WidgetWrapper::event(self, ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        WidgetWrapper::layout(self, ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        WidgetWrapper::paint(self, ctx, env)
    }
}

/*
/// Widgets that have only one child and wish to defer to this child's `Widget` implementation, except for event handling.
pub trait Controller {
    fn child(&self) -> &dyn Widget;
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment);
}

impl<T: Controller> Widget for T {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        Controller::event(self, ctx, event, env)
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        self.child().layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.child().paint(ctx, bounds, env);
    }
}
*/

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        cache::Signal, composable, Alignment, BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements,
        Offset, Orientation, PaintCtx, Point, Rect, Size, Transform, Widget, WidgetId, WidgetPod,
    };
}
