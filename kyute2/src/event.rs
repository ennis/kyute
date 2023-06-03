use crate::WidgetId;
use kurbo::Point;
use std::collections::HashSet;

////////////////////////////////////////////////////////////////////////////////////////////////////

// glazier reexports
pub use glazier::{
    Code as KeyCode, KeyEvent, KeyState, MouseButton, MouseButtons, MouseEvent, PointerButton, PointerEvent,
    PointerType,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum InternalEvent<'a> {
    RouteEvent {
        target: WidgetId,
        event: Box<Event<'a>>,
    },
    /// Contains a pointer event delivered directly to a target widget (because e.g. it captured
    /// the pointer).
    RoutePointerEvent {
        target: WidgetId,
        event: PointerEvent,
    },
    RouteRedrawRequest(WidgetId),
    HitTest {
        position: Point,
        hovered: &'a mut HashSet<WidgetId>,
        hot: &'a mut Option<WidgetId>,
    },
    /*DumpTree {
        nodes: &'a mut Vec<DebugWidgetTreeNode>,
    },*/
}

/// Events.
#[derive(Debug)]
pub enum Event<'a> {
    FocusGained,
    FocusLost,
    MenuCommand(usize),
    PointerMove(PointerEvent),
    PointerUp(PointerEvent),
    PointerDown(PointerEvent),
    /// A keyboard event.
    Keyboard(KeyEvent),
    Internal(InternalEvent<'a>),
}
