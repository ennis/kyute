//! Elements of the visual tree (after layout): `Visual`s and `Node`s.
use crate::event::{Event, PointerButtons, PointerEvent};
use crate::layout::{Layout, Offset, PaintLayout, Point};
use crate::renderer::Theme;
use crate::state::NodeKey;
use crate::{Bounds, BoxConstraints, Widget};
use euclid::{Point2D, UnknownUnit};
use log::trace;
use std::any;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::ops::{Deref, Range};
use std::rc::{Rc, Weak};

use crate::application::WindowCtx;
use crate::widget::ActionSink;
use kyute_shell::drawing::{Size, Transform};
use kyute_shell::window::DrawContext;
use std::collections::HashMap;
use std::ops::DerefMut;
use winit::event::DeviceId;

pub mod reconciliation;

pub struct PointerState {
    pub(crate) buttons: PointerButtons,
    pub(crate) position: Point,
}

impl Default for PointerState {
    fn default() -> Self {
        PointerState {
            buttons: PointerButtons(0),
            position: Point::origin(),
        }
    }
}

/// Last known state of various input devices.
pub struct InputState {
    /// Current state of keyboard modifiers.
    pub(crate) mods: winit::event::ModifiersState,
    /// Current state of pointers.
    pub(crate) pointers: HashMap<DeviceId, PointerState>,
}

pub struct FocusState {}

pub(crate) enum RepaintRequest {
    /// Do nothing
    None,
    /// Repaint the widgets
    Repaint,
    /// Relayout and repaint the widgets
    Relayout,
}

/// The result of event delivery.
pub(crate) struct EventResult {
    /// The event was handled.
    pub(crate) handled: bool,
    /// Whether repaint or relayout was requested.
    pub(crate) repaint: RepaintRequest,
}

/// Context passed to [`Visual::event`] during event propagation.
/// Also serves as a return value for this function.
pub struct EventCtx<'a> {
    pub(crate) input_state: &'a InputState,
    /// Contains the currently focused widgets.
    pub(crate) focus_state: &'a mut FocusState,
    /// The bounds of the current visual.
    pub(crate) bounds: Bounds,
    /// A redraw has been requested.
    pub(crate) redraw_requested: bool,
    /// The passed event was handled.
    pub(crate) handled: bool,
    /// The current node has asked to get the pointer grab
    pub(crate) pointer_capture: bool,
}

impl<'a> EventCtx<'a> {
    /*pub(crate) fn new(input_state: &'a InputState, focus_state: &'a mut FocusState) -> EventCtx<'a> {
        EventCtx {
            input_state,
            focus_state,
            bounds: Bounds::default(),
            redraw_requested: false,
            handled: false,
            pointer_capture: false,
        }
    }*/

    fn make_child_ctx(&'a mut self, bounds: Bounds) -> EventCtx<'a> {
        EventCtx {
            input_state: self.input_state,
            focus_state: self.focus_state,
            bounds,
            handled: false,
            pointer_capture: false,
            redraw_requested: false,
        }
    }

    /// Returns the bounds of the current widget.
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    /// Requests a redraw of the current visual.
    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    /// Requests that the current node grabs all pointer events.
    pub fn capture_pointer(&mut self) {
        self.pointer_capture = true;
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        self.pointer_capture = false;
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.handled = true;
    }
}

/// Context passed to [`Visual::paint`].
pub struct PaintCtx<'a, 'b> {
    pub(crate) draw_ctx: &'a mut DrawContext<'b>,
    pub(crate) size: Size,
}

impl<'a, 'b> PaintCtx<'a, 'b> {
    /// Returns the bounds of the visual.
    pub fn bounds(&self) -> Bounds {
        Bounds::new(Point::origin(), self.size)
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

impl<'a, 'b> Deref for PaintCtx<'a, 'b> {
    type Target = DrawContext<'b>;

    fn deref(&self) -> &Self::Target {
        self.draw_ctx
    }
}

impl<'a, 'b> DerefMut for PaintCtx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.draw_ctx
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
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme);

    /// Checks if the given point falls inside the widget.
    ///
    /// Usually it's a simple matter of checking whether the point falls in the provided bounds,
    /// but some widgets may want a more complex hit test.
    fn hit_test(&mut self, point: Point, bounds: Bounds) -> bool;

    /// Handles an event that targets this visual, and returns the _actions_ emitted in response
    /// to this event.
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event);

    /// as_any for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A visual that has no particular behavior and one child, used for layout wrappers.
pub struct LayoutBox<V> {
    pub inner: Node<V>,
}

impl<V: Visual> LayoutBox<V> {
    pub fn new(inner: Node<V>) -> LayoutBox<V> {
        LayoutBox { inner }
    }
}

impl<V: Visual> Visual for LayoutBox<V> {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        self.inner.paint(ctx, theme)
    }
    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        true
    }
    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        self.inner.event(ctx, event);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A visual that has no particular behavior.
pub struct DummyVisual;

impl Visual for DummyVisual {
    fn paint(&mut self, _ctx: &mut PaintCtx, _theme: &Theme) {}
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

/// A node within the visual tree.
///
/// It contains the bounds of the visual, and an instance of [`Visual`] that defines its behavior:
/// painting, hit-testing, and how it responds to events that target the visual.
pub struct Node<V: ?Sized> {
    /// Layout of the node relative to the containing window.
    pub layout: Layout,
    /// Key associated to the node.
    pub key: Option<u64>,
    /// The visual. Defines the painting, hit-testing, and event behaviors.
    /// The visual instance is set up by the [widget] during [layout](Widget::layout).
    pub visual: V,
}

impl<V: Visual> Node<V> {
    /// Creates a new node from a layout and a visual.
    pub fn new(layout: Layout, key: Option<u64>, visual: V) -> Node<V> {
        Node {
            // A dummy type is specified here because Weak::new() has a Sized bound on T.
            // See discussion at https://users.rust-lang.org/t/why-cant-weak-new-be-used-with-a-trait-object/29976
            // also see issue https://github.com/rust-lang/rust/issues/50513
            // and https://github.com/rust-lang/rust/issues/60728
            key,
            layout,
            visual,
        }
    }
}

/// Painting methods.
impl<V: Visual + ?Sized> Node<V> {
    /// Draws the node using the specified theme, in the specified context.
    ///
    /// Effectively, it applies the transform of the node (which, right now, is only an offset relative to the parent),
    /// and calls [`Visual::paint`] on `self.visual`.
    pub fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let mut ctx2 = PaintCtx {
            size: self.layout.size,
            draw_ctx: ctx.draw_ctx,
        };

        let saved = ctx2.draw_ctx.save();
        ctx2.draw_ctx.transform(&self.layout.offset.to_transform());
        self.visual.paint(&mut ctx2, theme);
        ctx2.draw_ctx.restore();
    }
}

/// Event handling methods.
impl<V: Visual + ?Sized> Node<V> {
    /// Performs hit-test of the specified [`PointerEvent`] on the node, then, if hit-test
    /// is successful, returns the [`PointerEvent`] mapped to local coordinates.
    ///
    /// [`PointerEvent`]: crate::event::PointerEvent
    fn translate_pointer_event(&self, pointer: &PointerEvent) -> Option<PointerEvent> {
        let bounds = Bounds::new(self.layout.offset.to_point(), self.layout.size);
        let hit = bounds.contains(pointer.position);

        if hit {
            Some(PointerEvent {
                position: pointer.window_position - bounds.origin.to_vector(),
                ..*pointer
            })
        } else {
            None
        }
    }

    /// Processes an event.
    ///
    /// This function will determine if the event is of interest for the node, then send it to the
    /// child visual.
    ///
    /// See also: [`Visual::event`].
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        if ctx.handled {
            // the event was already handled by a previous handler
            return;
        }

        let event = match event {
            Event::PointerUp(p) => self.translate_pointer_event(p).map(Event::PointerUp),
            Event::PointerDown(p) => self.translate_pointer_event(p).map(Event::PointerDown),
            Event::PointerMove(p) => self.translate_pointer_event(p).map(Event::PointerMove),
            e => Some(*e),
        };

        if let Some(ref event) = event {
            self.visual.event(ctx, event);
        }
    }

    ///
    pub(crate) fn propagate_event(
        &mut self,
        event: &Event,
        origin: Point,
        input_state: &InputState,
        focus_state: &mut FocusState,
    ) -> EventResult {
        let mut ctx = EventCtx {
            input_state,
            focus_state,
            bounds: Bounds::new(origin, self.layout.size),
            redraw_requested: false,
            handled: false,
            pointer_capture: false,
        };

        self.event(&mut ctx, event);

        EventResult {
            handled: ctx.handled,
            repaint: if ctx.redraw_requested {
                RepaintRequest::Repaint
            } else {
                RepaintRequest::None
            },
        }
    }
}

impl Node<dyn Visual> {
    /// Downcasts this node to a concrete type.
    pub fn downcast_mut<V: Visual>(&mut self) -> Option<&mut Node<V>> {
        if self.visual.as_any().is::<V>() {
            // SAFETY: see <dyn Any>::downcast_mut in std
            // TODO: this may be somewhat different since it's a DST?
            unsafe { Some(&mut *(self as *mut Self as *mut Node<V>)) }
        } else {
            None
        }
    }

    pub fn dummy() -> Box<Node<dyn Visual>> {
        Box::new(Node::new(Layout::default(), None, DummyVisual))
    }
}

impl<V> Default for Node<V>
where
    V: Visual + Default,
{
    fn default() -> Self {
        Node::new(Layout::default(), None, V::default())
    }
}
