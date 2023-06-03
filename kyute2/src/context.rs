use crate::{app_state::AppHandle, window::WindowFocusState, WidgetId};
use glazier::WindowHandle;
use kurbo::Affine;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TreeCtx {
    pub(crate) app_handle: AppHandle,
}

impl TreeCtx {
    pub fn new(app_handle: AppHandle) -> TreeCtx {
        TreeCtx { app_handle }
    }

    /// Requests a relayout of the node.
    pub fn relayout(&mut self) {
        todo!()
    }

    /// Requests a repaint of the node;
    pub fn repaint(&mut self) {
        todo!()
    }

    /// The ID of the node.
    pub fn id(&self) -> WidgetId {
        todo!()
    }

    /// Call to signal that a child widget has been removed.
    pub fn child_removed(&mut self, child_index: usize) {
        todo!()
    }

    /// Call to signal that a child widget is being added.
    pub fn child_added(&mut self) -> WidgetId {
        todo!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Event propagation context.
pub struct EventCtx<'a> {
    /// Parent window handle.
    window: &'a mut WindowHandle,

    /// Focus state of the parent window.
    pub(crate) window_state: &'a mut WindowFocusState,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<WidgetId>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Layout context.
pub struct LayoutCtx<'a> {
    /// Parent window handle.
    window: WindowHandle,

    /// Focus state of the parent window.
    pub(crate) window_state: &'a mut WindowFocusState,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<WidgetId>,
}

impl<'a> LayoutCtx<'a> {
    pub(crate) fn new(window: WindowHandle, window_state: &'a mut WindowFocusState) -> LayoutCtx {
        LayoutCtx {
            window,
            window_state,
            window_transform: Default::default(),
            id: None,
        }
    }
}
