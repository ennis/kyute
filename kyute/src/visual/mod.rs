//! Elements of the visual tree (after layout): `Visual`s and `Node`s.
use crate::event::{Event, MoveFocusDirection, PointerButtons, PointerEvent};
use crate::layout::{Layout, Offset, Point};
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
pub struct LayoutBox {
    pub inner: Box<Node>,
}

impl LayoutBox {
    pub fn new(inner: Box<Node>) -> LayoutBox {
        LayoutBox { inner }
    }
}

impl Default for LayoutBox {
    fn default() -> Self {
        LayoutBox {
            inner: Node::dummy(),
        }
    }
}

impl Visual for LayoutBox {
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
#[derive(Copy, Clone, Debug, Default)]
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
pub struct Node<V: ?Sized = dyn Visual> {
    /// Layout of the node relative to the containing window.
    pub(crate) layout: Layout,
    /// Key associated to the node.
    pub key: Option<u64>,
    /// Node state related to event flow
    pub(crate) state: NodeState,
    /// The visual. Defines the painting, hit-testing, and event behaviors.
    /// The visual instance is set up by the [widget] during [layout](Widget::layout).
    pub visual: V,
}

impl<V> Node<V> {
    /// Creates a new node from a layout and a visual.
    pub fn new(layout: Layout, key: Option<u64>, visual: V) -> Node<V> {
        Node {
            // A dummy type is specified here because Weak::new() has a Sized bound on T.
            // See discussion at https://users.rust-lang.org/t/why-cant-weak-new-be-used-with-a-trait-object/29976
            // also see issue https://github.com/rust-lang/rust/issues/50513
            // and https://github.com/rust-lang/rust/issues/60728
            key,
            layout,
            state: NodeState::default(),
            visual,
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

/// Global state related to focus and pointer grab.
pub struct FocusState {
    focus_marker: u32,
    pointer_grab_marker: u32,
    /// Whether a node has focus and should receive keyboard events
    has_focus: bool,
    /// Whether a node is grabbing pointer events
    has_pointer_grab: bool,
}

impl FocusState {
    pub(crate) fn new() -> FocusState {
        FocusState {
            focus_marker: 1,
            pointer_grab_marker: 1,
            has_focus: false,
            has_pointer_grab: false,
        }
    }
}

impl FocusState {
    pub(crate) fn release_focus(&mut self) {
        // this immediately invalidates any existing focus path
        self.focus_marker += 1;
        self.has_focus = false;
        trace!(
            "release_focus {} -> {}",
            self.focus_marker - 1,
            self.focus_marker
        );
    }

    pub(crate) fn release_pointer_grab(&mut self) {
        // this immediately invalidates any existing pointer grab
        self.pointer_grab_marker += 1;
        self.has_pointer_grab = false;
        trace!(
            "release_pointer_grab {} -> {}",
            self.pointer_grab_marker - 1,
            self.pointer_grab_marker
        );
    }

    pub(crate) fn acquire_focus(&mut self) -> u32 {
        self.focus_marker += 1;
        self.has_focus = true;
        trace!(
            "acquire_focus {} -> {}",
            self.focus_marker - 1,
            self.focus_marker
        );
        self.focus_marker
    }

    pub(crate) fn acquire_pointer_grab(&mut self) -> u32 {
        self.pointer_grab_marker += 1;
        self.has_pointer_grab = true;
        trace!(
            "acquire_pointer_grab {} -> {}",
            self.pointer_grab_marker - 1,
            self.pointer_grab_marker
        );
        self.pointer_grab_marker
    }
}

pub(crate) struct NodeState {
    /// node is on the delivery path to the focus node
    pub(crate) on_focus_path_marker: u32,
    /// node is focused
    pub(crate) focus_marker: u32,
    /// node is on the delivery path to the pointer grabbing node
    pub(crate) on_pointer_grab_path_marker: u32,
    /// node is grabbing the pointer
    pub(crate) pointer_grab_marker: u32,
}

impl Default for NodeState {
    fn default() -> Self {
        NodeState {
            on_focus_path_marker: 0,
            focus_marker: 0,
            on_pointer_grab_path_marker: 0,
            pointer_grab_marker: 0,
        }
    }
}

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FocusChange {
    /// Keep the focus, or do nothing if the node does not have it.
    Keep,
    /// Acquire focus, if the node does not have it already
    Acquire,
    /// Release the focus, if the node has it
    Release,
    /// Move the focus.
    Move(MoveFocusDirection),
}

/// Event traversal state (shared)
pub struct EventFlowCtx<'a> {
    /// State of various input devices.
    input_state: &'a InputState,
    /// Contains information about currently focused and pointer-grabbing nodes.
    focus: &'a mut FocusState,
    /// The node is on the event delivery path towards the focused node,
    /// and should update its markers
    mark_focus_path: bool,
    /// The node is on the event delivery path towards the pointer-grabbing node,
    /// and should update its markers
    mark_pointer_grab_path: bool,
    /// The event was handled.
    handled: bool,
    /// Redraw requested
    redraw_requested: bool,
}

impl<'a> EventFlowCtx<'a> {
    /// Returns whether we should follow a pointer-capture flow.
    fn is_capturing_pointer(&self) -> bool {
        self.focus.has_pointer_grab
    }

    fn is_capturing_keyboard(&self) -> bool {
        self.focus.has_focus
    }
}

/// Context passed to [`Visual::event`] during event propagation.
/// Also serves as a return value for this function.
pub struct EventCtx<'a, 'b> {
    flow: &'b mut EventFlowCtx<'a>,
    /// Event-related states of the node owning the visual.
    node_state: &'b mut NodeState,
    /// The bounds of the current visual.
    bounds: Bounds,
    /// The last node that got this context wants the pointer grab
    pointer_grab_requested: bool,
    /// The last node that got this context wants the focus
    focus_change_requested: FocusChange,
}

impl<'a, 'b> EventCtx<'a, 'b> {
    fn new_child_ctx<'c>(
        &'c mut self,
        node_state: &'c mut NodeState,
        bounds: Bounds,
    ) -> EventCtx<'a, 'c>
    where
        'b: 'c,
    {
        EventCtx {
            flow: self.flow,
            node_state,
            bounds,
            pointer_grab_requested: false,
            focus_change_requested: FocusChange::Keep,
        }
    }

    /// Returns whether the node is currently focused.
    pub fn is_focused(&self) -> bool {
        self.node_state.focus_marker == self.flow.focus.focus_marker
    }

    /// Returns whether the node is currently grabbing the pointer.
    pub fn is_grabbing_pointer(&self) -> bool {
        self.node_state.pointer_grab_marker == self.flow.focus.pointer_grab_marker
    }

    /// Returns the bounds of the current widget.
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    /// Requests a redraw of the current visual.
    pub fn request_redraw(&mut self) {
        self.flow.redraw_requested = true;
    }

    /// Requests that the current node grabs all pointer events.
    pub fn capture_pointer(&mut self) {
        self.flow.handled = true;
        let marker = self.flow.focus.acquire_pointer_grab();
        self.node_state.pointer_grab_marker = marker;
        self.flow.mark_pointer_grab_path = true;
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        self.flow.focus.release_pointer_grab();
        self.flow.mark_pointer_grab_path = false;
    }

    /// Acquires the focus.
    pub fn acquire_focus(&mut self) {
        self.flow.handled = true;
        let marker = self.flow.focus.acquire_focus();
        self.node_state.focus_marker = marker;
        self.flow.mark_focus_path = true;
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.flow.handled = true;
    }

    #[must_use]
    pub fn handled(&self) -> bool {
        self.flow.handled
    }
}

/// Event handling methods.
impl<V: Visual + ?Sized> Node<V> {
    /// Performs hit-test of the specified [`PointerEvent`] on the node, then, if hit-test
    /// is successful, returns the [`PointerEvent`] mapped to local coordinates.
    ///
    /// [`PointerEvent`]: crate::event::PointerEvent
    fn translate_pointer_event(&self, pointer: &PointerEvent) -> PointerEvent {
        let bounds = Bounds::new(self.layout.offset.to_point(), self.layout.size);
        trace!(
            "pointer event in local coords: {}",
            pointer.position - bounds.origin.to_vector()
        );
        PointerEvent {
            position: pointer.position - bounds.origin.to_vector(),
            ..*pointer
        }
    }

    pub fn is_on_focus_path(&self, ctx: &EventCtx) -> bool {
        self.state.on_focus_path_marker == ctx.flow.focus.focus_marker
    }

    pub fn is_on_pointer_grab_path(&self, ctx: &EventCtx) -> bool {
        self.state.on_pointer_grab_path_marker == ctx.flow.focus.pointer_grab_marker
    }

    /// Processes an event.
    ///
    /// This function will determine if the event is of interest for the node, then send it to the
    /// child visual.
    ///
    /// See also: [`Visual::event`].
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        if ctx.flow.handled {
            // the event was already handled by a previous handler
            return;
        }

        // bounds of the current node, in the coordinate spa
        let bounds = Bounds::new(Point::origin(), self.layout.size);

        // Event to deliver to the wrapped visual.
        // It might not be the same because for pointer events the pointer position is
        // adjusted to the local coordinate space of the visual.
        let mut child_event = *event;

        let pointer_event = match event {
            Event::PointerUp(p) => Some(*p),
            Event::PointerDown(p) => Some(*p),
            Event::PointerMove(p) => Some(*p),
            e => None,
        };

        if let Some(mut p) = pointer_event {
            // translate pointer events to local coordinates
            p.position -= self.layout.offset;
            if !ctx.flow.is_capturing_pointer() {
                // not in a pointer capture flow, so perform hit-test
                if !bounds.contains(p.position) {
                    // hit-test failed
                    return;
                }
            } else {
                trace!(
                    "pointer capture flow: {}",
                    self.state.on_pointer_grab_path_marker
                );
                // we're in a pointer capture flow; if we're not in the pointer capture path,
                // then return;
                if !self.is_on_pointer_grab_path(ctx) {
                    return;
                }
            }

            // rewrap pointer event
            match event {
                Event::PointerUp(_) => child_event = Event::PointerUp(p),
                Event::PointerDown(_) => child_event = Event::PointerDown(p),
                Event::PointerMove(_) => child_event = Event::PointerMove(p),
                _ => {}
            };
        }

        {
            let mut child_ctx = ctx.new_child_ctx(
                &mut self.state,
                Bounds::new(self.layout.offset.to_point(), self.layout.size),
            );
            self.visual.event(&mut child_ctx, &child_event);

            if child_ctx.flow.handled {
                // handle possible focus changes
                match child_ctx.focus_change_requested {
                    FocusChange::Move(_) => {
                        // handled in a separate traversal
                        todo!("focus move")
                    }
                    _ => {}
                }
            }
            // drop child_ctx
        }

        // if the node or one of its descendants requested focus and/or pointer grab, update the
        // corresponding markers on the node state since this node is on the delivery path.
        if ctx.flow.mark_focus_path {
            self.state.on_focus_path_marker = ctx.flow.focus.focus_marker;
        }

        if ctx.flow.mark_pointer_grab_path {
            trace!("mark pointer grab {}", ctx.flow.focus.pointer_grab_marker);
            self.state.on_pointer_grab_path_marker = ctx.flow.focus.pointer_grab_marker;
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
        let mut flow = EventFlowCtx {
            input_state,
            focus: focus_state,
            mark_focus_path: false,
            mark_pointer_grab_path: false,
            handled: false,
            redraw_requested: false,
        };

        {
            let mut ctx = EventCtx {
                flow: &mut flow,
                node_state: &mut self.state,
                bounds: Default::default(),
                pointer_grab_requested: false,
                focus_change_requested: FocusChange::Keep,
            };

            self.visual.event(&mut ctx, event);
        }

        // FIXME duplicate of event()
        if flow.mark_focus_path {
            self.state.on_focus_path_marker = flow.focus.focus_marker;
        }
        if flow.mark_pointer_grab_path {
            self.state.on_pointer_grab_path_marker = flow.focus.pointer_grab_marker;
        }

        EventResult {
            handled: flow.handled,
            repaint: if flow.redraw_requested {
                RepaintRequest::Repaint
            } else {
                RepaintRequest::None
            },
        }
    }
}
