//! Cache context API.
//!
//! Functions available within a caching context (see `Cache::run`).
use crate::{
    cache::{CacheContext, CacheVar},
    call_id::CallId,
};
use kyute_common::Data;
use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    panic::Location,
    rc::Rc,
};

/// Context stored in TLS when running a function within the positional cache.
pub(crate) struct CacheContextTLS {
    pub(crate) cx: CacheContext,
    //env: Environment,
}

thread_local! {
    // The cache context is put in TLS so that we don't have to pass an additional parameter
    // to all functions.
    // A less hack-ish solution would be to rewrite composable function calls, but we need
    // more than a proc macro to be able to do that (must resolve function paths and rewrite call sites)
    // The cache context lives on the main thread.
    pub(crate) static CACHE_CONTEXT: RefCell<Option<CacheContextTLS>> = RefCell::new(None);
}

//==================================================================================================
// Cache context API

fn with_cache_cx<R>(f: impl FnOnce(&mut CacheContext) -> R) -> R {
    CACHE_CONTEXT.with(|cx_cell| {
        let mut cx = cx_cell.borrow_mut();
        if let Some(ref mut cx) = &mut *cx {
            f(&mut cx.cx)
        } else {
            panic!("this function cannot be called outside of recomposition");
        }
    })
}

/// Returns the current call-trace identifier.
pub fn current_call_id() -> CallId {
    with_cache_cx(|cx| cx.caller_id())
}

/// Returns the current cache revision.
pub fn revision() -> usize {
    with_cache_cx(|cx| cx.revision())
}

/// Must be called inside `Cache::run`.
#[track_caller]
pub fn enter_call(index: impl Hash) {
    let mut hasher = DefaultHasher::new();
    index.hash(&mut hasher);
    // FIXME Hash implementations do not guarantee reasonable uniqueness
    enter_call_by_index(hasher.finish() as usize);
}

/// Must be called inside `Cache::run`.
#[track_caller]
fn enter_call_by_index(index: usize) {
    let location = Location::caller();
    with_cache_cx(move |cx| cx.enter_call_scope(location, index));
}

/// Must be called inside `Cache::run`.
pub fn exit_call() {
    with_cache_cx(move |cx| cx.exit_call_scope());
}

/// Must be called inside `Cache::run`.
#[track_caller]
pub fn scoped<R>(index: impl Hash, f: impl FnOnce() -> R) -> R {
    enter_call(index);
    let r = f();
    exit_call();
    r
}

#[track_caller]
pub fn changed<T: Data>(value: T) -> bool {
    let location = Location::caller();
    with_cache_cx(move |cx| {
        cx.enter_call_scope(location, 0);
        let changed = cx.compare_and_update(value);
        cx.exit_call_scope();
        changed
    })
}

#[track_caller]
pub fn variable<T: 'static, Init: FnOnce() -> T>(init: Init) -> (Rc<CacheVar<T>>, bool) {
    let location = Location::caller();
    with_cache_cx(move |cx| {
        cx.enter_call_scope(location, 0);
        let r = cx.enter_var(init);
        cx.exit_var();
        cx.exit_call_scope();
        r
    })
}

#[track_caller]
pub fn enter_variable<T: 'static, Init: FnOnce() -> T>(init: Init) -> (Rc<CacheVar<T>>, bool) {
    let location = Location::caller();
    with_cache_cx(move |cx| {
        cx.enter_call_scope(location, 0);
        cx.enter_var(init)
    })
}

pub fn exit_variable() {
    with_cache_cx(move |cx| {
        cx.exit_var();
        cx.exit_call_scope();
    })
}

/// Runs the function only once at the call site and caches the result (like memoize without parameters).
/// TODO better docs
#[track_caller]
pub fn once<T: Clone + 'static>(f: impl FnOnce() -> T) -> T {
    variable(f).0.get()
}

/// Memoizes the result of a function at this call site.
#[track_caller]
pub fn memoize<Args: Data, T: Clone + 'static>(args: Args, f: impl FnOnce() -> T) -> T {
    let location = Location::caller();

    let (result_var, dirty) = with_cache_cx(move |cx| {
        cx.enter_call_scope(location, 0);
        let args_changed = cx.compare_and_update(args);
        let (result_var, _) = cx.enter_var(|| None);
        let dirty = result_var.is_dirty() || args_changed;
        if args_changed {
            eprintln!("memoize: recomputing because arguments have changed");
        }
        if result_var.is_dirty() {
            eprintln!("memoize: recomputing because state entry is dirty");
        }
        (result_var, dirty)
    });

    if dirty {
        let result = f();
        result_var.replace(Some(result), false);
    } else {
        eprintln!("memoize: clean");
        skip_to_end_of_group();
    }

    with_cache_cx(|cx| {
        cx.exit_var();
        cx.exit_call_scope();
    });

    // make the parent variable dependent on this value
    result_var.set_dependency();
    result_var.get().expect("memoize: no value calculated")
}

/// Runs the function if it has changed, otherwise returns `Default::default`.
#[track_caller]
pub fn run_if_changed<Args, T>(args: Args, f: impl FnOnce() -> T) -> Option<T>
where
    Args: Data,
{
    if changed(args) {
        Some(f())
    } else {
        None
    }
}

/*/// TODO document
#[track_caller]
pub fn run_async<T, Fut>(future: Fut, restart: bool) -> Poll<T>
where
    T: Clone + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
{
    struct AsyncTaskEntry {
        handle: tokio::task::JoinHandle<()>,
        revision: usize,
    }

    let task_key = state::<Option<AsyncTaskEntry>, _>(|| None);

    // if we requested a restart, abort the current running task
    let mut task = task_key.take_without_invalidation();

    let revision = if let Some(ref mut task) = task {
        if restart {
            trace!("run_async: restarting task");
            task.handle.abort();
            task.revision += 1;
            task.revision
        } else {
            task.revision
        }
    } else {
        0
    };

    let CacheEntryInsertResult {
        key: result_key,
        inserted,
        ..
    } = scoped(revision, || state_inner(|| Poll::Pending));

    if inserted || restart {
        with_cache_cx(|_cx| {
            let result_key_2 = result_key.clone();
            // spawn task that will set the value
            let handle = tokio::spawn(async move {
                let result = future.await;
                result_key_2.set(Poll::Ready(result));
            });

            task = Some(AsyncTaskEntry { handle, revision })
        });
    }

    task_key.set_without_invalidation(task);
    result_key.get()
}*/

pub fn skip_to_end_of_group() {
    with_cache_cx(|cx| {
        cx.skip_until_end_of_group();
    })
}

/*/// Memoizes the result of a function at this call site.
#[track_caller]
pub fn memoize<Args: Data, T: Clone + 'static>(args: Args, f: impl FnOnce() -> T) -> T {
    // We expect two cache entries here:
    // - the cached arguments
    // - the result

    group(move || {
        let (result_key, args_changed, dirty) = with_cache_cx(move |cx| {
            let args_changed = cx.writer.compare_and_update(args);
            let CacheEntryInsertResult {
                key: result_key, dirty, ..
            } = cx.writer.get_or_insert_entry(|| None);
            /*if args_changed {
                trace!("memoize: recomputing because arguments have changed {:#?}", call_node);
            }
            if dirty {
                trace!("memoize: recomputing because state entry is dirty {:#?}", call_node);
            }*/
            (result_key, args_changed, dirty)
        });
        let result_dirty = args_changed || dirty;

        if result_dirty {
            with_cache_cx(|cx| {
                cx.writer.start_state(&result_key);
            });
            let result = f();
            with_cache_cx(|cx| {
                cx.writer.end_state();
            });
            result_key.set_without_invalidation(Some(result));
        } else {
            skip_to_end_of_group();
        }

        // it's important to call `get()` in all circumstances to make the parent state entry
        // dependent on this value.
        result_key.get().expect("memoize: no value calculated")
    })
}
*/
