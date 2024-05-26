//! Widget tree manipulation and traversal.

use std::{
    any::{Any, TypeId},
    cell::{Cell, Ref, RefCell},
    ffi::c_void,
    fmt,
    hash::Hash,
    mem,
    ops::{Deref, DerefMut},
    panic::Location,
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
use kurbo::{Affine, Point, Rect, Vec2};
use skia_safe as sk;
use weak_table::{
    traits::{WeakElement, WeakKey},
    WeakKeyHashMap,
};
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

    pub fn with_clip_rect(&mut self, rect: Rect, f: impl FnOnce(&mut PaintCtx<'a>)) {
        let mut surface = self.surface.surface();
        surface.canvas().save();
        surface.canvas().clip_rect(rect.to_skia(), sk::ClipOp::Intersect, false);
        f(self);
        surface.canvas().restore();
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
    fn mount(&mut self, cx: &mut WidgetCtx<Self>)
    where
        Self: Sized;

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
    fn update(&mut self, cx: &mut WidgetCtx<Self>)
    where
        Self: Sized,
    {
    }

    fn environment(&self) -> Environment {
        Environment::new()
    }

    /// Event handling.
    fn event(&mut self, cx: &mut WidgetCtx<Self>, event: &mut Event)
    where
        Self: Sized,
    {
    }

    /// Hit-testing.
    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool;

    /// Layout.
    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry;

    /// Raw window events.
    fn window_event(&mut self, _cx: &mut WidgetCtx<Self>, _event: &winit::event::WindowEvent, _time: Duration)
    where
        Self: Sized,
    {
    }

    fn paint(&mut self, cx: &mut PaintCtx);
}

pub trait AnyWidget: Widget {
    /// # Safety
    ///
    /// `this` must be the `Rc<WidgetPod>` containing the widget (`&mut self`).
    unsafe fn mount(&mut self, this: WidgetPtrAny, cx: &mut Ctx);
    unsafe fn update(&mut self, this: WidgetPtrAny, cx: &mut Ctx);
    unsafe fn event(&mut self, this: WidgetPtrAny, cx: &mut Ctx, event: &mut Event);
    unsafe fn window_event(
        &mut self,
        this: WidgetPtrAny,
        cx: &mut Ctx,
        event: &winit::event::WindowEvent,
        time: Duration,
    );
}

impl<W: Widget> AnyWidget for W {
    unsafe fn mount(&mut self, this: WidgetPtrAny, cx: &mut Ctx) {
        WidgetCtx::with(cx, this.downcast_unchecked(), |cx| {
            self.mount(cx);
        });
    }

    unsafe fn update(&mut self, this: WidgetPtrAny, cx: &mut Ctx) {
        WidgetCtx::with(cx, this.downcast_unchecked(), |cx| {
            self.update(cx);
        });
    }

    unsafe fn event(&mut self, this: WidgetPtrAny, cx: &mut Ctx, event: &mut Event) {
        WidgetCtx::with(cx, this.downcast_unchecked(), |cx| {
            self.event(cx, event);
        });
    }

    unsafe fn window_event(&mut self, this: WidgetPtrAny, cx: &mut Ctx, event: &WindowEvent, time: Duration) {
        WidgetCtx::with(cx, this.downcast_unchecked(), |cx| {
            self.window_event(cx, event, time);
        });
    }
}

pub type WidgetPtr<T> = Rc<WidgetPod<T>>;
pub type WeakWidgetPtr<T> = Weak<WidgetPod<T>>;
pub type WidgetPtrAny = Rc<WidgetPod<dyn AnyWidget>>;
pub type WeakWidgetPtrAny = Weak<WidgetPod<dyn AnyWidget>>;

/*
/// Proxy trait to support trait objects on `Widget`.
pub trait AnyWidgetPod {
    fn dyn_mount(self: Rc<Self>, cx: &mut Ctx, parent: WeakWidgetPtrAny);
    fn dyn_update(self: Rc<Self>, cx: &mut Ctx);
    fn dyn_event(self: Rc<Self>, cx: &mut Ctx, event: &mut Event);
    fn dyn_window_event(self: Rc<Self>, cx: &mut Ctx, event: &winit::event::WindowEvent, time: Duration);
    fn environment(&self) -> Environment;
    fn dyn_hit_test(self: Rc<Self>, result: &mut HitTestResult, position: Point) -> bool;
    fn layout(&self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry;
    fn paint(&self, cx: &mut PaintCtx);
}

// impl AnyWidgetPod for WidgetPod<dyn Widget>

impl dyn AnyWidgetPod {
    pub fn mount(self: &Rc<Self>, parent_cx: &mut WidgetCtx<impl Widget>) {
        let parent = Rc::downgrade(&parent_cx.current_widget);
        AnyWidgetPod::dyn_mount(self.clone(), parent_cx.base, parent)
    }

    pub fn mount_root(self: &Rc<Self>, cx: &mut Ctx) {
        AnyWidgetPod::dyn_mount(self.clone(), cx, WeakWidgetPtr::<Null>::default());
    }

    pub fn update(self: &Rc<Self>, cx: &mut Ctx) {
        AnyWidgetPod::dyn_update(self.clone(), cx);
    }

    pub fn event(self: &Rc<Self>, cx: &mut Ctx, event: &mut Event) {
        AnyWidgetPod::dyn_event(self.clone(), cx, event);
    }

    pub fn window_event(self: &Rc<Self>, cx: &mut Ctx, event: &winit::event::WindowEvent, time: Duration) {
        AnyWidgetPod::dyn_window_event(self.clone(), cx, event, time);
    }

    pub fn hit_test(self: &Rc<Self>, result: &mut HitTestResult, position: Point) -> bool {
        AnyWidgetPod::dyn_hit_test(self.clone(), result, position)
    }
}*/

impl<W: Widget + ?Sized> WidgetPod<W> {}

/// Container for widgets.
pub struct WidgetPod<T: ?Sized = dyn AnyWidget> {
    mounted: Cell<bool>,
    parent: RefCell<WeakWidgetPtrAny>,
    environment: RefCell<Environment>,
    pub widget: RefCell<T>,
}

impl<W> WidgetPod<W> {
    pub fn new(widget: W) -> WidgetPtr<W> {
        Rc::new(WidgetPod {
            mounted: Cell::new(false),
            // Weak::new() doesn't work with unsized types, so use the dummy Null widgets (https://github.com/rust-lang/rust/issues/50513)
            parent: RefCell::new(WeakWidgetPtr::<Null>::new()),
            environment: Default::default(),
            widget: RefCell::new(widget),
        })
    }

    pub fn environment(&self) -> Environment {
        self.environment.borrow().clone()
    }
}

impl WidgetPod {
    pub unsafe fn downcast_unchecked<T: Widget + 'static>(self: Rc<Self>) -> WidgetPtr<T> {
        Rc::from_raw(Rc::into_raw(self) as *const WidgetPod<T>)
    }

    pub fn downcast<W: Widget + 'static>(self: Rc<Self>) -> Option<WidgetPtr<W>> {
        if self.widget.borrow().deref().type_id() == TypeId::of::<W>() {
            Some(unsafe { self.downcast_unchecked() })
        } else {
            None
        }
    }

    fn mount_inner(self: &Rc<Self>, cx: &mut Ctx, parent: WeakWidgetPtrAny) {
        if !self.mounted.get() {
            self.mounted.set(true);
            let env = self.widget.borrow().environment().union(cx.environment.clone());
            self.environment.replace(env);
            self.parent.replace(parent);
        }
        unsafe {
            AnyWidget::mount(&mut *self.widget.borrow_mut(), self.clone(), cx);
        }
    }

    pub fn dyn_mount_root(self: &Rc<Self>, cx: &mut Ctx) {
        self.mount_inner(cx, WeakWidgetPtr::<Null>::default());
    }

    pub fn dyn_mount(self: &Rc<Self>, cx: &mut WidgetCtx<impl Widget>) {
        let parent = Rc::downgrade(&cx.current_widget);
        self.mount_inner(cx, parent);
    }

    pub fn dyn_update(self: &Rc<Self>, cx: &mut Ctx) {
        unsafe {
            AnyWidget::update(&mut *self.widget.borrow_mut(), self.clone(), cx);
        }
    }

    pub fn dyn_event(self: &Rc<Self>, cx: &mut Ctx, event: &mut Event) {
        unsafe {
            AnyWidget::event(&mut *self.widget.borrow_mut(), self.clone(), cx, event);
        }
    }

    pub fn dyn_window_event(self: &Rc<Self>, cx: &mut Ctx, event: &winit::event::WindowEvent, time: Duration) {
        unsafe {
            AnyWidget::window_event(&mut *self.widget.borrow_mut(), self.clone(), cx, event, time);
        }
    }

    pub fn dyn_hit_test(self: &Rc<Self>, result: &mut HitTestResult, position: Point) -> bool {
        let hit = self.widget.borrow_mut().hit_test(result, position);
        if hit {
            result.add(self.clone());
        }
        hit
    }
}

impl<W: Widget> WidgetPod<W> {
    pub fn as_dyn(self: &Rc<Self>) -> WidgetPtrAny {
        self.clone()
    }

    pub fn call_with_cx(self: &Rc<Self>, ctx: &mut Ctx, f: impl FnOnce(&mut W, &mut WidgetCtx<W>)) {
        WidgetCtx::with(ctx, self.clone(), |cx| {
            f(&mut *self.widget.borrow_mut(), cx);
        });
    }

    pub fn mount_root(self: &Rc<Self>, cx: &mut Ctx) {
        // TODO we can implement this without downcasts, but for now defer to the dyn implementation
        self.as_dyn().dyn_mount_root(cx);
    }

    pub fn mount(self: &Rc<Self>, cx: &mut WidgetCtx<impl Widget>) {
        self.as_dyn().dyn_mount(cx);
    }

    pub fn update(self: &Rc<Self>, cx: &mut Ctx) {
        WidgetCtx::with(cx, self.clone(), |cx| {
            self.widget.borrow_mut().update(cx);
        });
    }

    pub fn event(self: &Rc<Self>, cx: &mut Ctx, event: &mut Event) {
        WidgetCtx::with(cx, self.clone(), |cx| {
            self.widget.borrow_mut().event(cx, event);
        });
    }

    pub fn window_event(self: &Rc<Self>, cx: &mut Ctx, event: &winit::event::WindowEvent, time: Duration) {
        WidgetCtx::with(cx, self.clone(), |cx| {
            self.widget.borrow_mut().window_event(cx, event, time);
        });
    }

    pub fn hit_test(self: &Rc<Self>, result: &mut HitTestResult, position: Point) -> bool {
        self.as_dyn().dyn_hit_test(result, position)
    }
}

impl<W: Widget + ?Sized> WidgetPod<W> {
    pub fn layout(&self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        self.widget.borrow_mut().layout(cx, bc)
    }

    pub fn paint(&self, cx: &mut PaintCtx) {
        self.widget.borrow_mut().paint(cx);
    }
}

impl fmt::Debug for WidgetPod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetPod").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct HitTestEntry {
    pub widget: WidgetPtrAny,
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

    pub fn add(&mut self, widget: WidgetPtrAny) {
        self.hits.push(HitTestEntry {
            widget,
            transform: self.current_transform,
        });
    }
}

/// Context passed during tree traversals.
pub struct Ctx<'a> {
    pub app_state: &'a mut AppState,
    pub event_loop: &'a EventLoopWindowTarget<ExtEvent>,
    /// Widgets that need updating after the current dispatch.
    pending_callbacks: RefCell<Vec<Rc<dyn Fn(&mut Ctx)>>>,
    /// Whether relayout is necessary.
    relayout: bool,
    environment: Environment,
}

impl<'a> Ctx<'a> {
    /// Creates the root TreeCtx.
    pub(crate) fn new(
        app_state: &'a mut AppState,
        event_loop: &'a EventLoopWindowTarget<ExtEvent>,
        environment: Environment,
    ) -> Ctx<'a> {
        Ctx {
            app_state,
            event_loop,
            pending_callbacks: RefCell::new(Default::default()),
            relayout: false,
            environment,
        }
    }

    pub fn queue_callback(&mut self, callback: Rc<dyn Fn(&mut Ctx)>) {
        self.pending_callbacks.borrow_mut().push(callback);
    }

    pub fn dispatch_queued_callbacks(&mut self) {
        while self.pending_callbacks.borrow().len() > 0 {
            let mut cbs = self.pending_callbacks.take();
            for callback in cbs.drain(..) {
                callback(self);
            }
        }
    }

    pub fn mark_needs_layout(&mut self) {
        self.relayout = true;
    }

    #[must_use]
    pub fn needs_layout(&self) -> bool {
        self.relayout
    }

    pub fn env<T: EnvValue>(&self) -> Option<T> {
        self.environment.get::<T>()
    }
}

pub struct WidgetCtx<'a, 'b, W> {
    base: &'b mut Ctx<'a>,
    current_widget: WidgetPtr<W>,
}

impl<'a, 'b, W> Deref for WidgetCtx<'a, 'b, W> {
    type Target = Ctx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a, 'b, W> DerefMut for WidgetCtx<'a, 'b, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'a, 'b, W: Widget> WidgetCtx<'a, 'b, W> {
    pub fn with<R>(
        parent_cx: &'b mut Ctx<'a>,
        current_widget: WidgetPtr<W>,
        f: impl FnOnce(&mut WidgetCtx<W>) -> R,
    ) -> R {
        let prev_env = mem::replace(&mut parent_cx.environment, current_widget.environment.borrow().clone());
        let mut ctx = WidgetCtx {
            base: parent_cx,
            current_widget,
        };
        let r = f(&mut ctx);
        parent_cx.environment = prev_env;
        r
    }

    pub fn current(&self) -> WidgetPtr<W> {
        self.current_widget.clone()
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
            .base
            .app_state
            .windows
            .insert(window_id, self.current_widget.clone())
            .is_some()
        {
            panic!("window {window_id:?} already registered");
        }
    }
}

/// A widget that builds a widget given a TreeCtx
pub struct Builder<W> {
    f: Box<dyn FnMut(&mut WidgetCtx<Self>) -> W>,
    inner: Option<WidgetPtrAny>,
}

impl<W> Builder<W> {
    pub fn new(f: impl FnMut(&mut WidgetCtx<Self>) -> W + 'static) -> Builder<W> {
        Builder {
            f: Box::new(f),
            inner: None,
        }
    }
}

impl<W> Widget for Builder<W>
where
    W: Widget + 'static,
{
    fn mount(&mut self, cx: &mut WidgetCtx<Self>) {
        self.inner = {
            let widget: WidgetPtrAny = WidgetPod::new((self.f)(cx));
            widget.dyn_mount(cx);
            Some(widget)
        };
    }

    fn update(&mut self, cx: &mut WidgetCtx<Self>) {
        self.inner = {
            let widget: WidgetPtrAny = WidgetPod::new((self.f)(cx));
            widget.dyn_update(cx);
            cx.mark_needs_layout();
            Some(widget)
        };
    }

    fn event(&mut self, cx: &mut WidgetCtx<Self>, event: &mut Event) {}

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        if let Some(ref inner) = self.inner {
            inner.dyn_hit_test(result, position)
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
            //dependents: Default::default(),
            callbacks: Default::default(),
            data: RefCell::new(data),
        }))
    }

    /*pub fn set_dependency<W>(&self, cx: &Ctx<W>) {
        self.0.dependents.borrow_mut().insert(cx.current());
    }

    pub fn track<W>(&self, cx: &Ctx<W>) {
        self.set_dependency(cx);
    }*/

    fn notify(&self, cx: &mut Ctx) {
        let mut callbacks = self.0.callbacks.borrow_mut();
        for callback in callbacks.values() {
            cx.queue_callback(callback.clone());
        }
    }

    /// Modifies the state and notify the dependents.
    pub fn set(&self, cx: &mut Ctx, value: T) {
        self.0.data.replace(value);
        self.notify(cx);
    }

    /// Modifies the state and notify the dependents.
    pub fn update<R>(&self, cx: &mut Ctx, f: impl FnOnce(&mut T) -> R) -> R {
        let mut data = self.0.data.borrow_mut();
        let r = f(&mut *data);
        self.notify(cx);
        r
    }

    /*/// Returns the current value of the state, setting a dependency on the current widget.
    pub fn get(&self, cx: &mut Ctx) -> Ref<T> {
        self.set_dependency(cx);
        self.0.data.borrow()
    }*/

    pub fn get(&self) -> Ref<T> {
        self.0.data.borrow()
    }

    pub fn get_tracked(&self, cx: &mut WidgetCtx<impl Widget>) -> Ref<T> {
        // FIXME: this keeps adding more and more callbacks that are never cleaned up

        // We need to only add the callback when the same (receiver, method) pair isn't present in the
        // callbacks list. The problem is that we have no way of telling whether two `&dyn Fn(&mut Ctx)`
        // represent the same callback (closures have no identity and can't be compared).
        //
        // Alternatives:
        // - store both the WeakWidgetPtr *and* the location where the callback was added (with #[track_caller]); compare locations
        self.watch(cx, Widget::update);
        self.0.data.borrow()
    }

    pub fn at(cx: &mut WidgetCtx<impl Widget>) -> Option<T>
    where
        T: Clone,
    {
        cx.env::<State<T>>().map(|state| state.get_tracked(cx).clone())
    }

    #[track_caller]
    pub fn watch<W, F>(&self, cx: &mut WidgetCtx<W>, f: F)
    where
        W: Widget,
        F: Fn(&mut W, &mut WidgetCtx<W>) + 'static,
    {
        let location = Location::caller();
        let target = Rc::downgrade(&cx.current());
        self.0.callbacks.borrow_mut().insert(
            CallbackStrongKey(location, cx.current()),
            Rc::new(move |ctx| {
                if let Some(target) = target.upgrade() {
                    target.call_with_cx(ctx, &f)
                }
            }),
        );
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

struct CallbackWeakKey(&'static Location<'static>, WeakWidgetPtrAny);
struct CallbackStrongKey(&'static Location<'static>, WidgetPtrAny);

impl WeakElement for CallbackWeakKey {
    type Strong = CallbackStrongKey;

    fn new(view: &Self::Strong) -> Self {
        CallbackWeakKey(view.0, Rc::downgrade(&view.1))
    }

    fn view(&self) -> Option<Self::Strong> {
        if let Some(strong) = self.1.upgrade() {
            Some(CallbackStrongKey(self.0, strong))
        } else {
            None
        }
    }
}

impl WeakKey for CallbackWeakKey {
    type Key = (&'static Location<'static>, *const WidgetPod<dyn AnyWidget>);

    fn with_key<F, R>(view: &Self::Strong, f: F) -> R
    where
        F: FnOnce(&Self::Key) -> R,
    {
        f(&(view.0, Rc::as_ptr(&view.1)))
    }
}

/*
struct Callback {
    key: CallbackWeakKey,
    function: Rc<dyn Fn(&mut Ctx)>,
}*/

struct StateInner<T: ?Sized> {
    callbacks: RefCell<WeakKeyHashMap<CallbackWeakKey, Rc<dyn Fn(&mut Ctx)>>>,
    data: RefCell<T>,
}

pub enum Binding<T> {
    Constant(T),
    Func {
        result: T,
        update: Box<dyn Fn(&mut Ctx, &mut T) -> bool>,
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

    pub fn update(&mut self, cx: &mut Ctx) -> bool {
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
    F: Fn(&mut Ctx) -> T + 'static,
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
