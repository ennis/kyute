use crate::{
    application::{AppState, ExtEvent},
    composition::DrawableSurface,
    drawing::ToSkia,
    widget::Axis,
    window::WindowFocusState,
    AppCtx, ChangeFlags, Element, Environment, Event, Geometry, LayoutParams, Widget,
};
use kurbo::{Affine, Point};
use skia_safe as sk;
use std::{
    any::Any,
    collections::{hash_map::DefaultHasher, HashMap},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU64,
};
use tracing::warn;
use winit::{event_loop::EventLoopWindowTarget, window::WindowId};

////////////////////////////////////////////////////////////////////////////////////////////////////

/// ID of a node in the tree.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElementId(NonZeroU64);

impl ElementId {
    pub const ANONYMOUS: ElementId = ElementId(NonZeroU64::MAX);

    pub fn is_anonymous(self) -> bool {
        self == Self::ANONYMOUS
    }

    pub fn to_u64(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Debug for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08X}", self.to_u64())
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////////

// Child -> Parent
pub type ElementTree = HashMap<ElementId, ElementId>;

struct IdStack {
    id_stack: Vec<NonZeroU64>,
}

impl IdStack {
    /// Creates a new empty CallIdStack.
    fn new() -> IdStack {
        IdStack { id_stack: vec![] }
    }

    fn chain_hash<H: Hash>(&self, s: &H) -> u64 {
        let stacklen = self.id_stack.len();
        let key1 = if stacklen >= 2 {
            self.id_stack[stacklen - 2]
        } else {
            unsafe { NonZeroU64::new_unchecked(0xFFFF_FFFF_FFFF_FFFF) }
        };
        let key0 = if stacklen >= 1 {
            self.id_stack[stacklen - 1]
        } else {
            unsafe { NonZeroU64::new_unchecked(0xFFFF_FFFF_FFFF_FFFF) }
        };
        let mut hasher = DefaultHasher::new();
        key0.hash(&mut hasher);
        key1.hash(&mut hasher);
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Enters a scope in the call graph.
    fn enter<T: Hash>(&mut self, id: &T) -> ElementId {
        let hash = self.chain_hash(id);
        let id = ElementId(NonZeroU64::new(hash).expect("invalid CallId hash"));
        self.id_stack.push(id.0);
        id
    }

    /// Exits a scope previously entered with `enter`.
    fn exit(&mut self) {
        self.id_stack.pop();
    }

    /// Returns the `CallId` of the current scope.
    fn current(&self) -> ElementId {
        ElementId(*self.id_stack.last().unwrap())
    }

    /*/// Returns the current node in the call tree.
    pub fn current_call_node(&self) -> Option<Arc<CallNode>> {
        self.current_node.clone()
    }*/

    /*/// Returns the call node corresponding to the specified CallId.
    pub fn call_node(&self, id: CallId) -> Option<Arc<CallNode>> {
        self.nodes.get(&id).cloned()
    }*/

    /// Returns whether the stack is empty.
    ///
    /// The stack is empty just after creation, and when `enter` and `exit` calls are balanced.
    fn is_empty(&self) -> bool {
        self.id_stack.is_empty()
    }
}

pub struct TreeCtx<'a> {
    pub(crate) app_state: &'a mut AppState,
    pub(crate) event_loop: &'a EventLoopWindowTarget<ExtEvent>,
    pub(crate) tree: &'a mut ElementTree,
    state_stack: Vec<*mut dyn Any>, // issue: these don't have the same lifetime: bottom of the stack is long-lived, top is short-lived
    id_stack: IdStack,
}

impl<'a> TreeCtx<'a> {
    pub(crate) fn new(
        app_state: &'a mut AppState,
        event_loop: &'a EventLoopWindowTarget<ExtEvent>,
        tree: &'a mut ElementTree,
    ) -> TreeCtx<'a> {
        let mut id_stack = IdStack::new();
        // Push a dummy ID on the stack so that the root element gets an ID.
        id_stack.enter(&0);
        TreeCtx {
            app_state,
            event_loop,
            tree,
            state_stack: vec![],
            id_stack,
        }
    }

    fn enter<ID: Hash>(&mut self, id: &ID) -> ElementId {
        self.id_stack.enter(id)
    }

    fn exit(&mut self) {
        self.id_stack.exit();
    }

    /// The element ID of the node.
    pub fn element_id(&self) -> ElementId {
        self.id_stack.current()
    }

    /*/// Call to signal that a child widget has been removed.
    pub fn child_removed(&mut self, id: WidgetId) {
        self.tree.remove(&id);
    }

    /// Call to signal that a child widget is being added.
    pub fn child_added(&mut self, id: WidgetId) {
        if id != WidgetId::ANONYMOUS && self.current_id != WidgetId::ANONYMOUS {
            let prev = self.tree.insert(id, self.current_id);
            if let Some(prev) = prev {
                warn!(
                    "child_added called with id {:?} already in the tree (old parent: {:?}, new parent: {:?})",
                    id, prev, self.current_id
                );
            }
        }
    }*/

    pub fn state<T: Any>(&self) -> Option<&T> {
        for s in self.state_stack.iter().rev() {
            // SAFETY: we bind the resulting lifetime to the lifetime of self
            // and all the references in the stack are guaranteed to outlive
            // self since they are added and removed in only one function: with_state,
            // and access to the reference can only be done via the closure passed to with_state.
            unsafe {
                let s = &**s;
                if let Some(s) = s.downcast_ref::<T>() {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn state_mut<T: Any>(&mut self) -> Option<&mut T> {
        for s in self.state_stack.iter_mut().rev() {
            // SAFETY: same as above, plus state_mut borrows self mutably
            // so it's impossible to call state_mut while there are still
            // mutable references.
            unsafe {
                let s = &mut **s;
                if let Some(s) = s.downcast_mut::<T>() {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn with_state<R>(&mut self, state: &mut dyn Any, f: impl FnOnce(&mut TreeCtx<'a>) -> R) -> R {
        self.state_stack.push(state as *mut dyn Any);
        let result = f(self);
        self.state_stack.pop();
        result
    }

    pub fn build<W: Widget>(&mut self, widget: W) -> W::Element {
        let id = self.element_id();
        widget.build(self, id)

        /*if id != self.current_id && id != WidgetId::ANONYMOUS {
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
        }*/
    }

    pub fn build_with_id<W: Widget, ID: Hash>(&mut self, id: &ID, widget: W) -> W::Element {
        let parent_id = self.element_id();
        self.enter(id);
        self.tree.insert(self.element_id(), parent_id);
        let element = self.build(widget);
        self.exit();
        element
    }

    pub fn update<W: Widget>(&mut self, widget: W, element: &mut W::Element) -> ChangeFlags {
        let element_id = self.element_id();
        assert_eq!(element_id, element.id());
        widget.update(self, element)
    }

    pub fn update_with_id<W: Widget, ID: Hash>(&mut self, id: &ID, widget: W, element: &mut W::Element) -> ChangeFlags {
        self.enter(id);
        let change_flags = self.update(widget, element);
        self.exit();
        change_flags
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Event propagation context.
pub struct EventCtx<'a> {
    /// Parent window handle.
    pub(crate) window: &'a winit::window::Window,

    /// Focus state of the parent window.
    pub(crate) focus: &'a mut Option<ElementId>,
    pub(crate) pointer_capture: &'a mut Option<ElementId>,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<ElementId>,

    pub change_flags: ChangeFlags,
}

impl<'a> EventCtx<'a> {
    pub fn request_focus(&mut self, id: ElementId) {
        *self.focus = Some(id);
    }

    pub fn request_pointer_capture(&mut self, id: ElementId) {
        *self.pointer_capture = Some(id);
    }

    /*pub fn move_focus(&mut self) {

    }*/
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Event passed to RouteEventCtx
pub struct RouteEventCtx<'a> {
    pub inner: EventCtx<'a>,
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
    pub(crate) window: &'a winit::window::Window,

    /// Focus state of the parent window.
    pub(crate) focus: Option<ElementId>,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<ElementId>,
}

impl<'a> LayoutCtx<'a> {
    pub(crate) fn new(window: &'a winit::window::Window, focus: Option<ElementId>) -> LayoutCtx<'a> {
        LayoutCtx {
            window,
            focus,
            window_transform: Default::default(),
            id: None,
        }
    }

    /// Returns the scale factor of the parent window.
    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Hit-test context.
pub struct HitTestResult {
    pub(crate) hits: Vec<ElementId>,
}

impl HitTestResult {
    pub(crate) fn new() -> HitTestResult {
        HitTestResult { hits: vec![] }
    }

    pub fn add(&mut self, id: ElementId) {
        self.hits.push(id)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Paint context.
pub struct PaintCtx<'a> {
    /// Parent window handle.
    pub(crate) window: &'a winit::window::Window,

    /// Focus state of the parent window.
    pub(crate) focus: Option<ElementId>,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<ElementId>,

    /// Drawable surface.
    pub surface: DrawableSurface,
}

impl<'a> PaintCtx<'a> {
    pub fn with_transform<F, R>(&mut self, transform: &Affine, f: F) -> R
    where
        F: FnOnce(&mut PaintCtx<'a>) -> R,
    {
        let scale = self.window.scale_factor() as sk::scalar;
        let prev_transform = self.window_transform;
        self.window_transform *= *transform;
        let mut surface = self.surface.surface();
        surface.canvas().save();
        surface.canvas().reset_matrix();
        surface.canvas().scale((scale, scale));
        surface.canvas().concat(&self.window_transform.to_skia());
        // TODO clip
        let result = f(self);
        let mut surface = self.surface.surface();
        surface.canvas().restore();
        self.window_transform = prev_transform;

        result
    }
}
