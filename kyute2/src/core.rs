//! Widget tree manipulation and traversal.

use std::{
    any::Any,
    cell::{Cell, Ref, RefCell},
    fmt,
    hash::Hash,
    mem,
    rc::{Rc, Weak},
    time::Duration,
};

use crate::{
    application::{AppState, ExtEvent},
    composition::DrawableSurface,
    drawing::ToSkia,
    environment::EnvValue,
    text::TextSpan,
    widgets::Null,
    BoxConstraints, Environment, Event, Geometry,
};
use bitflags::bitflags;
use kurbo::{Affine, Point, Vec2};
use skia_safe as sk;
use weak_table::PtrWeakHashSet;
use winit::{event::WindowEvent, event_loop::EventLoopWindowTarget, window::WindowId};

////////////////////////////////////////////////////////////////////////////////////////////////////

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ChangeFlags: u32 {
        const NONE = 0;
        /// Any structural change (child added / removed).
        const STRUCTURE = (1 << 0);
        /// The geometry of the element has changed.
        const GEOMETRY = (1 << 1);
        // Geometry has changed (SIZE | POSITIONING)
        //const GEOMETRY = Self::SIZE.bits() | Self::POSITIONING.bits();
        /// Element must be repainted.
        const PAINT = (1<<3);
        /// The app logic may need to be re-run.
        const APP_LOGIC = (1<<4);

        /// Child geometry may have changed.
        const CHILD_GEOMETRY = (1<<5);
        /// (Layout) constraints have changed.
        const LAYOUT_CONSTRAINTS = (1<<6);
        /// (Layout) child positions within the parent may have changed.
        const LAYOUT_CHILD_POSITIONS = (1<<7);
        /// The baseline of the element has changed.
        const BASELINE_CHANGED = (1<<8);

        const LAYOUT_FLAGS = Self::CHILD_GEOMETRY.bits()
            | Self::LAYOUT_CONSTRAINTS.bits()
            | Self::LAYOUT_CHILD_POSITIONS.bits()
            | Self::BASELINE_CHANGED.bits();

        const ALL = 0xFFFF;
    }
}

/// Context passed to `Element::layout`.
pub struct LayoutCtx {
    /// Parent window handle.
    pub scale_factor: f64,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,
}

impl LayoutCtx {
    pub(crate) fn new(scale_factor: f64) -> LayoutCtx {
        LayoutCtx {
            scale_factor,
            window_transform: Default::default(),
        }
    }

    /*pub fn layout<T: ?Sized>(&mut self, child_widget: &mut T, constraints: &BoxConstraints) -> Geometry
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
    }*/
}

////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Paint context.
pub struct PaintCtx<'a> {
    /// Scale factor.
    pub(crate) scale_factor: f64,

    /// Transform from window area to the current element.
    pub(crate) window_transform: Affine,

    /// Drawable surface.
    pub surface: &'a DrawableSurface,
    //pub(crate) debug_info: PaintDebugInfo,
}

impl<'a> PaintCtx<'a> {
    pub fn with_offset<F, R>(&mut self, offset: Vec2, f: F) -> R
    where
        F: FnOnce(&mut PaintCtx<'a>) -> R,
    {
        self.with_transform(&Affine::translate(offset), f)
    }

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

/// Widget types.
///
/// See the crate documentation for more information.
pub trait Widget: Any {
    // On pointer down
    // If widget declares grab, also set the transform (coordinate space)
    // then the widget receives all subsequent pointer events in the specified coordinate space
    // (even if somehow the widget has moved in the meantime)

    /// Updates this widget.
    ///
    /// This is called when something has determined that this widget needs to update itself somehow.
    /// This can be because:
    /// - a state dependency of this widget has changed
    /// - the widget was just inserted into the widget tree
    ///
    /// # Guidelines
    ///
    /// You shouldn't have to manually call `update()` on child widgets. Instead, request an update by
    /// calling `cx.request_update(widgetpaths)`.
    fn update(&mut self, cx: &mut WidgetCtx);

    fn environment(&self) -> Environment {
        Environment::new()
    }

    /// Event handling.
    fn event(&mut self, cx: &mut WidgetCtx, event: &mut Event);

    /// Hit-testing.
    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool;

    /// Layout.
    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry;

    /// Raw window events.
    fn window_event(&mut self, _cx: &mut WidgetCtx, _event: &winit::event::WindowEvent, _time: Duration) {}

    fn paint(&mut self, cx: &mut PaintCtx);
}

pub trait ProxyWidget: Any {
    type Inner: Widget;
    fn inner(&self) -> &Self::Inner;
    fn inner_mut(&mut self) -> &mut Self::Inner;
}

impl<T: ProxyWidget> Widget for T {
    fn update(&mut self, cx: &mut WidgetCtx) {
        self.inner_mut().update(cx);
    }

    fn environment(&self) -> Environment {
        self.inner().environment()
    }

    fn event(&mut self, cx: &mut WidgetCtx, event: &mut Event) {
        self.inner_mut().event(cx, event);
    }

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        self.inner_mut().hit_test(result, position)
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        self.inner_mut().layout(cx, bc)
    }

    fn window_event(&mut self, cx: &mut WidgetCtx, event: &WindowEvent, time: Duration) {
        self.inner_mut().window_event(cx, event, time);
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        self.inner_mut().paint(cx);
    }
}

pub struct WidgetPod<T: ?Sized = dyn Widget> {
    mounted: Cell<bool>,
    parent: RefCell<WeakWidgetPtr>,
    environment: RefCell<Environment>,
    pub widget: RefCell<T>,
}

pub type WidgetPtr<T = dyn Widget> = Rc<WidgetPod<T>>;
pub type WeakWidgetPtr<T = dyn Widget> = Weak<WidgetPod<T>>;

impl<T> WidgetPod<T> {
    pub fn new(widget: T) -> WidgetPtr<T> {
        Rc::new(WidgetPod {
            mounted: Cell::new(false),
            // Weak::new() doesn't work with unsized types, so use the dummy Null widgets (https://github.com/rust-lang/rust/issues/50513)
            parent: RefCell::new(WeakWidgetPtr::<Null>::new()),
            environment: Default::default(),
            widget: RefCell::new(widget),
        })
    }
}

pub trait IntoWidgetPod {
    fn into_widget_pod(self) -> WidgetPtr;
}

impl IntoWidgetPod for WidgetPtr {
    fn into_widget_pod(self) -> WidgetPtr {
        self
    }
}

impl<T: Widget> IntoWidgetPod for T {
    fn into_widget_pod(self) -> WidgetPtr {
        WidgetPod::new(self)
    }
}

impl<T: Widget + 'static> WidgetPod<T> {
    pub fn as_dyn(self: &Rc<Self>) -> WidgetPtr {
        self.clone()
    }
}

impl<T: ?Sized> fmt::Debug for WidgetPod<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetPod").finish_non_exhaustive()
    }
}

impl WidgetPod {
    /*pub fn weak(self: &Rc<self>) -> WeakWidgetPtr<T> {
        Rc::downgrade(&self.0))
    }*/

    /// Sends an event to the specified widgets.
    pub fn event(self: &Rc<Self>, cx: &mut WidgetCtx, event: &mut Event) {
        cx.with_widget(self, |cx, widget| {
            widget.event(cx, event);
        })
    }

    /*pub fn downcast<T: Widget + 'static>(self: Rc<Self>) -> Result<WidgetPtr<T>, WidgetPtr> {
        let type_id = {
            let widgets = &*self.widgets.borrow();
            widgets.type_id()
        };
        if type_id == TypeId::of::<T>() {
            unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as *const WidgetPod<T>)) }
        } else {
            Err(self)
        }
    }*/

    /*/// Dispatches a pointer event.
    pub fn dispatch_pointer_event(self: &Rc<Self>, cx: &mut TreeCtx, path: &[HitTestEntry], event: &mut Event) {
        for entry in path {
            event.set_transform(&entry.transform);
            entry.widgets.event(cx, event);
        }
    }*/

    pub fn window_event(self: &Rc<Self>, cx: &mut WidgetCtx, event: &winit::event::WindowEvent, time: Duration) {
        cx.with_widget(self, |cx, widget| {
            widget.window_event(cx, event, time);
        });
    }

    pub fn update(self: &Rc<Self>, cx: &mut WidgetCtx) {
        if !self.mounted.get() {
            self.mounted.set(true);
            let parent = cx.current();
            let env = self
                .widget
                .borrow()
                .environment()
                .union(parent.environment.borrow().clone());
            self.environment.replace(env);
            self.parent.replace(Rc::downgrade(&cx.current()));
        }
        cx.with_widget(self, |cx, widget| {
            widget.update(cx);
        });
    }

    pub fn hit_test(self: &Rc<Self>, result: &mut HitTestResult, position: Point) -> bool {
        let hit = self.widget.borrow_mut().hit_test(result, position);
        if hit {
            result.add(self.clone());
        }
        hit
    }

    pub fn layout(&self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        self.widget.borrow_mut().layout(cx, bc)
    }

    pub fn paint(&self, cx: &mut PaintCtx) {
        self.widget.borrow_mut().paint(cx);
    }
}

#[derive(Clone, Debug)]
pub struct HitTestEntry {
    pub widget: WidgetPtr,
    pub transform: Affine,
}

impl HitTestEntry {
    pub fn same(&self, other: &HitTestEntry) -> bool {
        Rc::ptr_eq(&self.widget, &other.widget)
    }
}

/// Hit-test context.
#[derive(Clone, Debug)]
pub struct HitTestResult {
    current_transform: Affine,
    pub hits: Vec<HitTestEntry>,
}

impl HitTestResult {
    pub fn new() -> HitTestResult {
        HitTestResult {
            current_transform: Default::default(),
            hits: Default::default(),
        }
    }

    pub fn test_with_offset(
        &mut self,
        offset: Vec2,
        position: Point,
        f: impl FnOnce(&mut Self, Point) -> bool,
    ) -> bool {
        self.test_with_transform(&Affine::translate(offset), position, f)
    }

    pub fn test_with_transform(
        &mut self,
        transform: &Affine,
        position: Point,
        f: impl FnOnce(&mut Self, Point) -> bool,
    ) -> bool {
        let prev_transform = self.current_transform;
        self.current_transform *= *transform;
        let hit = f(self, transform.inverse() * position);
        self.current_transform = prev_transform;
        hit
    }

    pub fn add(&mut self, widget: WidgetPtr) {
        self.hits.push(HitTestEntry {
            widget,
            transform: self.current_transform,
        });
    }
}

/// Context passed during tree traversals.
pub struct WidgetCtx<'a> {
    pub(crate) app_state: &'a mut AppState,
    pub(crate) event_loop: &'a EventLoopWindowTarget<ExtEvent>,
    /// Pointer to the current widgets.
    current_widget: WidgetPtr,
    /// Widgets that need updating after the current dispatch.
    /// XXX do we need RefCell here?
    need_update: RefCell<PtrWeakHashSet<WeakWidgetPtr>>,
    /// Whether relayout is necessary.
    relayout: bool,
}

impl<'a> WidgetCtx<'a> {
    /// Creates the root TreeCtx.
    pub(crate) fn new(
        app_state: &'a mut AppState,
        event_loop: &'a EventLoopWindowTarget<ExtEvent>,
        target: WidgetPtr,
    ) -> WidgetCtx<'a> {
        WidgetCtx {
            app_state,
            event_loop,
            current_widget: target,
            need_update: RefCell::new(Default::default()),
            relayout: false,
        }
    }

    /// Associates the current widgets with a window with the specified ID.
    ///
    /// The widgets will receive window events from the specified window (via `window_event`).
    ///
    /// # Arguments
    ///
    /// - `window_id`: The ID of the window to associate with the widgets.
    ///
    /// # Panics
    ///
    /// Panics if the window is already associated with another widgets.
    pub fn register_window(&mut self, window_id: WindowId) {
        //trace!("registering window {:016X}", u64::from(window_id));
        //eprintln!("register window {window_id:?} on path {:?}", &self.path[..]);
        if self
            .app_state
            .windows
            .insert(window_id, self.current_widget.clone())
            .is_some()
        {
            panic!("window {window_id:?} already registered");
        }
    }

    pub fn current(&self) -> WidgetPtr {
        self.current_widget.clone()
    }

    pub fn dispatch_pending_updates(&mut self) {
        while self.need_update.borrow().len() > 0 {
            let mut need_update = self.need_update.take();
            for widget in need_update.drain() {
                assert!(widget.mounted.get());
                widget.update(self);
            }
        }
    }

    #[must_use]
    pub fn needs_layout(&self) -> bool {
        self.relayout
    }

    /*/// Propagates a visitor through the specified widgets and its children.
    ///
    /// # Arguments
    /// * `subtree` the subtree to visit, rooted at `current_widget`.
    /// * `widgets` the widgets to propagate the visitor through, and the widgets corresponding to the root of `subtree`.
    ///
    fn dispatch(&mut self, current_widget: &mut dyn Widget, subpaths: &WidgetSlice, visitor: &mut WidgetVisitor) {
        for (id, is_leaf, rest) in subpaths.traverse() {
            current_widget.visit_child(self, id, &mut |cx: &mut TreeCtx, widgets: &mut dyn Widget| {
                cx.with_child(widgets, |cx, widgets| {
                    if is_leaf {
                        visitor(cx, widgets);
                    }
                    cx.dispatch(widgets, rest, visitor);
                });
            });
        }
    }

    fn dispatch_root(&mut self, root_widget: &mut dyn Widget, paths: &WidgetSlice, visitor: &mut WidgetVisitor) {
        for (id, is_leaf, rest) in paths.traverse() {
            if id != root_widget.id() {
                warn!("dispatch: path does not start at the root widgets");
            }
            self.with_child(root_widget, |cx, widgets| {
                if is_leaf {
                    visitor(cx, widgets);
                }
                cx.dispatch(widgets, rest, visitor);
            });
        }
    }

    /// Dispatches an event to child widgets.
    pub fn dispatch_event(&mut self, current_widget: &mut dyn Widget, paths: &WidgetSlice, event: EventKind) -> bool {
        let mut event = Event::new(event);
        self.dispatch(
            current_widget,
            paths,
            &mut |cx: &mut TreeCtx, widgets: &mut dyn Widget| {
                if !event.handled {
                    widgets.event(cx, &mut event);
                }
            },
        );
        event.handled
    }*/

    /// Adds an update request that will be processed when the current dispatch is finished.
    pub fn mark_needs_update(&self, widget: WidgetPtr) {
        self.need_update.borrow_mut().insert(widget);
    }

    pub fn mark_needs_layout(&mut self) {
        self.relayout = true;
    }

    fn with_widget<R>(&mut self, child: &WidgetPtr, f: impl FnOnce(&mut WidgetCtx, &mut dyn Widget) -> R) -> R {
        let prev_widget = mem::replace(&mut self.current_widget, child.clone());
        let r = f(self, &mut *child.widget.borrow_mut());
        self.current_widget = prev_widget;
        r
    }

    /*/// Pushes the specified state value on the context and calls the specified closure.
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
        let entry = crate::context::ContextDataEntry {
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
        let entry = crate::context::ContextDataEntry {
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
    }*/

    /*
    /// Performs hit-testing of a subtree.
    pub fn hit_test_child(&mut self, child: &mut dyn Widget, position: Point) -> Vec<WidgetPtr> {
        let mut result = HitTestResult::new();
        result.hit_test_child(child, position);
        result.hits
    }*/

    // FIXME: this is completely unreliable, because widget types are almost always wrapped
    // in modifiers like `Frame` or `Padding`, so we almost never find the type we're looking for
    // (e.g. we can't find `Clickable` because it's wrapped in `Frame<Clickable>`).

    /*// Solutions:
    // - return state in a separate method, which wrapper widgets would forward to
    // - all widgets (even simple wrappers) have associated nodes (Padding, Frame, Transform, Constrained, Decoration)
    //    -> *** probably the best option; it's best to not be too clever with this
    pub fn find_ancestor<T: Widget + 'static>(&self) -> Option<WidgetPtr<T>> {
        let mut current = self.current_widget.clone();

        loop {
            match current.downcast::<T>() {
                Ok(v) => return Some(v),
                Err(v) => {
                    current = v.parent.borrow().upgrade()?;
                }
            }
        }
    }*/

    pub fn env<T: EnvValue>(&self) -> Option<T> {
        self.current_widget.environment.borrow().get::<T>()
    }
}

/// A widget that builds a widget given a TreeCtx
pub struct Builder<F> {
    f: F,
    inner: Option<WidgetPtr>,
}

impl<F> Builder<F> {
    pub fn new<W>(f: F) -> Builder<F>
    where
        W: Widget,
        F: Fn(&mut WidgetCtx) -> W,
    {
        Builder { f, inner: None }
    }
}

impl<F, W> Widget for Builder<F>
where
    F: Fn(&mut WidgetCtx) -> W + 'static,
    W: Widget + 'static,
{
    fn update(&mut self, cx: &mut WidgetCtx) {
        self.inner = {
            let widget: WidgetPtr = WidgetPod::new((self.f)(cx));
            widget.update(cx);
            cx.mark_needs_layout();
            Some(widget)
        };
    }

    fn event(&mut self, cx: &mut WidgetCtx, event: &mut Event) {}

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        if let Some(ref inner) = self.inner {
            inner.hit_test(result, position)
        } else {
            false
        }
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        if let Some(ref inner) = self.inner {
            inner.layout(cx, bc)
        } else {
            Geometry::default()
        }
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        if let Some(ref inner) = self.inner {
            inner.paint(cx);
        }
    }
}

/*

// An alternative Builder that passes a child widget to the builder closures.
// This is useful if the widget returned by the builder changes often, but a subtree of it remains the same.

pub struct Rebuilder<F, W> {
    builder: F,
    child: Option<W>,
    inner: Option<WidgetPtr>,
}

impl<F, W, ParentWidget> Widget for Rebuilder<F, W>
where
    F: FnMut(&mut TreeCtx, W) -> ParentWidget,
    W: Widget,
{
    fn update(&mut self, cx: &mut TreeCtx) {
        //if let Some(cl)
        //todo!()
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) {
        todo!()
    }

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        todo!()
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        todo!()
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        todo!()
    }
}

pub fn builder<F, W>(f: F) -> Builder<F>
where
    W: Widget,
    F: Fn(&mut TreeCtx) -> W,
{
    Builder::new(f)
}*/

pub struct State<T: ?Sized>(Rc<StateInner<T>>);

impl<T: 'static> EnvValue for State<T> {
    fn into_storage(self) -> Rc<dyn Any> {
        self.0.clone()
    }

    fn from_storage(storage: Rc<dyn Any>) -> Self {
        Self(storage.downcast().unwrap())
    }
}

impl<T: Default + 'static> Default for State<T> {
    fn default() -> Self {
        State::new(T::default())
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        State(Rc::clone(&self.0))
    }
}

impl<T: 'static> State<T> {
    /// Creates a new state with the specified data.
    pub fn new(data: T) -> Self {
        State(Rc::new(StateInner {
            dependents: Default::default(),
            data: RefCell::new(data),
        }))
    }

    pub fn set_dependency(&self, cx: &WidgetCtx) {
        self.0.dependents.borrow_mut().insert(cx.current());
    }

    fn notify(&self, cx: &WidgetCtx) {
        for dep in self.0.dependents.borrow().iter() {
            cx.mark_needs_update(dep);
        }
    }

    /// Modifies the state and notify the dependents.
    pub fn set(&self, cx: &mut WidgetCtx, value: T) {
        self.0.data.replace(value);
        self.notify(cx);
    }

    /// Modifies the state and notify the dependents.
    pub fn update<R>(&self, cx: &mut WidgetCtx, f: impl FnOnce(&mut T) -> R) -> R {
        let mut data = self.0.data.borrow_mut();
        let r = f(&mut *data);
        self.notify(cx);
        r
    }

    /// Returns the current value of the state, setting a dependency on the current widget.
    pub fn get(&self, cx: &mut WidgetCtx) -> Ref<T> {
        self.set_dependency(cx);
        self.0.data.borrow()
    }

    pub fn get_untracked(&self) -> Ref<T> {
        self.0.data.borrow()
    }

    pub fn at(cx: &mut WidgetCtx) -> Option<T>
    where
        T: Clone,
    {
        cx.env::<State<T>>().map(|state| state.get(cx).clone())
    }
}

impl State<dyn Any> {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&State<T>> {
        if self.0.data.borrow().is::<T>() {
            // SAFETY: the data is of the correct type
            Some(unsafe { &*(self as *const _ as *const State<T>) })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut State<T>> {
        if self.0.data.borrow().is::<T>() {
            // SAFETY: the data is of the correct type
            Some(unsafe { &mut *(self as *mut _ as *mut State<T>) })
        } else {
            None
        }
    }
}

struct StateInner<T: ?Sized> {
    dependents: RefCell<PtrWeakHashSet<WeakWidgetPtr>>,
    data: RefCell<T>,
}

pub enum Binding<T> {
    Constant(T),
    Func {
        result: T,
        update: Box<dyn Fn(&mut WidgetCtx, &mut T) -> bool>,
    },
}

impl<T: Clone> Binding<T> {
    pub const fn constant(value: T) -> Binding<T> {
        Binding::Constant(value)
    }

    pub fn value(&self) -> T {
        match self {
            Binding::Constant(v) => v.clone(),
            Binding::Func { result, .. } => result.clone(),
        }
    }

    pub fn value_ref(&self) -> &T {
        match self {
            Binding::Constant(v) => v,
            Binding::Func { result, .. } => result,
        }
    }

    pub fn update(&mut self, cx: &mut WidgetCtx) -> bool {
        match self {
            Binding::Constant(_) => false,
            Binding::Func { result, update } => update(cx, result),
        }
    }
}

/*
impl<T: Clone> From<T> for Binding<T> {
    fn from(value: T) -> Self {
        Binding::Constant(value)
    }
}*/

/*
impl<T, F> From<F> for Binding<T>
where
    T: Clone + Default,
    F: Fn(&mut TreeCtx, &mut T) -> bool + 'static,
{
    fn from(update: F) -> Self {
        Binding::Func {
            result: T::default(),
            update: Box::new(update),
        }
    }
}
*/

impl<T, F> From<F> for Binding<T>
where
    T: Clone + Default,
    F: Fn(&mut WidgetCtx) -> T + 'static,
{
    fn from(update: F) -> Self {
        Binding::Func {
            result: T::default(),
            update: Box::new(move |ctx, prev| {
                *prev = update(ctx);
                true
            }),
        }
    }
}

macro_rules! impl_binding_value {
    ($from:ty => $to:ty) => {
        impl From<$from> for Binding<$to> {
            fn from(v: $from) -> Binding<$to> {
                Binding::Constant(v.into())
            }
        }
    };
}

impl_binding_value!(f32 => f32);
impl_binding_value!(f64 => f64);
impl_binding_value!(i32 => i32);
impl_binding_value!(i64 => i64);
impl_binding_value!(u32 => u32);
impl_binding_value!(u64 => u64);
impl_binding_value!(bool => bool);
impl_binding_value!(String => String);
impl_binding_value!(&str => String);
impl_binding_value!(Point => Point);
impl_binding_value!(Vec2 => Vec2);
impl_binding_value!(TextSpan => TextSpan);
