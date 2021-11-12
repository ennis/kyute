//! Contains event propagation logic for [`NodeTrees`](crate::node::NodeTree).
use crate::{
    application::AppCtx,
    component::Action,
    event::{
        Event, InputState, MoveFocusDirection, PointerButtonEvent, PointerButtons, PointerEvent,
    },
    node::NodeTree,
    Point, Rect,
};
use generational_indextree::NodeId;
use kyute_shell::{platform::Platform, window::PlatformWindow};
use log::trace;
use std::collections::HashMap;
use winit::{
    event::{DeviceId, ModifiersState},
    window::WindowId,
};
use crate::node::NodeId;

/// Global state related to focus and pointer grab.
pub struct FocusState {
    pub(crate) focus: Option<NodeId>,
    pub(crate) pointer_grab: Option<NodeId>,
    pub(crate) hot: Option<NodeId>,
}

impl FocusState {
    pub fn new() -> FocusState {
        FocusState {
            focus: None,
            pointer_grab: None,
            hot: None,
        }
    }
}

impl Default for FocusState {
    fn default() -> Self {
        FocusState::new()
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RepaintRequest {
    /// Do nothing
    None,
    /// Repaint the widgets
    Repaint,
    /// Relayout and repaint the widgets
    Relayout,
}

/// The result of event delivery.
pub(crate) struct DispatchResult {
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

/// Context passed to [`Visual::event`] during event propagation.
/// Also serves as a return value for this function.
pub struct EventCtx<'a> {
    /// Window context
    pub(crate) app_ctx: &'a mut AppCtx,
    /// Window
    pub(crate) window: &'a PlatformWindow,
    /// State of various input devices.
    pub(crate) inputs: &'a InputState,
    /// Contains information about currently focused and pointer-grabbing nodes.
    pub(crate) focus: &'a mut FocusState,
    /// The ID of the current node.
    pub(crate) node_id: NodeId,
    /// The bounds of the current visual.
    pub(crate) bounds: Rect,
    /// Focus change requested
    pub(crate) focus_change: FocusChange,
    /// Redraw requested
    pub(crate) repaint: RepaintRequest,
    /// Pointer grab requested
    pub(crate) pointer_capture: bool,
    /// Event handled
    pub(crate) handled: bool,
    // Whether this is a focus change event
    //in_focus_event: bool,
}

impl<'a> EventCtx<'a> {
    pub fn platform(&self) -> &Platform {
        &self.app_ctx.platform
    }

    /// Returns the bounds of the current widget.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Requests a redraw of the current visual.
    pub fn request_redraw(&mut self) {
        self.repaint = RepaintRequest::Repaint;
    }

    /// Requests that the current node grabs all pointer events.
    pub fn capture_pointer(&mut self) {
        trace!("capture_pointer: {}", self.node_id);
        self.handled = true;
        self.pointer_capture = true;
    }

    /// Returns whether the current node is capturing the pointer.
    pub fn is_capturing_pointer(&self) -> bool {
        self.focus.pointer_grab == Some(self.node_id)
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        if self.focus.pointer_grab == Some(self.node_id) {
            self.focus.pointer_grab = None;
        }
    }

    /// Acquires the focus.
    pub fn request_focus(&mut self) {
        //assert!(!self.in_focus_event, "cannot request focus in a focus handler");
        self.set_handled();
        self.focus_change = FocusChange::Acquire;
    }

    /// Returns whether the current node has the focus.
    pub fn has_focus(&self) -> bool {
        self.focus.focus == Some(self.node_id)
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.handled = true;
    }

    /// Returns the window that the event was originally sent to.
    pub fn window(&self) -> &PlatformWindow {
        self.window
    }

    #[must_use]
    pub fn handled(&self) -> bool {
        self.handled
    }

    pub(crate) fn push_action(&mut self, action: Action) {
        self.app_ctx.actions.push(action)
    }
}

impl NodeTree {
    /// Builds the dispatch chain for a pointer event, by recursively hit-testing the bounds of
    /// nodes.
    pub(crate) fn find_pointer_event_target(
        &self,
        id: NodeId,
        window_id: WindowId,
        window_pos: Point,
        origin: Point,
    ) -> Option<NodeId> {
        let node_data = &self.arena[id].get();

        // don't cross into other windows
        match node_data.window_id() {
            Some(id) if id != window_id => return None,
            _ => {}
        }

        let offset = node_data.offset;
        let measurements = node_data.measurements;
        // bounds in window coordinates
        let bounds = Rect::new(origin + offset, measurements.size);

        if bounds.contains(window_pos) {
            // TODO more precise hit test
            // recurse on children
            let mut child_id = self.arena[id].first_child();
            while let Some(id) = child_id {
                if let Some(target_id) =
                    self.find_pointer_event_target(id, window_id, window_pos, bounds.origin)
                {
                    // hit
                    return Some(target_id);
                }
                // no hit, continue
                child_id = self.arena[id].next_sibling();
            }
            Some(id)
        } else {
            None
        }
    }

    /// Builds the dispatch chain followed by an event in the visual tree, or empty vec if it's a traversal.
    pub(crate) fn find_event_target(
        &self,
        root: NodeId,
        window_id: WindowId,
        focus: &FocusState,
        event: &Event,
    ) -> Option<NodeId> {
        match event {
            Event::PointerMove(pointer_event)
            | Event::PointerDown(PointerButtonEvent {
                pointer: pointer_event,
                ..
            })
            | Event::PointerUp(PointerButtonEvent {
                pointer: pointer_event,
                ..
            }) => {
                // we are delivering a pointer event.
                // if there is a pointer-capturing node, then deliver the event directly to it
                if let Some(pointer_capture_node_id) = focus.pointer_grab {
                    Some(pointer_capture_node_id)
                } else {
                    // otherwise, build a pointer dispatch chain
                    self.find_pointer_event_target(
                        root,
                        window_id,
                        pointer_event.window_position,
                        self.window_origin,
                    )
                }
            }
            Event::KeyUp(keyboard_event) | Event::KeyDown(keyboard_event) => {
                // keyboard events are delivered to the currently focused node
                if let Some(focused_node_id) = focus.focus {
                    Some(focused_node_id)
                } else {
                    None
                }
            }
            Event::Input(input_event) => {
                // same as keyboard events
                if let Some(focused_node_id) = focus.focus {
                    Some(focused_node_id)
                } else {
                    None
                }
            }
            Event::Wheel(wheel_event) => {
                // wheel events always follow a pointer dispatch chain, regardless of whether there
                // is a pointer grab or not
                self.find_pointer_event_target(
                    self.root,
                    window_id,
                    wheel_event.pointer.window_position,
                    self.window_origin,
                )
            }
            // default is standard traversal
            _ => None,
        }
    }

    /// Returns a copy of the event with all local coordinates re-calculated relative to the specified target node.
    pub(crate) fn build_local_event(&self, event: &Event, target: NodeId) -> Event {
        let node_window_pos = self.arena[target].get().window_pos.get();
        let mut event = *event;
        match event {
            Event::PointerUp(PointerButtonEvent {
                pointer: ref mut p, ..
            })
            | Event::PointerDown(PointerButtonEvent {
                pointer: ref mut p, ..
            })
            | Event::PointerMove(ref mut p) => {
                p.position = p.window_position - node_window_pos.to_vector();
            }
            _ => {}
        }
        event
    }

    /// Sends an event to a target node and optionally bubble up.
    pub(crate) fn dispatch_event(
        &mut self,
        window_ctx: &mut AppCtx,
        window: &PlatformWindow,
        root: NodeId,
        inputs: &InputState,
        focus: &mut FocusState,
        event: &Event,
        target: NodeId,
        repaint: &mut RepaintRequest,
        bubble: bool,
    ) -> Option<NodeId> {
        let mut next_id = Some(target);
        let mut handled_by = None;

        while let Some(id) = next_id {
            let local_event = self.build_local_event(event, id);
            // deliver event to visual
            let node = &mut self.arena[id];

            let mut ctx = EventCtx {
                app_ctx: window_ctx,
                window,
                inputs,
                focus,
                node_id: id,
                bounds: Rect::new(Point::origin(), node.get().measurements.size),
                focus_change: FocusChange::Keep,
                repaint: RepaintRequest::None,
                pointer_capture: false,
                handled: false,
            };

            // the node might not have a visual if it has been "temporarily moved out the tree"
            // for borrowing reasons: this happens within `Visual::window_paint` and `Visual::window_event`.
            node.get_mut()
                .visual
                .as_mut()
                .map(|v| v.event(&mut ctx, &local_event));

            *repaint = (*repaint).max(ctx.repaint);
            let focus_change = ctx.focus_change;
            let handled = ctx.handled;
            let pointer_capture = ctx.pointer_capture;

            // after delivering the event, immediately process the focus and pointer-capture related
            // events that must be sent.
            match focus_change {
                FocusChange::Acquire => {
                    let old_focus = focus.focus;
                    if old_focus != Some(id) {
                        if let Some(old_focus) = old_focus {
                            let r = self.dispatch_event(
                                window_ctx,
                                window,
                                root,
                                inputs,
                                focus,
                                &Event::FocusOut,
                                old_focus,
                                repaint,
                                true,
                            );
                        }

                        focus.focus = Some(id);
                        self.dispatch_event(
                            window_ctx,
                            window,
                            root,
                            inputs,
                            focus,
                            &Event::FocusIn,
                            id,
                            repaint,
                            true,
                        );
                    }
                }
                FocusChange::Release => {
                    if focus.focus == Some(id) {
                        self.dispatch_event(
                            window_ctx,
                            window,
                            root,
                            inputs,
                            focus,
                            &Event::FocusOut,
                            id,
                            repaint,
                            true,
                        );
                        focus.focus = None;
                    }
                }
                FocusChange::Move(_) => todo!("tab navigation"),
                FocusChange::Keep => {}
            }

            // handle pointer capture requests
            if pointer_capture {
                // TODO events?
                focus.pointer_grab = Some(id);
            }

            // stop propagation if the event was handled
            if handled {
                handled_by = Some(id);
            }

            if !bubble || handled {
                break;
            }

            // FIXME we could bubble to a parent window, is that what we want?
            next_id = self.arena[id].parent();
        }

        handled_by
    }

    /// Delivers an event to a subtree.
    ///
    /// Parameters:
    /// - window: the window corresponding to the subtree
    /// - root: the root node of the window subtree (contents of the window)
    /// - inputs: tracked input state (keyboard modifiers and pointers)
    /// - focus: focus state of the window. This should **only** be changed by this function,
    /// or otherwise widgets may not be notified of a focus change.
    /// - event: event to be delivered
    pub fn event(
        &mut self,
        window_ctx: &mut AppCtx,
        window: &PlatformWindow,
        root: NodeId,
        inputs: &InputState,
        focus: &mut FocusState,
        event: &Event,
    ) -> RepaintRequest {
        //trace!("event {:?}", event);

        // find the target of the event
        let target = self.find_event_target(root, window.id(), focus, event);
        let mut repaint = RepaintRequest::None;

        // event pre-processing
        match event {
            Event::PointerUp(PointerButtonEvent { pointer: p, .. })
            | Event::PointerDown(PointerButtonEvent { pointer: p, .. })
            | Event::PointerMove(p) => {
                // sending a pointer event, and not to the currently hot target, this means that
                // the hot widget is changing
                if focus.hot != target {
                    // handle pointerout/pointerover
                    if let Some(old_and_busted) = focus.hot {
                        // send a pointerout event for the widget losing the hot status
                        self.dispatch_event(
                            window_ctx,
                            window,
                            root,
                            inputs,
                            focus,
                            &Event::PointerOut(*p),
                            old_and_busted,
                            &mut repaint,
                            true,
                        );
                    }
                    if let Some(new_hotness) = target {
                        // set a pointerover event for the widget gaining the hot status
                        self.dispatch_event(
                            window_ctx,
                            window,
                            root,
                            inputs,
                            focus,
                            &Event::PointerOver(*p),
                            new_hotness,
                            &mut repaint,
                            true,
                        );
                        focus.hot.replace(new_hotness);
                    }
                }
            }
            _ => {}
        }

        if let Some(target) = target {
            // dispatch the event if we have determined a target
            self.dispatch_event(
                window_ctx,
                window,
                root,
                inputs,
                focus,
                event,
                target,
                &mut repaint,
                true,
            );
        }

        // post-processing
        match event {
            Event::PointerUp(p) => {
                // automatic release of pointer capture
                if p.pointer.buttons.is_empty() {
                    trace!("auto pointer release");
                    focus.pointer_grab = None;
                }
            }
            _ => {}
        }

        // TODO Tab navigation
        repaint
    }
}
