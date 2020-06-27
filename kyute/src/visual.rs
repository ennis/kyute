use crate::event::Event;
use crate::{Rect, Environment, EventCtx, PaintCtx, Point};
use std::any::Any;
use crate::node::NodeTree;
use generational_indextree::NodeId;
use winit::event::WindowEvent;
use winit::window::WindowId;
use crate::application::AppCtx;
use kyute_shell::window::PlatformWindow;


pub trait WindowHandler {
    /// Returns the window
    fn window(&self) -> &PlatformWindow;

    /// Returns the window
    fn window_mut(&mut self) -> &mut PlatformWindow;

    /// Handles a raw window event.
    ///
    /// This is called only if the node has been registered as a window.
    /// Returns the translated events to dispatch to this node (and the rest of the children)
    /// afterwards.
    fn window_event(&mut self, _ctx: &mut AppCtx, _window_event: &WindowEvent, _tree: &mut NodeTree, _anchor: NodeId) {
    }

    /// Paints a subtree into the window.
    ///
    /// This is called only if the node has been registered as a window.
    /// This method is responsible for creating a drawing context and passing it down to the nodes.
    fn window_paint(&mut self,
                    _ctx: &mut AppCtx, _tree: &mut NodeTree, _anchor: NodeId) {
    }
}

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
    fn hit_test(&mut self, point: Point, bounds: Rect) -> bool;

    /// Handles an event that targets this visual.
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event);

    /// as_any for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns a reference to the window handler object if this visual corresponds to a platform window.
    fn window_handler(&self) -> Option<&dyn WindowHandler> { None }

    /// Returns a reference to the window handler object if this visual corresponds to a platform window.
    fn window_handler_mut(&mut self) -> Option<&mut dyn WindowHandler> { None }
}

impl dyn Visual {
    /// Downcasts a `Visual` trait object to a concrete type.
    pub fn downcast<V: Visual>(self: Box<dyn Visual>) -> Result<Box<V>, Box<dyn Visual>> {
        if self.as_any().is::<V>() {
            unsafe {
                // SAFETY: see Box::<dyn Any>::downcast in std
                let raw: *mut dyn Visual = Box::into_raw(self);
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
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {}
    fn hit_test(&mut self, _point: Point, _bounds: Rect) -> bool {
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
    fn paint(&mut self, _ctx: &mut PaintCtx, env: &Environment) {}
    fn hit_test(&mut self, _point: Point, _bounds: Rect) -> bool {
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
