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
    time::Duration,
};

use kurbo::{Affine, Point};
use skia_safe as sk;
use string_cache::Atom;
use tracing::{trace, warn};
use usvg::Tree;
use winit::{event::WindowEvent, event_loop::EventLoopWindowTarget, window::WindowId};

use crate::{
    application::{AppState, ExtEvent},
    composition::DrawableSurface,
    counter::Counter,
    drawing::ToSkia,
    utils::{WidgetSet, WidgetSlice},
    widget::WidgetPtr,
    window::UiHostWindowHandler,
    BoxConstraints, ChangeFlags, Event, Geometry, Widget, WidgetId,
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
/*
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
    paths: &WidgetSlice,
    visitor: &mut WidgetVisitor,
) {
    if paths.is_empty() {
        return;
    }

    let mut cx = TreeCtx::new(app_state, event_loop);
    cx.dispatch_root(root, paths, visitor);
    let pending_events = cx.pending_events.take();
    let pending_updates = cx.pending_updates.take();

    // Dispatch events
    for (targets, event_kind) in pending_events.iter() {
        // TODO remove the need for cloning here
        let mut event = Event::new(event_kind.clone());
        root_tree_dispatch(
            app_state,
            event_loop,
            root,
            targets,
            &mut |cx: &mut TreeCtx, widget: &mut dyn Widget| {
                if !event.handled {
                    widget.event(cx, &mut event);
                }
            },
        );
    }

    // Handle follow-up updates
    // In case there's an infinite update loop, this will end in a stack overflow.
    root_tree_dispatch(
        app_state,
        event_loop,
        root,
        &pending_updates,
        &mut |cx: &mut TreeCtx, widget: &mut dyn Widget| {
            widget.update(cx);
        },
    );
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////
