use crate::event::Event;
use crate::{Bounds, EventCtx, Point, PaintCtx, Environment};
use std::any::Any;

/// The interface for painting a visual element on the screen, and handling events that target this
/// visual.
///
/// [`Visual`]s are typically wrapped in a [`Node`], which bundles the visual and the layout
/// information of the visual within a parent object.
pub trait Visual: Any {
    /// Draws the visual using the specified painter.
    ///
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment);

    /// Checks if the given point falls inside the widget.
    ///
    /// Usually it's a simple matter of checking whether the point falls in the provided bounds,
    /// but some widgets may want a more complex hit test.
    ///
    /// TODO remove this method, it's not used
    fn hit_test(&mut self, point: Point, bounds: Bounds) -> bool;

    /// Handles an event that targets this visual, and returns the _actions_ emitted in response
    /// to this event.
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event);

    /// as_any for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl dyn Visual {
    /// Downcasts a `Visual` trait object to a concrete type.
    pub fn downcast<V: Visual>(self: Box<dyn Visual>) -> Result<Box<V>, Box<dyn Visual>> {
        if self.as_any().is::<V>() {
            unsafe {
                // SAFETY: see Box::<dyn Any>::downcast in std
                let raw: *mut dyn Any = Box::into_raw(self);
                Ok(Box::from_raw(raw as *mut V))
            }
        } else {
            Err(self)
        }
    }
}

/// A visual that has no particular behavior, used for layout wrappers.
pub struct LayoutBox;

impl Default for LayoutBox {
    fn default() -> Self {
        LayoutBox
    }
}

impl Visual for LayoutBox {
    fn paint(&mut self, ctx: &mut PaintCtx) {}
    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        true
    }
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A visual that has no particular behavior.
#[derive(Copy, Clone, Debug, Default)]
pub struct DummyVisual;

impl Visual for DummyVisual {
    fn paint(&mut self, _ctx: &mut PaintCtx) {}
    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }
    fn event(&mut self, _event_ctx: &mut EventCtx, _event: &Event) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
