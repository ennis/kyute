use crate::{
    app_state::AppHandle, composition::DrawableSurface, drawing::ToSkia, window::WindowFocusState, ChangeFlags,
    Element, Environment, Event, Widget, WidgetId,
};
use glazier::{Scale, WindowHandle};
use kurbo::{Affine, Point};
use kyute_compose::CallId;
use skia_safe as sk;
use std::collections::HashMap;
use tracing::warn;

////////////////////////////////////////////////////////////////////////////////////////////////////

// Child -> Parent
pub type WidgetTree = HashMap<WidgetId, WidgetId>;

pub struct TreeCtx<'a> {
    pub(crate) app_handle: AppHandle,
    pub(crate) tree: &'a mut WidgetTree,
    current_id: WidgetId,
}

impl<'a> TreeCtx<'a> {
    pub(crate) fn new(app_handle: AppHandle, tree: &'a mut WidgetTree) -> TreeCtx {
        TreeCtx {
            app_handle,
            tree,
            current_id: WidgetId::from_call_id(CallId::DUMMY),
        }
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
    pub fn child_removed(&mut self, id: WidgetId) {
        self.tree.remove(&id);
    }

    /// Call to signal that a child widget is being added.
    pub fn child_added(&mut self, id: WidgetId) {
        let prev = self.tree.insert(id, self.current_id);
        if let Some(prev) = prev {
            warn!(
                "child_added called with id {:?} already in the tree (old parent: {:?}, new parent: {:?})",
                id, prev, self.current_id
            );
        }
    }

    pub fn build<W: Widget>(&mut self, widget: W, env: &Environment) -> W::Element {
        let id = widget.id();
        if id != self.current_id && id != WidgetId::ANONYMOUS {
            // build child with different ID
            self.child_added(id);
            let last_id = self.current_id;
            self.current_id = id;
            let r = widget.build(self, env);
            self.current_id = last_id;
            r
        } else {
            // same inherited ID
            widget.build(self, env)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Event propagation context.
pub struct EventCtx<'a> {
    /// Parent window handle.
    pub(crate) window: WindowHandle,

    /// Focus state of the parent window.
    pub(crate) window_state: &'a mut WindowFocusState,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<WidgetId>,

    pub change_flags: ChangeFlags,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Event passed to RouteEventCtx
pub struct RouteEventCtx<'a> {
    pub(crate) inner: EventCtx<'a>,
}

impl<'a> RouteEventCtx<'a> {
    /*/// Default event propagation behavior.
    pub fn route_event<E: Element>(&mut self, element: &mut E, event: &mut Event) -> ChangeFlags {
        // this relies on the caller element to bypass this function if it inherits the ID of
        // the child element
        if let Some(next_target) = event.next_target() {
            if Some(element.id()) == self.inner.id {
                warn!("RouteEventCtx::route_event should not be used for a child element with the same ID as its parent. Instead, forward the event directly to the child with `Element::route_event`.")
            }
            element.route_event(self, next_target, event)
        } else {
            element.event(&mut self.inner, event)
        }
    }*/
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Layout context.
pub struct LayoutCtx<'a> {
    /// Parent window handle.
    pub(crate) window: WindowHandle,

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

    /// Returns the scale factor of the parent window.
    pub fn scale_factor(&self) -> Scale {
        self.window.get_scale().unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Hit {
    widget: WidgetId,
}

/// Hit-test context.
pub struct HitTestResult {
    pub(crate) hits: Vec<WidgetId>,
}

impl HitTestResult {
    pub(crate) fn new() -> HitTestResult {
        HitTestResult { hits: vec![] }
    }

    pub fn add(&mut self, id: WidgetId) {
        self.hits.push(id)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Paint context.
pub struct PaintCtx<'a> {
    /// Parent window handle.
    pub(crate) window: WindowHandle,

    /// Focus state of the parent window.
    pub(crate) window_state: &'a mut WindowFocusState,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<WidgetId>,

    /// Drawable surface.
    pub surface: DrawableSurface,
}

impl<'a> PaintCtx<'a> {
    pub fn with_transform<F, R>(&mut self, transform: &Affine, f: F) -> R
    where
        F: FnOnce(&mut PaintCtx<'a>) -> R,
    {
        let scale = self.window.get_scale().unwrap();
        let prev_transform = self.window_transform;
        self.window_transform *= *transform;
        let mut surface = self.surface.surface();
        surface.canvas().save();
        surface.canvas().reset_matrix();
        surface
            .canvas()
            .scale((scale.x() as sk::scalar, scale.y() as sk::scalar));
        surface.canvas().concat(&self.window_transform.to_skia());
        // TODO clip
        let result = f(self);
        let mut surface = self.surface.surface();
        surface.canvas().restore();
        self.window_transform = prev_transform;

        result
    }
}
