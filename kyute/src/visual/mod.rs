//! Elements of the visual tree (after layout): `Visual`s and `Node`s.
use crate::event::{Event, MoveFocusDirection, PointerButtons, PointerEvent};
use crate::layout::{Layout, Offset, Point};
use crate::renderer::Theme;
use crate::state::NodeKey;
use crate::{Bounds, BoxConstraints, Widget, BoxedWidget};
use euclid::{Point2D, UnknownUnit};
use log::trace;
use std::any;
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, Range};
use std::rc::{Rc, Weak};

use crate::application::WindowCtx;
use crate::widget::{ActionSink, LayoutCtx};
use generational_indextree::{Node, NodeEdge, NodeId};
use kyute_shell::drawing::{Size, Transform};
use kyute_shell::window::{DrawContext, PlatformWindow};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::CoerceUnsized;
use std::ops::DerefMut;
use winit::event::DeviceId;

mod reconciliation;
use std::cell::Cell;
pub use reconciliation::NodeCursor;

/// Context passed to [`Visual::paint`].
pub struct PaintCtx<'a, 'b> {
    pub(crate) draw_ctx: &'a mut DrawContext<'b>,
    pub(crate) size: Size,
    pub(crate) is_hot: bool,
}

impl<'a, 'b> PaintCtx<'a, 'b> {
    /// Returns the bounds of the visual.
    pub fn bounds(&self) -> Bounds {
        Bounds::new(Point::origin(), self.size)
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn is_hot(&self) -> bool {
        self.is_hot
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

/// A visual that has no particular behavior, used for layout wrappers.
pub struct LayoutBox;

impl Default for LayoutBox {
    fn default() -> Self {
        LayoutBox
    }
}

impl Visual for LayoutBox {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {}
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
pub struct NodeData<V: ?Sized = dyn Visual> {
    /// Layout of the node relative to the parent element.
    pub layout: Layout,
    /// Position of the node in window coordinates.
    pub window_pos: Cell<Point>,
    /// Key associated to the node.
    pub key: Option<u64>,
    /// The visual. Defines the painting, hit-testing, and event behaviors.
    /// The visual instance is set up by the [widget] during [layout](Widget::layout).
    pub visual: V,
}

impl<V> NodeData<V> {
    /// Creates a new node from a layout and a visual.
    pub fn new(layout: Layout, key: Option<u64>, visual: V) -> NodeData<V> {
        NodeData {
            // A dummy type is specified here because Weak::new() has a Sized bound on T.
            // See discussion at https://users.rust-lang.org/t/why-cant-weak-new-be-used-with-a-trait-object/29976
            // also see issue https://github.com/rust-lang/rust/issues/50513
            // and https://github.com/rust-lang/rust/issues/60728
            key,
            layout,
            window_pos: Cell::new(Point::origin()),
            visual,
        }
    }
}

impl NodeData<dyn Visual> {
    /// Downcasts this node to a concrete type.
    pub fn downcast_mut<V: Visual>(&mut self) -> Option<&mut NodeData<V>> {
        if self.visual.as_any().is::<V>() {
            // SAFETY: see <dyn Any>::downcast_mut in std
            // TODO: this may be somewhat different since it's a DST?
            unsafe { Some(&mut *(self as *mut Self as *mut NodeData<V>)) }
        } else {
            None
        }
    }

    pub fn dummy() -> Box<NodeData<dyn Visual>> {
        Box::new(NodeData::new(Layout::default(), None, DummyVisual))
    }
}

impl<V> Default for NodeData<V>
where
    V: Visual + Default,
{
    fn default() -> Self {
        NodeData::new(Layout::default(), None, V::default())
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

impl InputState {
    pub fn synthetic_pointer_event(&self, device_id: DeviceId) -> Option<PointerEvent> {
        self.pointers.get(&device_id).map(|state| PointerEvent {
            position: state.position,
            window_position: state.position,
            modifiers: self.mods,
            button: None,
            buttons: state.buttons,
            pointer_id: device_id,
        })
    }
}

/// Global state related to focus and pointer grab.
pub struct FocusState {
    focus: Option<NodeId>,
    pointer_grab: Option<NodeId>,
    hot: Option<NodeId>,
}

impl FocusState {
    pub(crate) fn new() -> FocusState {
        FocusState {
            focus: None,
            pointer_grab: None,
            hot: None,
        }
    }
}

impl FocusState {
    pub(crate) fn release_focus(&mut self) {
        self.focus = None;
    }

    pub(crate) fn release_pointer_grab(&mut self) {
        self.pointer_grab = None;
    }

    pub(crate) fn acquire_focus(&mut self, node: NodeId) {
        self.focus = Some(node);
    }

    pub(crate) fn acquire_pointer_grab(&mut self, node: NodeId) {
        self.pointer_grab = Some(node);
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
pub struct EventFlowCtx<'fcx, 'wcx> {
    /// Window context
    window_ctx: &'fcx mut WindowCtx<'wcx>,
    /// Window
    window: &'fcx PlatformWindow,
    /// State of various input devices.
    input_state: &'fcx InputState,
    /// Contains information about currently focused and pointer-grabbing nodes.
    focus: &'fcx mut FocusState,
    /// The event was handled.
    handled: bool,
    /// Redraw requested
    repaint: RepaintRequest,
}

impl<'fcx, 'wcx> EventFlowCtx<'fcx, 'wcx> {
    /// Returns whether we should follow a pointer-capture flow.
    fn is_capturing_pointer(&self) -> bool {
        self.focus.pointer_grab.is_some()
    }

    fn is_capturing_keyboard(&self) -> bool {
        self.focus.focus.is_some()
    }
}

/// Context passed to [`Visual::event`] during event propagation.
/// Also serves as a return value for this function.
pub struct EventCtx<'a, 'fctx, 'wctx> {
    flow: &'a mut EventFlowCtx<'fctx, 'wctx>,
    /// The ID of the current node.
    node: NodeId,
    /// The bounds of the current visual.
    bounds: Bounds,
    /// The last node that got this context wants the pointer grab
    pointer_capture_requested: bool,
    /// The last node that got this context wants the focus
    focus_change_requested: FocusChange,
}

impl<'a, 'fctx, 'wctx> EventCtx<'a, 'fctx, 'wctx> {
    /// Returns the bounds of the current widget.
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    /// Requests a redraw of the current visual.
    pub fn request_redraw(&mut self) {
        self.flow.repaint = RepaintRequest::Repaint;
    }

    /// Requests that the current node grabs all pointer events.
    pub fn capture_pointer(&mut self) {
        self.flow.handled = true;
        self.flow.focus.pointer_grab = Some(self.node);
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        if self.flow.focus.pointer_grab == Some(self.node) {
            self.flow.focus.pointer_grab = None;
        }
    }

    /// Acquires the focus.
    pub fn acquire_focus(&mut self) {
        self.flow.handled = true;
        self.flow.focus.focus = Some(self.node);
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.flow.handled = true;
    }

    /// Returns the window that the event was originally sent to.
    pub fn window(&self) -> &PlatformWindow {
        self.flow.window
    }

    #[must_use]
    pub fn handled(&self) -> bool {
        self.flow.handled
    }
}

pub type NodeArena = generational_indextree::Arena<Box<NodeData>>;

/// A tree of visual nodes representing the user interface elements shown in a window.
///
/// Contrary to the widget tree, those nodes are retained (as much as possible) across data updates
/// and relayouts. It is incrementally updated by [widgets](crate::widget::Widget) during layout.
///
/// See also: [`Widget::layout`](crate::widget::Widget::layout).
pub struct NodeTree {
    nodes: NodeArena,
    root: NodeId,
}

impl NodeTree {
    /// Creates a new node tree containing a single root node.
    pub fn new() -> NodeTree {
        let mut nodes = NodeArena::new();
        let root = nodes.new_node(NodeData::dummy());
        NodeTree { nodes, root }
    }

    /// Given a widget, runs the layout pass that updates the visual nodes of this tree.
    pub(crate) fn layout<A>(
        &mut self,
        widget: BoxedWidget<A>,
        size: Size,
        root_constraints: &BoxConstraints,
        theme: &Theme,
        win_ctx: &mut WindowCtx,
        action_sink: Rc<dyn ActionSink<A>>)
    {
        let mut layout_ctx = LayoutCtx {
            win_ctx,
            action_sink
        };
        widget.layout_child(&mut layout_ctx, &mut self.nodes, self.root, &root_constraints, theme);
        self.nodes[self.root].get_mut().layout.size = size;
        self.propagate_root_layout(Point::origin(), size);
    }

    /// Propagates the root size and compute window positions.
    pub(crate) fn propagate_root_layout(&mut self, origin: Point, size: Size) {
        let mut stack = Vec::new();
        let mut current_origin = origin;
        for edge in self.root.traverse(&self.nodes) {
            match edge {
                NodeEdge::Start(id) => {
                    stack.push(current_origin);
                    let node = self.nodes[id].get();
                    current_origin += node.layout.offset;
                    node.window_pos.set(current_origin);
                }
                NodeEdge::End(id) => {
                    current_origin = stack.pop().expect("unbalanced traversal");
                }
            }
        }
    }

    /// Builds the dispatch chain for a pointer event.
    pub(crate) fn build_pointer_dispatch_chain(
        &self,
        id: NodeId,
        window_pos: Point,
        dispatch_chain: &mut Vec<NodeId>,
        origin: Point,
        input_state: &InputState,
        focus_state: &FocusState,
    ) -> bool {
        let layout = &self.nodes[id].get().layout;
        // bounds in window coordinates
        let bounds = Bounds::new(origin + layout.offset, layout.size);

        if bounds.contains(window_pos) {
            // TODO more precise hit test
            dispatch_chain.push(id);
            // recurse on children
            let mut child_id = self.nodes[id].first_child();
            while let Some(id) = child_id {
                if self.build_pointer_dispatch_chain(
                    id,
                    window_pos,
                    dispatch_chain,
                    bounds.origin,
                    input_state,
                    focus_state,
                ) {
                    // hit
                    break;
                }
                // no hit, continue
                child_id = self.nodes[id].next_sibling();
            }
            true
        } else {
            false
        }
    }

    /// Builds a dispatch chain for a target node.
    pub(crate) fn build_target_dispatch_chain(&self, target: NodeId) -> Vec<NodeId> {
        let mut dispatch_chain = Vec::new();
        let mut next_id = Some(target);
        while let Some(id) = next_id {
            dispatch_chain.push(id);
            next_id = self.nodes[id].parent();
        }
        dispatch_chain.reverse();
        dispatch_chain
    }

    /// Builds the dispatch chain followed by an event in the visual tree, or empty vec if it's a traversal.
    pub(crate) fn build_dispatch_chain(
        &self,
        event: &Event,
        origin: Point,
        input_state: &InputState,
        focus_state: &FocusState,
    ) -> Vec<NodeId> {
        match event {
            Event::PointerMove(pointer_event)
            | Event::PointerDown(pointer_event)
            | Event::PointerUp(pointer_event) => {
                // if there is a pointer-capturing node, then deliver the event directly to it
                if let Some(pointer_capture_node_id) = focus_state.pointer_grab {
                    self.build_target_dispatch_chain(pointer_capture_node_id)
                } else {
                    // otherwise, build a pointer dispatch chain
                    let mut dispatch_chain = Vec::new();
                    self.build_pointer_dispatch_chain(
                        self.root,
                        pointer_event.window_position,
                        &mut dispatch_chain,
                        origin,
                        input_state,
                        focus_state,
                    );
                    dispatch_chain
                }
            }
            Event::KeyUp(keyboard_event) | Event::KeyDown(keyboard_event) => {
                // keyboard events are delivered to the currently focused node
                if let Some(focused_node_id) = focus_state.focus {
                    self.build_target_dispatch_chain(focused_node_id)
                } else {
                    // no node has focus, normal traversal
                    vec![]
                }
            }
            Event::Input(input_event) => {
                // same as keyboard events
                if let Some(focused_node_id) = focus_state.focus {
                    self.build_target_dispatch_chain(focused_node_id)
                } else {
                    vec![]
                }
            }
            Event::Wheel(wheel_event) => {
                // wheel events always follow a pointer dispatch chain, regardless of whether there
                // is a pointer grab or not
                let mut dispatch_chain = Vec::new();
                self.build_pointer_dispatch_chain(
                    self.root,
                    wheel_event.pointer.window_position,
                    &mut dispatch_chain,
                    origin,
                    input_state,
                    focus_state,
                );
                dispatch_chain
            }
            // default is standard traversal
            _ => vec![],
        }
    }

    /// Returns a copy of the event with all local coordinates re-calculated relative to the specified target node.
    pub(crate) fn build_local_event(&self, event: &Event, target: NodeId) -> Event {
        let node_window_pos = self.nodes[target].get().window_pos.get();
        let mut event = *event;
        match event {
            Event::PointerUp(ref mut p)
            | Event::PointerDown(ref mut p)
            | Event::PointerMove(ref mut p) => {
                p.position = p.window_position - node_window_pos.to_vector();
            }
            _ => {}
        }
        event
    }

    /// Sends a non-bubbling event to a target node
    pub(crate) fn send_event(&mut self, flow: &mut EventFlowCtx, event: &Event, target: NodeId) {
        let local_event = self.build_local_event(event, target);
        let mut ctx = EventCtx {
            flow,
            node: target,
            bounds: Bounds::default(),
            pointer_capture_requested: false,
            focus_change_requested: FocusChange::Keep,
        };
        // deliver event to visual
        self.nodes[target]
            .get_mut()
            .visual
            .event(&mut ctx, &local_event);
        trace!("send_event(target={:?}) {:?}", target, event);
    }

    pub(crate) fn dispatch_event(
        &mut self,
        flow: &mut EventFlowCtx,
        event: &Event,
        dispatch_chain: &[NodeId],
    ) {
        // deliver events to the dispatch chain, starting from the end (innermost element)
        // and bubble up
        // TODO capture phase?

        for &node_id in dispatch_chain.iter().rev() {
            self.send_event(flow, event, node_id);
            if flow.handled {
                // stop bubbling
                break;
            }
        }
    }

    pub(crate) fn event(
        &mut self,
        event: &Event,
        window_ctx: &mut WindowCtx,
        window: &PlatformWindow,
        origin: Point,
        input_state: &InputState,
        focus_state: &mut FocusState,
    ) -> EventResult {
        let dispatch_chain = self.build_dispatch_chain(event, origin, input_state, focus_state);

        let mut flow = EventFlowCtx {
            window,
            window_ctx,
            input_state,
            focus: focus_state,
            handled: false,
            repaint: RepaintRequest::None,
        };

        // event pre-processing
        match event {
            Event::PointerUp(p) | Event::PointerDown(p) | Event::PointerMove(p) => {
                // handle change of current "hot" element
                let target = dispatch_chain.last().cloned();
                if flow.focus.hot != target {
                    // dispatch a hover exit and hover enter event
                    if let Some(old_and_busted) = flow.focus.hot {
                        self.send_event(&mut flow, &Event::PointerLeave(*p), old_and_busted);
                    }
                    if let Some(new_hotness) = target {
                        self.send_event(&mut flow, &Event::PointerEnter(*p), new_hotness);
                        flow.focus.hot.replace(dbg!(new_hotness));
                    }
                }
            }
            _ => {}
        }

        // post-processing
        match event {
            Event::PointerUp(p) => {
                // automatic release of pointer capture
                if p.buttons.is_empty() {
                    trace!("auto pointer release");
                    flow.focus.pointer_grab = None;
                }
            }
            _ => {}
        }

        // TODO Tab navigation
        EventResult {
            handled: flow.handled,
            repaint: flow.repaint,
        }
    }

    /// Painting.
    pub fn paint(
        &mut self,
        draw_context: &mut DrawContext,
        focus_state: &FocusState,
        input_state: &InputState,
        theme: &Theme)
    {
        self.paint_node(draw_context, focus_state, input_state, self.root, theme)
    }

    /// Draws the node using the specified theme, in the specified context.
    ///
    /// Effectively, it applies the transform of the node (which, right now, is only an offset relative to the parent),
    /// and calls [`Visual::paint`] on `self.visual`.
    fn paint_node(&mut self,
                  draw_context: &mut DrawContext,
                  focus_state: &FocusState,
                  input_state: &InputState,
                  node_id: NodeId,
                  theme: &Theme)
    {
        let mut node = self.nodes[node_id].get_mut();

        let mut ctx = PaintCtx {
            draw_ctx: draw_context,
            size: node.layout.size,
            is_hot: focus_state.hot == Some(node_id)
        };

        trace!("id={}, size={}", node_id, node.layout.size);

        ctx.draw_ctx.save();
        ctx.draw_ctx.transform(&node.layout.offset.to_transform());
        node.visual.paint(&mut ctx, theme);

        // paint children
        let mut child_id = self.nodes[node_id].first_child();
        while let Some(id) = child_id {
            self.paint_node(ctx.draw_ctx, focus_state, input_state, id, theme);
            child_id = self.nodes[id].next_sibling();
        }

        ctx.draw_ctx.restore();
    }
}
