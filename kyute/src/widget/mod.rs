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
mod popup;
mod titled_pane;

pub use align::Align;
pub use baseline::Baseline;
pub use button::Button;
pub use clickable::Clickable;
pub use constrained::ConstrainedBox;
pub use container::Container;
pub use drop_down::DropDown;
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use grid::{Grid, GridLength};
pub use image::Image;
pub use label::Label;
pub use layout_wrapper::LayoutWrapper;
pub use menu::{Action, Menu, MenuItem, Shortcut};
pub use null::Null;
pub use padding::Padding;
pub use popup::Popup;
pub use separator::separator;
pub use slider::Slider;
pub use text::Text;
pub use text_edit::{Formatter, TextEdit, TextInput, ValidationResult};
pub use titled_pane::TitledPane;

use crate::{BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, PaintCtx, Rect, Widget, WidgetId};

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
pub trait SingleChildWidget {
    fn child(&self) -> &dyn Widget;
}

impl<T: SingleChildWidget> Widget for T {
    fn widget_id(&self) -> Option<WidgetId> {
        self.child().widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.child().event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.child().layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.child().paint(ctx, bounds, env);
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
        composable, Alignment, BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, Offset,
        Orientation, PaintCtx, Point, Rect, Size, Widget, WidgetId, WidgetPod,
    };
}
