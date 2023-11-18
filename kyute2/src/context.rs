use std::{
    any::Any,
    collections::{hash_map::DefaultHasher, HashMap},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
    num::NonZeroU64,
    ops::{Deref, DerefMut, Index, IndexMut},
    rc::Rc,
};

use kurbo::Affine;
use skia_safe as sk;
use winit::event_loop::EventLoopWindowTarget;

use crate::{
    application::{AppState, ExtEvent},
    composition::DrawableSurface,
    debug_util::{
        elem_ptr_id, ElementLayoutDebugInfo, EventDebugInfo, EventHandlingDebugInfo, LayoutDebugInfo, PaintDebugInfo,
        PaintElementDebugInfo,
    },
    drawing::ToSkia,
    window::UiHostWindowHandler,
    AppCtx, BoxConstraints, ChangeFlags, Element, Event, Geometry, Widget,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

/// ID of a UI element (`dyn Element`).
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct ElementId(NonZeroU64);

impl ElementId {
    /// ID used for anonymous elements.
    ///
    /// IDs are only needed by elements that need to receive events.
    /// If the element doesn't need to receive events, it can use this anonymous ID instead of
    /// generating a unique ID.
    pub const ANONYMOUS: ElementId = ElementId(NonZeroU64::MAX);

    /// Returns whether the ID is the anonymous ID.
    pub fn is_anonymous(self) -> bool {
        self == Self::ANONYMOUS
    }

    /// Converts the ID to a `u64` value.
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

/// Handle to a state value in a `TreeCtx`.
///
/// A handle to a state value that can be accessed by indexing into a `TreeCtx`.
/// It is created with `TreeCtx::with_state`, and is valid only within the closure passed to `with_state`.
///
/// # Example
///
/// TODO
pub struct State<T> {
    /// Position of the state pointer in the stack.
    index: usize,
    _phantom: PhantomData<fn() -> T>,
}

// Copyable so that it's easily movable in closures
impl<T> Copy for State<T> {}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        State {
            index: self.index,
            _phantom: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Trait to access ambient state by type in a `TreeCtx`.
///
/// It is blanket-implemented for all `'static` types.
pub trait Ambient: 'static {
    /// Returns a reference to an ambient value of this type in the specified context.
    fn ambient<'a>(ctx: &'a TreeCtx) -> Option<&'a Self>;

    /// Returns a reference to an ambient value of this type in the specified context,
    /// or a default value if no ambient value of the specified type is found.
    fn ambient_or_default(ctx: &TreeCtx) -> Self
    where
        Self: Default + Clone,
    {
        Self::ambient(ctx).cloned().unwrap_or_default()
    }
}

impl<T> Ambient for T
where
    T: 'static,
{
    fn ambient<'a>(ctx: &'a TreeCtx) -> Option<&'a Self> {
        ctx.ambient()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

//pub type ElementTreeHash = HashMap<ElementId, ElementId>;

/// Maps an element ID to its parent.
///
/// This is mainly used to determine the propagation path of window events
/// (like keyboard events, pointer events, etc.).
#[derive(Default, Clone, Debug)]
pub struct ElementIdTree {
    pub(crate) map: HashMap<ElementId, ElementId>,
}

impl ElementIdTree {
    /// Inserts a parent-child relationship between two elements.
    ///
    /// # Arguments
    /// * `element` the child element
    /// * `parent` the parent element
    pub fn insert(&mut self, element: ElementId, parent: ElementId) {
        self.map.insert(element, parent);
    }

    /// Gets the parent element and owner window of the specified element.
    pub fn get(&self, element: ElementId) -> Option<ElementId> {
        self.map.get(&element).cloned()
    }

    /// Returns the chain of element IDs from the specified one to the highest ancestor that it still
    /// in the same window as the specified element.
    pub fn id_path(&self, element: ElementId) -> Vec<ElementId> {
        let mut path = vec![element];
        let mut current = element;
        while let Some(parent) = self.get(current) {
            path.push(parent);
            current = parent;
        }
        path
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

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

    /*
    /// Returns whether the stack is empty.
    ///
    /// The stack is empty just after creation, and when `enter` and `exit` calls are balanced.
    fn is_empty(&self) -> bool {
        self.id_stack.is_empty()
    }*/
}

/// TODO rename this to something more meaningful (what "tree" are we talking about?)
pub struct TreeCtx<'a> {
    /// Application context.
    pub app_ctx: AppCtx<'a>,

    /// Parent window ID.
    ///
    /// TODO this should be Rc<UiHostWindowHandler> directly. Don't bother supporting different kinds of windows for now.
    parent_window: Option<Rc<UiHostWindowHandler>>,

    /// Keeps track of parent-child relationships between element IDs.
    tree: ElementIdTree,

    /// Ambient states in scope.
    ambient_stack: Vec<*const dyn Any>, // issue: these don't have the same lifetime: bottom of the stack is long-lived, top is short-lived

    /// States in scope.
    state_stack: Vec<*mut dyn Any>, // issue: these don't have the same lifetime: bottom of the stack is long-lived, top is short-lived

    /// ID stack used to generate unique IDs for elements.
    id_stack: IdStack,
}

impl<'a> Deref for TreeCtx<'a> {
    type Target = AppCtx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.app_ctx
    }
}

impl<'a> DerefMut for TreeCtx<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app_ctx
    }
}

impl<'a> TreeCtx<'a> {
    /// Creates the root TreeCtx.
    pub(crate) fn new(app_state: &'a mut AppState, event_loop: &'a EventLoopWindowTarget<ExtEvent>) -> TreeCtx<'a> {
        let mut id_stack = IdStack::new();
        // Push a dummy ID on the stack so that the root element gets an ID.
        id_stack.enter(&0);
        TreeCtx {
            app_ctx: AppCtx { app_state, event_loop },
            parent_window: None,
            tree: ElementIdTree::default(),
            ambient_stack: vec![],
            state_stack: vec![],
            id_stack,
        }
    }

    /// Sets the parent window and runs the specified closure with the updated context.
    ///
    /// # Safety
    ///
    /// * The caller must ensure the validity of the window handle for the duration of the function.
    /// * The caller must ensure all safety conditions related to the use of the handle in as a parent
    ///   window in winit APIs.
    pub fn with_parent_window<R>(
        &mut self,
        window_handler: Rc<UiHostWindowHandler>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let prev_parent_window = mem::replace(&mut self.parent_window, Some(window_handler));
        let result = f(self);
        self.parent_window = prev_parent_window;
        result
    }

    /// Updates an element that has its own separate element tree.
    ///
    /// Child windows keep a separate `ElementIdTree` corresponding to the subtree of elements
    /// rooted at the child window. This method allows to update that `ElementIdTree` independently of
    /// the element tree of any parent elements.
    ///
    /// # Arguments
    ///
    /// * `content` the widget used to update the element
    /// * `element` the element to update
    /// * `element_tree` the element tree returned by the previous call to `update_with_element_tree`, or the empty tree if this is the first call
    ///
    /// # Return value
    ///
    /// A tuple `(change_flags, element_tree)` with the change flags that resulted from the element update, and the updated element tree.
    ///
    /// # Implementation notes
    ///
    /// We pass the element tree o
    pub fn update_with_id_tree<T>(
        &mut self,
        content: T,
        element: &mut Box<dyn Element>,
        element_tree: &mut ElementIdTree,
    ) -> ChangeFlags
    where
        T: Widget + Any,
    {
        // This is a bit convoluted, but this way we don't have to deal with an additional lifetime
        // in `TreeCtx`
        mem::swap(&mut self.tree, element_tree);
        // NOTE: this is pretty much the same logic as AnyWidget::update
        let change_flags = if let Some(element) = (&mut **element).as_any_mut().downcast_mut::<T::Element>() {
            // We can update the element in place if it's the expected type
            self.update(content, element)
        } else {
            // Otherwise it is rebuilt from scratch
            eprintln!("rebuilding element");
            *element = Box::new(self.build(content));
            ChangeFlags::STRUCTURE
        };
        mem::swap(&mut self.tree, element_tree);
        change_flags
    }

    /// Returns the current parent window handle.
    pub fn parent_window(&self) -> Option<Rc<UiHostWindowHandler>> {
        self.parent_window.clone()
    }

    /// Appends an ID to the current ID path.
    ///
    /// Should be matched by a call to `exit`.
    fn enter<ID: Hash>(&mut self, id: &ID) -> ElementId {
        self.id_stack.enter(id)
    }

    /// Removes the last ID from the current ID path.
    fn exit(&mut self) {
        self.id_stack.exit();
    }

    /// The current element ID.
    pub fn current_id(&self) -> ElementId {
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

    /// Looks up an ambient state entry with the specified type in this context and returns a reference to it.
    pub fn ambient<T: Any>(&self) -> Option<&T> {
        for s in self.ambient_stack.iter().rev() {
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

    /// Pushes the specified state value on the context and calls the specified closure.
    ///
    /// # Example
    pub fn with_state<T, F, R>(&mut self, state: &mut T, f: F) -> R
    where
        T: Any,
        F: FnOnce(&mut TreeCtx<'a>, State<T>) -> R,
    {
        self.state_stack.push(state as &mut dyn Any as *mut dyn Any);
        let handle = State {
            index: self.state_stack.len() - 1,
            _phantom: PhantomData,
        };
        let result = f(self, handle);
        self.state_stack.pop();
        result
    }

    /// Pushes the specified ambient value on the context and calls the specified closure.
    ///
    /// The ambient value is accessible in the closure with `Ambient::ambient`.
    /// # Example
    pub fn with_ambient<T, F, R>(&mut self, value: &T, f: F) -> R
    where
        T: Any,
        F: FnOnce(&mut TreeCtx<'a>) -> R,
    {
        self.ambient_stack.push(value as &dyn Any as *const dyn Any);
        let result = f(self);
        self.ambient_stack.pop();
        result
    }

    /// Builds a child widget.
    pub fn build<W: Widget>(&mut self, widget: W) -> W::Element {
        let id = self.current_id();
        widget.build(self, id)
    }

    /// Builds a child widget with the specified ID.
    pub fn build_with_id<W: Widget, ID: Hash>(&mut self, id: &ID, widget: W) -> W::Element {
        let parent_id = self.current_id();
        self.enter(id);
        self.tree.insert(self.current_id(), parent_id);
        let element = self.build(widget);
        self.exit();
        element
    }

    /// Updates an element from the provided widget.
    pub fn update<W: Widget>(&mut self, widget: W, element: &mut W::Element) -> ChangeFlags {
        let current_id = self.current_id();
        let element_id = element.id();
        assert!(current_id == element_id || element_id == ElementId::ANONYMOUS);
        widget.update(self, element)
    }

    /// Updates an element from the provided widget with the specified ID.
    pub fn update_with_id<W: Widget, ID: Hash>(&mut self, id: &ID, widget: W, element: &mut W::Element) -> ChangeFlags {
        self.enter(id);
        let change_flags = self.update(widget, element);
        self.exit();
        change_flags
    }
}

impl<'a, T: 'static> Index<State<T>> for TreeCtx<'a> {
    type Output = T;

    fn index(&self, state: State<T>) -> &Self::Output {
        let ptr = self.state_stack.get(state.index).expect("invalid state handle");
        // SAFETY: we bind the resulting lifetime to the lifetime of self
        // and all the references in the stack are guaranteed to outlive
        // self since they are added and removed in only one function: with_state,
        // and access to the reference can only be done via the closure passed to with_state.
        unsafe {
            let ptr = &**ptr;
            ptr.downcast_ref::<T>().expect("invalid state handle")
        }
    }
}

impl<'a, T: 'static> IndexMut<State<T>> for TreeCtx<'a> {
    fn index_mut(&mut self, state: State<T>) -> &mut Self::Output {
        let ptr = self.state_stack.get(state.index).expect("invalid state handle");
        // SAFETY: same as above, plus state_mut borrows self mutably
        // so it's impossible to call index_mut while there are still
        // mutable references.
        unsafe {
            let ptr = &mut **ptr;
            // NOTE: downcast_mut_unchecked would be unsound: the state could be moved
            // to another branch of the state tree. The stack would be of the same size in the
            // new branch, but the type of the state could be different.
            ptr.downcast_mut::<T>().expect("invalid state handle")
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Event propagation context.
pub struct EventCtx<'a> {
    /// Focus state of the parent window.
    pub(crate) focus: &'a mut Option<ElementId>,
    pub(crate) pointer_capture: &'a mut Option<ElementId>,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<ElementId>,

    pub change_flags: ChangeFlags,
    pub debug_info: EventDebugInfo,
}

impl<'a> EventCtx<'a> {
    pub fn event<T>(&mut self, child_element: &mut T, event: &mut Event) -> ChangeFlags
    where
        T: Element,
    {
        let change_flags = child_element.event(self, event);
        #[cfg(debug_assertions)]
        {
            self.debug_info.add(EventHandlingDebugInfo {
                element_ptr: elem_ptr_id(child_element),
                element_id: child_element.id(),
                event: event.kind.clone(),
                handled: false,
                change_flags: change_flags.clone(),
            });
        }
        change_flags
    }

    pub fn request_focus(&mut self, id: ElementId) {
        *self.focus = Some(id);
    }

    pub fn request_pointer_capture(&mut self, id: ElementId) {
        *self.pointer_capture = Some(id);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Context passed to `Element::layout`.
pub struct LayoutCtx {
    /// Parent window handle.
    pub scale_factor: f64,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<ElementId>,

    pub(crate) debug_info: LayoutDebugInfo,
}

impl LayoutCtx {
    pub(crate) fn new(scale_factor: f64) -> LayoutCtx {
        LayoutCtx {
            scale_factor,
            window_transform: Default::default(),
            id: None,
            debug_info: Default::default(),
        }
    }

    pub fn layout<T>(&mut self, child_element: &mut T, constraints: &BoxConstraints) -> Geometry
    where
        T: Element,
    {
        let geometry = child_element.layout(self, constraints);
        #[cfg(debug_assertions)]
        {
            self.debug_info.add(ElementLayoutDebugInfo {
                element_ptr: elem_ptr_id(child_element),
                constraints: *constraints,
                geometry: geometry.clone(),
            });
        }
        geometry
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
    /// Scale factor.
    pub(crate) scale_factor: f64,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<ElementId>,

    /// Drawable surface.
    pub surface: &'a DrawableSurface,

    pub(crate) debug_info: PaintDebugInfo,
}

impl<'a> PaintCtx<'a> {
    pub fn with_transform<F, R>(&mut self, transform: &Affine, f: F) -> R
    where
        F: FnOnce(&mut PaintCtx<'a>) -> R,
    {
        let scale = self.scale_factor as sk::scalar;
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

    pub fn paint(&mut self, child_element: &mut dyn Element) {
        #[cfg(debug_assertions)]
        {
            self.debug_info.add(PaintElementDebugInfo {
                element_ptr: elem_ptr_id(child_element),
                transform: self.window_transform,
            });
        }

        child_element.paint(self);
    }

    pub fn with_canvas<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut sk::Canvas) -> R,
    {
        let mut surface = self.surface.surface();
        let result = f(surface.canvas());
        result
    }
}
