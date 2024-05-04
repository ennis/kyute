use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::{hash_map::DefaultHasher, HashMap},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
    num::NonZeroU32,
    ops::{Deref, DerefMut, Index, IndexMut},
    rc::Rc,
};

use kurbo::{Affine, Point};
use skia_safe as sk;
use string_cache::Atom;
use tracing::{trace, warn};
use usvg::Tree;
use winit::{event_loop::EventLoopWindowTarget, window::WindowId};

use crate::{
    application::{AppState, ExtEvent},
    composition::DrawableSurface,
    counter::Counter,
    drawing::ToSkia,
    widget::{WidgetPaths, WidgetPathsRef, WidgetVisitor},
    window::UiHostWindowHandler,
    BoxConstraints, ChangeFlags, Event, EventKind, Geometry, Widget, WidgetId,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Handle to a state value in a `TreeCtx`.
///
/// A handle to a state value that can be accessed by indexing into a `TreeCtx`.
/// It is created with `TreeCtx::with_state`, and is valid only within the closure passed to `with_state`.
///
/// # Example
///
/// TODO
pub struct ContextDataHandle<T> {
    /// Position of the state pointer in the stack.
    index: usize,
    _phantom: PhantomData<fn() -> T>,
}

// Copyable so that it's easily movable in closures
impl<T> Copy for ContextDataHandle<T> {}

impl<T> Clone for ContextDataHandle<T> {
    fn clone(&self) -> Self {
        ContextDataHandle {
            index: self.index,
            _phantom: Default::default(),
        }
    }
}

/// Identifies a context datum.
#[repr(transparent)]
pub struct ContextDataKey<T>(&'static str, PhantomData<fn() -> T>);

impl<T> ContextDataKey<T> {
    pub const fn new(name: &'static str) -> ContextDataKey<T> {
        ContextDataKey(name, PhantomData)
    }
}

impl<T> Clone for ContextDataKey<T> {
    fn clone(&self) -> Self {
        ContextDataKey(self.0, PhantomData)
    }
}

impl<T> Copy for ContextDataKey<T> {}

impl<T> PartialEq for ContextDataKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for ContextDataKey<T> {}

impl<T> Hash for ContextDataKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

struct ContextDataEntry {
    /// Pointer to the state.
    ///
    /// We can't store borrows here because they all have different lifetimes in the stack of state entries
    /// (the bottom of the stack is long-lived, the top is short-lived).
    ///
    /// Access to the state is only possible inside the closure passed to `with_state`, which does borrow
    /// the state mutably. Additionally, to access the state users have to borrow the `TreeCtx` mutably,
    /// which makes it safe.
    data: *mut dyn Any,

    /// The key of the data entry, if it is identified by a key instead of a handle.
    key: Option<&'static str>,

    /// The depth in the widget ID path at which the state was created.
    path_depth: usize,
}

/// Context passed during tree traversals.
pub struct TreeCtx<'a> {
    pub(crate) app_state: &'a mut AppState,
    pub(crate) event_loop: &'a EventLoopWindowTarget<ExtEvent>,

    /// Path to the current widget.
    path: Vec<WidgetId>,

    /// Data in scope.
    ///
    /// Each entry corresponds to a call to `with_state` and contains a pointer to the state value.
    data: Vec<ContextDataEntry>,

    /// Pending updates to widgets (all paths are absolute).
    pending_updates: RefCell<WidgetPaths>,

    /// Pending events
    pending_events: RefCell<Vec<(WidgetPaths, EventKind)>>,
}

impl<'a> TreeCtx<'a> {
    /// Creates the root TreeCtx.
    pub(crate) fn new(app_state: &'a mut AppState, event_loop: &'a EventLoopWindowTarget<ExtEvent>) -> TreeCtx<'a> {
        TreeCtx {
            app_state,
            event_loop,
            data: vec![],
            path: vec![],
            pending_updates: Default::default(),
            pending_events: Default::default(),
        }
    }

    /// Associates the current widget with a window with the specified ID.
    ///
    /// The widget will receive window events from the specified window (via `window_event`).
    ///
    /// # Arguments
    ///
    /// - `window_id`: The ID of the window to associate with the widget.
    ///
    /// # Panics
    ///
    /// Panics if the window is already associated with another widget.
    pub fn register_window(&mut self, window_id: WindowId) {
        //trace!("registering window {:016X}", u64::from(window_id));
        eprintln!("register window {window_id:?} on path {:?}", &self.path[..]);
        if self.app_state.windows.insert(window_id, self.path.clone()).is_some() {
            panic!("window {window_id:?} already registered");
        }
    }

    pub fn current_path(&self) -> &[WidgetId] {
        &self.path
    }

    /*fn dispatch_pending_updates(&mut self, widget: &mut dyn Widget) {
        let pending = mem::take(&mut self.pending_updates);
        // FIXME: pending should be a subtree, not a PathSet, `as_slice` is dubious here
        self.dispatch(widget, pending.as_slice(), &mut |cx, widget| {
            widget.update(cx);
        });
    }*/

    /// Propagates a visitor through the specified widget and its children.
    ///
    /// # Arguments
    /// * `subtree` the subtree to visit, rooted at `current_widget`.
    /// * `widget` the widget to propagate the visitor through, and the widget corresponding to the root of `subtree`.
    ///
    fn dispatch(&mut self, current_widget: &mut dyn Widget, subpaths: WidgetPathsRef, visitor: &mut WidgetVisitor) {
        for (id, is_leaf, rest) in subpaths.traverse() {
            current_widget.visit_child(self, id, &mut |cx: &mut TreeCtx, widget: &mut dyn Widget| {
                cx.with_child(widget, |cx, widget| {
                    if is_leaf {
                        visitor(cx, widget);
                    }
                    cx.dispatch(widget, rest, visitor);
                });
            });
        }
    }

    fn dispatch_root(&mut self, root_widget: &mut dyn Widget, paths: WidgetPathsRef, visitor: &mut WidgetVisitor) {
        for (id, is_leaf, rest) in paths.traverse() {
            if id != root_widget.id() {
                warn!("dispatch: path does not start at the root widget");
            }
            self.with_child(root_widget, |cx, widget| {
                if is_leaf {
                    visitor(cx, widget);
                }
                cx.dispatch(widget, rest, visitor);
            });
        }
    }

    /// Adds an update request that will be processed when the current dispatch is finished.
    pub fn request_update(&self, widgets: WidgetPathsRef) {
        self.pending_updates.borrow_mut().merge_with(widgets);
    }

    /// Schedule an event.
    pub fn schedule_event(&self, widgets: &WidgetPaths, event_kind: EventKind) {
        self.pending_events.borrow_mut().push((widgets.clone(), event_kind));
    }

    pub fn update<T: Widget + ?Sized>(&mut self, child: &mut T) {
        self.with_child(child, |cx, child| {
            child.update(cx);
        });
    }

    fn with_child<T: Widget + ?Sized>(&mut self, child: &mut T, f: impl FnOnce(&mut TreeCtx, &mut T)) {
        self.path.push(child.id());
        f(self, child);
        self.path.pop();
    }

    /// Pushes the specified state value on the context and calls the specified closure.
    ///
    /// # Return value
    /// A tuple `(result, depends_on)`, where `result` is the result of the closure, and
    /// `depends_on` is `true` if the closure depends on the state value (i.e. if it accessed the state).
    ///
    /// # Example
    pub fn with_data<T: 'static, F, R>(&mut self, data: &mut T, f: F) -> R
    where
        F: FnOnce(&mut TreeCtx, ContextDataHandle<T>) -> R,
    {
        let entry = ContextDataEntry {
            data: data as *mut _ as *mut dyn Any,
            key: None,
            path_depth: self.path.len(),
        };
        self.data.push(entry);
        let handle = ContextDataHandle {
            index: self.data.len() - 1,
            _phantom: PhantomData,
        };
        let result = f(self, handle);
        self.data.pop().unwrap();
        result
    }

    pub fn with_keyed_data<T: 'static, F, R>(&mut self, key: ContextDataKey<T>, data: &mut T, f: F) -> R
    where
        F: FnOnce(&mut TreeCtx) -> R,
    {
        let entry = ContextDataEntry {
            data: data as *mut _ as *mut dyn Any,
            key: Some(key.0),
            path_depth: self.path.len(),
        };
        self.data.push(entry);
        let result = f(self);
        self.data.pop().unwrap();
        result
    }

    /// Returns a state entry by index.
    pub fn data<T: 'static>(&self, handle: ContextDataHandle<T>) -> &T {
        let entry = self.data.get(handle.index).expect("invalid state handle");
        // SAFETY: we bind the resulting lifetime to the lifetime of TreeCtx
        // and all the references in the stack are guaranteed to outlive
        // TreeCtx since they are added and removed in only one function: TreeCtx::with_state,
        // and access to the reference can only be done via the closure passed to with_state.
        unsafe { &*entry.data }.downcast_ref::<T>().expect("invalid state type")
    }

    /// Returns a mutable state entry by index.
    pub fn data_mut<T: 'static>(&mut self, handle: ContextDataHandle<T>) -> &mut T {
        let entry = self.data.get_mut(handle.index).expect("invalid state handle");
        // SAFETY: we bind the resulting lifetime to the lifetime of TreeCtx
        // and all the references in the stack are guaranteed to outlive
        // TreeCtx since they are added and removed in only one function: TreeCtx::with_state,
        // and access to the reference can only be done via the closure passed to with_state.
        unsafe { &mut *entry.data }
            .downcast_mut::<T>()
            .expect("invalid state type")
    }

    /// Returns context data by key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not found in the context or if it is of the wrong type.
    pub fn keyed_data<T: 'static>(&self, key: ContextDataKey<T>) -> &T {
        for entry in self.data.iter().rev() {
            if let Some(entry_key) = entry.key {
                if entry_key == key.0 {
                    // SAFETY: same as `data` and `data_mut`
                    return unsafe { &*entry.data }.downcast_ref::<T>().expect("invalid state type");
                }
            }
        }
        panic!("key not found in context");
    }

    /// Performs hit-testing of a subtree.
    pub fn hit_test_child(&mut self, child: &mut dyn Widget, position: Point) -> WidgetPaths {
        let mut result = HitTestResult::new();
        result.path = self.path.clone();
        result.hit_test_child(child, position);
        result.hits
    }
}

/// Dispatches a visitor through the widget tree, to the specified widgets in `paths`.
///
/// If any updates are emitted during the traversal, they are dispatched after the traversal is finished.
///
/// # Arguments
/// * `app_state` - the application state.
/// * `event_loop` - the event loop.
/// * `root` - the root widget.
/// * `paths` - the paths to the widgets to visit.
/// * `visitor` - the visitor to dispatch.
pub(crate) fn root_tree_dispatch(
    app_state: &mut AppState,
    event_loop: &EventLoopWindowTarget<ExtEvent>,
    root: &mut dyn Widget,
    paths: WidgetPathsRef,
    visitor: &mut WidgetVisitor,
) {
    if paths.is_empty() {
        return;
    }

    let mut cx = TreeCtx::new(app_state, event_loop);
    cx.dispatch_root(root, paths, visitor);

    // Handle follow-up updates
    // In case there's an infinite update loop, this will end in a stack overflow.
    let pending_updates = cx.pending_updates.take();
    root_tree_dispatch(
        app_state,
        event_loop,
        root,
        pending_updates.as_slice(),
        &mut |cx: &mut TreeCtx, widget: &mut dyn Widget| {
            widget.update(cx);
        },
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/*
/// A widget that builds a widget given a TreeCtx
pub struct WithContext<F, W> {
    f: F,
    inner: Option<W>,
}

impl<F, W> WithContext<F, W> {
    pub fn new(f: F) -> WithContext<F, W>
    where
        F: FnMut(&mut TreeCtx) -> W,
    {
        WithContext { f, inner: None }
    }
}

impl<F, W> Widget for WithContext<F, W>
where
    F: FnMut(&mut TreeCtx) -> W,
{
    fn id(&self) -> WidgetId {
        self.inner.as_ref().map(|w| w.id()).unwrap_or(WidgetId::ANONYMOUS)
    }

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        self.inner.replace((self.f)(cx));
        ChangeFlags::ALL
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        //self.inner.
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        todo!()
    }
}*/

/// Context passed to `Element::layout`.
pub struct LayoutCtx {
    /// Parent window handle.
    pub scale_factor: f64,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<WidgetId>,
    //pub(crate) debug_info: LayoutDebugInfo,
}

impl LayoutCtx {
    pub(crate) fn new(scale_factor: f64) -> LayoutCtx {
        LayoutCtx {
            scale_factor,
            window_transform: Default::default(),
            id: None,
            //debug_info: Default::default(),
        }
    }

    pub fn layout<T: ?Sized>(&mut self, child_widget: &mut T, constraints: &BoxConstraints) -> Geometry
    where
        T: Widget,
    {
        //let geometry = child_element.layout(self, constraints);
        /*#[cfg(debug_assertions)]
        {
            self.debug_info.add(ElementLayoutDebugInfo {
                element_ptr: elem_ptr_id(child_element),
                constraints: *constraints,
                geometry: geometry.clone(),
            });
        }*/
        child_widget.layout(self, constraints)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Hit-test context.
pub struct HitTestResult {
    hits: WidgetPaths,
    path: Vec<WidgetId>,
}

impl HitTestResult {
    pub(crate) fn new() -> HitTestResult {
        HitTestResult {
            hits: Default::default(),
            path: vec![],
        }
    }

    pub fn hit_test_child(&mut self, child: &dyn Widget, position: Point) -> bool {
        self.path.push(child.id());
        let r = child.hit_test(self, position);
        if r {
            self.hits.insert(&self.path);
        }
        self.path.pop();
        r
    }

    /*pub fn add(&mut self, id: WidgetId) {
        self.hits.insert(&[id]);
    }*/
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Paint context.
pub struct PaintCtx<'a> {
    /// Scale factor.
    pub(crate) scale_factor: f64,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// ID of the parent element
    pub(crate) id: Option<WidgetId>,

    /// Drawable surface.
    pub surface: &'a DrawableSurface,
    //pub(crate) debug_info: PaintDebugInfo,
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

    pub fn paint(&mut self, widget: &mut dyn Widget) {
        /*#[cfg(debug_assertions)]
        {
            self.debug_info.add(PaintElementDebugInfo {
                element_ptr: elem_ptr_id(child_element),
                transform: self.window_transform,
            });
        }*/
        widget.paint(self)
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
