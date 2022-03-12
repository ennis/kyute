//! GUI positional cache.
use crate::{
    application::ExtEvent,
    call_id::{CallId, CallIdStack, CallNode},
    composable, Data, Environment,
};
use parking_lot::Mutex;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    convert::TryInto,
    fmt,
    future::Future,
    hash::Hash,
    mem,
    panic::Location,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Poll, Waker},
};
use threadbound::ThreadBound;

slotmap::new_key_type! {
    struct KeyInner;
}

#[derive(Debug)]
struct DepNode {
    dirty: AtomicBool,
    dependents: Mutex<Vec<Arc<DepNode>>>,
}

impl DepNode {
    fn new() -> DepNode {
        DepNode {
            dirty: AtomicBool::new(false),
            dependents: Mutex::new(vec![]),
        }
    }

    fn set_dirty(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    fn invalidate_dependents(&self) {
        for d in self.dependents.lock().iter() {
            d.invalidate()
        }
    }

    fn add_dependent(&self, dep: &Arc<DepNode>) {
        let mut deps = self.dependents.lock();
        for d in deps.iter() {
            if Arc::ptr_eq(d, dep) {
                return;
            }
        }
        deps.push(dep.clone());
    }

    fn invalidate(&self) {
        self.set_dirty(true);
        self.invalidate_dependents();
    }
}

/// Entry representing a mutable state slot inside a composition cache.
struct StateCell<T: ?Sized = dyn Any> {
    call_id: CallId,
    // for debugging
    call_node: Option<Arc<CallNode>>,
    dep_node: Arc<DepNode>,
    waker: Waker,
    value: Mutex<T>,
}

impl<T: ?Sized> fmt::Debug for StateCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StateCell").finish_non_exhaustive()
    }
}

impl StateCell {
    fn downcast<U: Any + 'static>(self: Arc<Self>) -> Result<Arc<StateCell<U>>, Arc<StateCell>> {
        if (*self).value.lock().is::<U>() {
            unsafe { Ok(Arc::from_raw(Arc::into_raw(self) as *const StateCell<U>)) }
        } else {
            Err(self)
        }
    }
}

impl<T: ?Sized> StateCell<T> {
    fn update_dependents(&self) {
        if let Some(var) = parent_state() {
            self.dep_node.add_dependent(&var.dep_node);
        }
    }
}

impl<T: 'static> StateCell<T> {
    fn replace(&self, new_value: T, invalidate: bool) -> T {
        self.update_dependents();
        let mut value = self.value.lock();
        let ret = mem::replace(&mut *value, new_value);
        if invalidate {
            self.dep_node.invalidate_dependents();
            self.waker.wake_by_ref();
        }
        ret
    }
}

impl<T: Clone + 'static> StateCell<T> {
    fn get(&self) -> T {
        self.update_dependents();
        self.value.lock().clone()
    }
}

impl<T: Data> StateCell<T> {
    fn update(&self, new_value: T) -> Option<T> {
        self.update_dependents();
        let mut value = self.value.lock();
        if !new_value.same(&*value) {
            let ret = mem::replace(&mut *value, new_value);
            self.dep_node.invalidate_dependents();
            self.waker.wake_by_ref();
            Some(ret)
        } else {
            None
        }
    }
}

/// A key used to access a state variable stored in a `Cache`.
pub struct State<T>(Arc<StateCell<T>>);

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        State(self.0.clone())
    }
}

impl<T> fmt::Debug for State<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T: 'static> State<T> {
    /// Returns the value of the cache entry and replaces it by the given value.
    /// Always invalidates.
    /// Can be called outside of recomposition.
    pub fn replace(&self, new_value: T) -> T {
        self.0.replace(new_value, true)
    }

    /// Returns the value of the cache entry and replaces it by the default value.
    /// Does not invalidate the dependent entries.
    pub fn replace_without_invalidation(&self, new_value: T) -> T {
        self.0.replace(new_value, false)
    }

    pub fn set(&self, new_value: T) {
        // TODO idea: log the call sites that invalidated the cache, for debugging
        // e.g. `state entry @ (call site) invalidated because of (state entries), because of manual invalidation @ (call site) OR invalidated externally`
        self.replace(new_value);
    }

    pub fn set_without_invalidation(&self, new_value: T) {
        // TODO idea: log the call sites that invalidated the cache, for debugging
        // e.g. `state entry @ (call site) invalidated because of (state entries), because of manual invalidation @ (call site) OR invalidated externally `
        self.replace_without_invalidation(new_value);
    }
}

impl<T: Data + 'static> State<T> {
    ///
    pub fn update(&self, new_value: T) -> Option<T> {
        self.0.update(new_value)
    }
}

impl<T: Clone + 'static> State<T> {
    pub fn get(&self) -> T {
        self.0.get()
    }
}

impl<T: Default + 'static> State<T> {
    /// Returns the value of the cache entry and replaces it by the default value.
    pub fn take(&self) -> T {
        self.replace(T::default())
    }

    /// Returns the value of the cache entry and replaces it by the default value. Does not invalidate dependent entries.
    pub fn take_without_invalidation(&self) -> T {
        self.replace_without_invalidation(T::default())
    }
}

/// A slot in the slot table.
enum Slot {
    /// Marks the start of a group.
    /// Contains the length of the group including this slot and the `GroupEnd` marker.
    StartGroup {
        call_id: CallId,
        len: u32,
    },
    /// Marks the end of a scope.
    EndGroup,
    Value {
        var: Arc<StateCell>,
    },
}

/// Composition cache. Contains the recorded call tree and state entries.
struct CacheInner {
    waker: Waker,
    /// The call tree, represented as an array of slots.
    slots: Vec<Slot>,
    /// The number of times `Cache::run` has been called.
    revision: usize,
}

impl CacheInner {
    fn new(waker: Waker) -> CacheInner {
        CacheInner {
            waker,
            slots: vec![],
            revision: 0,
        }
    }

    //fn create_state_proxy(&self, key: Key<T>) ->

    fn dump(&self, current_position: usize) {
        for (i, s) in self.slots.iter().enumerate() {
            if i == current_position {
                eprint!("* ");
            } else {
                eprint!("  ");
            }
            match s {
                Slot::StartGroup { call_id, len } => {
                    eprintln!(
                        "{:3} StartGroup call_id={:?} len={} (end={})",
                        i,
                        call_id,
                        *len,
                        i + *len as usize - 1,
                    )
                }
                Slot::EndGroup => {
                    eprintln!("{:3} EndGroup", i)
                }
                Slot::Value { var } => {
                    let call_id = var.call_id;
                    if let Some(ref node) = var.call_node {
                        eprintln!(
                            "{:3} Value      call_id={:?} dirty={:?} [{}]",
                            i,
                            call_id,
                            var.dep_node.is_dirty(),
                            node.location
                        )
                    } else {
                        eprintln!(
                            "{:3} Value      call_id={:?} dirty={:?}",
                            i,
                            call_id,
                            var.dep_node.is_dirty()
                        )
                    }
                }
            }
        }
    }
}

struct CacheEntryInsertResult<T> {
    key: State<T>,
    dirty: bool,
    inserted: bool,
}

/// Holds the state during cache updates (`Cache::run`).
struct CacheWriter {
    cache: CacheInner,
    /// Current position in the slot table (`self.cache.slots`)
    pos: usize,
    /// Stack of call IDs.
    id_stack: CallIdStack,
    /// Stack of group start positions.
    /// The top element is the start of the current group.
    group_stack: Vec<usize>,
    /// Stack of dependent state variables.
    state_stack: Vec<Arc<StateCell<dyn Any>>>,
}

impl CacheWriter {
    fn new(root_location: &'static Location<'static>, cache: CacheInner) -> CacheWriter {
        let mut writer = CacheWriter {
            cache,
            pos: 0,
            id_stack: CallIdStack::new(),
            group_stack: vec![],
            state_stack: vec![],
        };

        writer.enter_scope(root_location, 0);
        writer.start_group();
        // setup root cache entry
        // the root cache entry is there to detect whether any state entry in the cache
        // has been invalidated during invalidation
        let CacheEntryInsertResult {
            key: root_cache_key, ..
        } = writer.get_or_insert_entry(|| ());
        writer.state_stack.push(root_cache_key.0.clone());
        root_cache_key.0.dep_node.set_dirty(false);
        writer
    }

    fn finish(mut self) -> (CacheInner, bool) {
        let root_state_key = self.state_stack.pop().expect("unbalanced state scopes");
        let should_rerun = root_state_key.dep_node.is_dirty();
        if should_rerun {
            trace!("cache: state entry invalidated, will re-run");
        }
        self.end_group();
        self.exit_scope();
        assert!(self.group_stack.is_empty(), "unbalanced groups");
        assert!(self.id_stack.is_empty(), "unbalanced scopes");
        // may not be true anymore (we can have multiple roots on a single cache)
        //assert_eq!(self.pos, self.cache.slots.len());
        (self.cache, should_rerun)
    }

    fn enter_scope(&mut self, location: &'static Location<'static>, index: usize) {
        self.id_stack.enter(location, index);
    }

    fn exit_scope(&mut self) {
        self.id_stack.exit();
    }

    fn start_state<T: 'static>(&mut self, key: &State<T>) {
        self.state_stack.push(key.0.clone());
        key.0.dep_node.set_dirty(false);
    }

    fn end_state(&mut self) {
        self.state_stack.pop().expect("unbalanced state scopes");
    }

    /// Finds a slot with the specified key in the current group, starting from the current position.
    ///
    /// # Return value
    ///
    /// The position of the matching slot in the table, or None.
    fn find_call_id(&self, call_id: CallId) -> Option<usize> {
        let mut i = self.pos;
        let slots = &self.cache.slots[..];

        while i < self.cache.slots.len() {
            match slots[i] {
                Slot::StartGroup {
                    call_id: this_call_id,
                    len,
                    ..
                } => {
                    if this_call_id == call_id {
                        return Some(i);
                    }
                    i += len as usize;
                }
                Slot::Value { ref var } if var.call_id == call_id => {
                    return Some(i);
                }
                Slot::EndGroup => {
                    // reached the end of the current group
                    return None;
                }
                _ => {
                    i += 1;
                }
            }
        }

        // no slot found
        None
    }

    fn rotate_in_current_position(&mut self, pos: usize) {
        assert!(pos >= self.pos);
        let group_end_pos = self.group_end_position();
        assert!(pos <= group_end_pos);
        self.cache.slots[self.pos..group_end_pos].rotate_left(pos - self.pos);
    }

    /// TODO docs
    fn sync(&mut self) -> bool {
        let call_id = self.id_stack.current();
        let pos = self.find_call_id(call_id);
        match pos {
            Some(pos) => {
                // move slots in position
                self.rotate_in_current_position(pos);
                true
            }
            None => false,
        }
    }

    fn start_group(&mut self) {
        //let parent = self.parent_entry_key();
        if self.sync() {
            /*match self.cache.slots[self.pos] {
                Slot::StartGroup { .. } => {
                    // reset the dirty flag now:
                    // if something sets it again when inside the group, it means we should run the group again
                    //mem::replace(&mut self.cache.entries[key].dirty, false)
                }
                _ => panic!("unexpected slot type"),
            }*/
        } else {
            // insert new group - start and end markers
            let call_id = self.id_stack.current();
            self.cache.slots.insert(
                self.pos,
                Slot::StartGroup {
                    call_id,
                    len: 2, // 2 = initial length of group (start+end slots)
                },
            );
            self.cache.slots.insert(self.pos + 1, Slot::EndGroup);
        };

        // enter group
        self.group_stack.push(self.pos);
        self.pos += 1;
    }

    fn dump(&self) {
        eprintln!("position : {}", self.pos);
        eprintln!("stack    : {:?}", self.group_stack);
        eprintln!("slots:");
        self.cache.dump(self.pos);
    }

    fn group_end_position(&self) -> usize {
        let mut i = self.pos;

        while i < self.cache.slots.len() {
            match self.cache.slots[i] {
                Slot::EndGroup => break,
                Slot::StartGroup { len, .. } => {
                    i += len as usize;
                }
                _ => i += 1,
            }
        }

        i
    }

    fn end_group(&mut self) {
        // all remaining slots in the group are now considered dead in this revision:
        // - find position of group end marker
        let group_end_pos = self.group_end_position();

        // remove the extra slots, and associated entries
        for slot in self.cache.slots.drain(self.pos..group_end_pos) {
            match slot {
                Slot::Value { var } => {
                    trace!(
                        "removing cache entry dep_node={:?} call_id={:?}, call_node={:#?}",
                        var.dep_node,
                        var.call_id,
                        var.call_node
                    );
                }
                _ => {}
            }
        }

        // skip GroupEnd marker
        self.pos += 1;
        // update group length
        let group_start_pos = self.group_stack.pop().expect("unbalanced groups");
        match self.cache.slots[group_start_pos] {
            Slot::StartGroup { ref mut len, .. } => {
                *len = (self.pos - group_start_pos).try_into().unwrap();
            }
            _ => {
                panic!("expected group start")
            }
        }
    }

    /// Skips the next entry or the next group.
    fn skip(&mut self) {
        match self.cache.slots[self.pos] {
            Slot::StartGroup { len, .. } => {
                self.pos += len as usize;
            }
            Slot::Value { .. } => {
                self.pos += 1;
            }
            Slot::EndGroup => {
                // nothing to skip
            }
        }
    }

    fn skip_until_end_of_group(&mut self) {
        while !matches!(self.cache.slots[self.pos], Slot::EndGroup) {
            self.skip()
        }
    }

    /// Inserts a new state entry.
    ///
    fn insert_entry<T: 'static>(&mut self, initial_value: T) -> State<T> {
        let call_id = self.id_stack.current();
        let call_node = self.id_stack.current_call_node();
        let var = Arc::new(StateCell {
            call_id,
            call_node,
            dep_node: Arc::new(DepNode::new()),
            waker: self.cache.waker.clone(),
            value: Mutex::new(initial_value),
        });
        self.cache.slots.insert(self.pos, Slot::Value { var: var.clone() });
        State(var)
    }

    fn get_or_insert_entry<T: 'static, Init: FnOnce() -> T>(&mut self, init: Init) -> CacheEntryInsertResult<T> {
        let result = if self.sync() {
            match self.cache.slots[self.pos] {
                Slot::Value { ref var } => CacheEntryInsertResult {
                    key: State(var.clone().downcast::<T>().unwrap()),
                    dirty: var.dep_node.is_dirty(),
                    inserted: false,
                },
                _ => panic!("unexpected entry type"),
            }
        } else {
            let key = self.insert_entry(init());
            CacheEntryInsertResult {
                key,
                dirty: false,
                inserted: true,
            }
        };
        self.pos += 1;
        result
    }

    fn compare_and_update<T: Data>(&mut self, new_value: T) -> bool {
        let CacheEntryInsertResult { key, inserted, .. } = self.get_or_insert_entry(|| new_value.clone());
        inserted || {
            if let Some(var) = self.state_stack.last() {
                key.0.dep_node.add_dependent(&var.dep_node);
            }
            let mut value = key.0.value.lock();
            if !new_value.same(&*value) {
                mem::replace(&mut *value, new_value);
                key.0.dep_node.invalidate_dependents();
                true
            } else {
                false
            }
        }
    }
}

/// TODO Document this stuff.
/// FIXME: verify that the automatic clone impl doesn't have sketchy implications w.r.t. cache invalidation
#[derive(Clone, Debug)]
pub struct Signal<T> {
    fetched: Cell<bool>,
    value: RefCell<Option<T>>,
    key: State<Option<T>>,
}

impl<T: Clone + 'static> Signal<T> {
    #[composable]
    pub fn new() -> Signal<T> {
        let key = state(|| None);
        Signal {
            fetched: Cell::new(false),
            value: RefCell::new(None),
            key,
        }
    }

    fn fetch_value(&self) {
        if !self.fetched.get() {
            let value = self.key.get();
            if value.is_some() {
                self.key.set(None);
            }
            self.value.replace(value);
            self.fetched.set(true);
        }
    }

    fn set(&self, value: T) {
        self.value.replace(Some(value));
        self.fetched.set(true);
    }

    pub fn signalled(&self) -> bool {
        self.fetch_value();
        self.value.borrow().is_some()
    }

    pub fn value(&self) -> Option<T> {
        self.fetch_value();
        self.value.borrow().clone()
    }

    pub fn map<U>(&self, f: impl FnOnce(T) -> U) -> Option<U> {
        self.value().map(f)
    }

    pub fn signal(&self, value: T) {
        self.key.set(Some(value));
    }
}

/// Context stored in TLS when running a function within the positional cache.
struct CacheContext {
    writer: CacheWriter,
    env: Environment,
}

thread_local! {
    // The cache context is put in TLS so that we don't have to pass an additional parameter
    // to all functions.
    // A less hack-ish solution would be to rewrite composable function calls, but we need
    // more than a proc macro to be able to do that (must resolve function paths and rewrite call sites)
    // The cache context lives on the main thread.
    static CACHE_CONTEXT: RefCell<Option<CacheContext>> = RefCell::new(None);
}

pub struct Cache {
    inner: Option<CacheInner>,
}

impl Cache {
    pub fn new(waker: Waker) -> Cache {
        Cache {
            inner: Some(CacheInner::new(waker)),
        }
    }

    /// Runs a cached function with the cache.
    pub fn recompose<T>(&mut self, env: &Environment, function: impl Fn() -> T) -> T {
        let root_location = Location::caller();

        CACHE_CONTEXT.with(|cx_cell| {
            let mut result;
            let mut inner = self.inner.take().unwrap();

            loop {
                inner.revision += 1;

                {
                    let mut cx = cx_cell.borrow_mut();
                    cx.replace(CacheContext {
                        writer: CacheWriter::new(root_location, inner),
                        env: env.clone(),
                    });
                }

                // run the function
                enter(0);
                result = function();
                exit();

                let mut cx = cx_cell.borrow_mut();
                let (cache, should_rerun) = cx.take().unwrap().writer.finish();
                inner = cache;

                if should_rerun {
                    // internal state within the cache is not consistent, run again
                    continue;
                }

                break;
            }

            self.inner = Some(inner);
            result
        })
    }
}

//--------------------------------------------------------------------------------------------------
fn with_cache_cx<R>(f: impl FnOnce(&mut CacheContext) -> R) -> R {
    CACHE_CONTEXT.with(|cx_cell| {
        let mut cx = cx_cell.borrow_mut();
        if let Some(ref mut cx) = &mut *cx {
            f(cx)
        } else {
            panic!("this function cannot be called outside of recomposition");
        }
    })
}

fn parent_state() -> Option<Arc<StateCell>> {
    CACHE_CONTEXT.with(|cx_cell| {
        let cx = cx_cell.borrow();
        if let Some(ref cx) = &*cx {
            cx.writer.state_stack.last().cloned()
        } else {
            None
        }
    })
}

/// Returns the current call identifier.
pub fn current_call_id() -> CallId {
    with_cache_cx(|cx| cx.writer.id_stack.current())
}

/// Returns the current revision.
pub fn revision() -> usize {
    with_cache_cx(|cx| cx.writer.cache.revision)
}

/// Must be called inside `Cache::run`.
#[track_caller]
fn enter(index: usize) {
    let location = Location::caller();
    with_cache_cx(move |cx| cx.writer.enter_scope(location, index));
}

/// Must be called inside `Cache::run`.
fn exit() {
    with_cache_cx(move |cx| cx.writer.exit_scope());
}

/// Must be called inside `Cache::run`.
#[track_caller]
pub fn scoped<R>(index: usize, f: impl FnOnce() -> R) -> R {
    enter(index);
    let r = f();
    exit();
    r
}

pub fn environment() -> Environment {
    with_cache_cx(|cx| cx.env.clone())
}

#[track_caller]
pub fn with_environment<R>(env: Environment, f: impl FnOnce() -> R) -> R {
    let parent_env = with_cache_cx(|cx| {
        let merged_env = cx.env.merged(env);
        mem::replace(&mut cx.env, merged_env)
    });
    let r = scoped(0, f);
    with_cache_cx(|cx| {
        cx.env = parent_env;
    });
    r
}

#[track_caller]
pub fn changed<T: Data>(value: T) -> bool {
    let location = Location::caller();
    with_cache_cx(move |cx| {
        cx.writer.enter_scope(location, 0);
        let changed = cx.writer.compare_and_update(value);
        cx.writer.exit_scope();
        changed
    })
}

#[track_caller]
fn state_inner<T: 'static, Init: FnOnce() -> T>(init: Init) -> CacheEntryInsertResult<T> {
    let location = Location::caller();
    with_cache_cx(move |cx| {
        cx.writer.enter_scope(location, 0);
        let r = cx.writer.get_or_insert_entry(init);
        cx.writer.exit_scope();
        r
    })
}

/// TODO document
#[track_caller]
pub fn state<T: 'static, Init: FnOnce() -> T>(init: Init) -> State<T> {
    state_inner(init).key
}

/// TODO document
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
        with_cache_cx(|cx| {
            let mut result_key_2 = result_key.clone();
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
}

#[track_caller]
pub fn group<R>(f: impl FnOnce() -> R) -> R {
    let location = Location::caller();
    with_cache_cx(|cx| {
        cx.writer.enter_scope(location, 0);
        cx.writer.start_group()
    });
    let r = f();
    with_cache_cx(|cx| {
        cx.writer.end_group();
        cx.writer.exit_scope();
    });
    r
}

pub fn skip_to_end_of_group() {
    with_cache_cx(|cx| {
        cx.writer.skip_until_end_of_group();
    })
}

/// Memoizes the result of a function at this call site.
#[track_caller]
pub fn memoize<Args: Data, T: Clone + 'static>(args: Args, f: impl FnOnce() -> T) -> T {
    group(move || {
        let (result_key, result_dirty) = with_cache_cx(move |cx| {
            let args_changed = cx.writer.compare_and_update(args);
            let CacheEntryInsertResult { key, dirty, .. } = cx.writer.get_or_insert_entry(|| None);
            /*if args_changed {
                trace!("memoize: recomputing because arguments have changed {:#?}", call_node);
            }
            if dirty {
                trace!("memoize: recomputing because state entry is dirty {:#?}", call_node);
            }*/
            (key, args_changed || dirty)
        });

        if result_dirty {
            with_cache_cx(|cx| {
                cx.writer.start_state(&result_key);
            });
            let result = f();
            with_cache_cx(|cx| {
                cx.writer.end_state();
            });
            result_key.replace(Some(result));
        } else {
            skip_to_end_of_group();
        }

        // it's important to call `get()` in all circumstances to make the parent state entry
        // dependent on this value.
        result_key.get().expect("memoize: no value calculated")
    })
}

/// Runs the function only once at the call site and caches the result (like memoize without parameters).
/// TODO better docs
#[track_caller]
pub fn once<T: Clone + 'static>(f: impl FnOnce() -> T) -> T {
    state(f).get()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_rewrite() {
        let mut cache = CacheInner::new();

        for _ in 0..3 {
            let mut writer = CacheWriter::new(cache);
            writer.start_group(CallId(99));
            writer.compare_and_update_value(CallId(1), 0, None);
            writer.compare_and_update_value(CallId(2), "hello world".to_string(), None);
            writer.end_group();
            cache = writer.finish();
            cache.dump(0);
        }
    }

    #[test]
    fn test_reorder() {
        use rand::prelude::SliceRandom;

        let mut cache = CacheInner::new();
        let mut rng = rand::thread_rng();
        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7];

        for i in 0..3 {
            let mut writer = CacheWriter::new(cache);
            for &item in items.iter() {
                eprintln!(" ==== Iteration {} - item {} =========================", i, item);
                writer.start_group(CallId(item));
                writer.compare_and_update_value(CallId(100), i, None);
                writer.end_group();
                writer.dump();
            }
            //writer.dump();
            cache = writer.finish();
            items.shuffle(&mut rng)
        }
    }

    /* #[test]
    fn test_placeholder() {
        let mut cache = CacheInner::new();

        for _ in 0..3 {
            let mut writer = CacheWriter::new(cache);
            writer.start_group(CallId(99));
            let changed = writer.compare_and_update_value(CallId(100), 0.0f32, None);
            let CacheEntryInsertResult { key, dirty, .. } =
                writer.get_or_insert_entry(CallId(101), None, || 0.0);
            let value = writer.get_value(key);

            if !changed {
                writer.skip_until_end_of_group();
            } else {
                writer.compare_and_update_value(CallId(102), "hello world".to_string(), None);
                writer.set_value(key, 0.0);
            }

            writer.end_group();
            cache = writer.finish();
            cache.dump(0);
        }
    }*/

    #[test]
    fn test_tagged_reorder() {
        use rand::prelude::SliceRandom;

        let mut cache = CacheInner::new();
        let mut rng = rand::thread_rng();
        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7];

        for i in 0..3 {
            let mut writer = CacheWriter::new(cache);
            for &item in items.iter() {
                eprintln!(" ==== Iteration {} - item {} =========================", i, item);
                writer.compare_and_update_value(CallId(100 + item), i, None);
            }
            //writer.dump();
            cache = writer.finish();
            cache.dump(0);
            items.shuffle(&mut rng)
        }
    }

    /*#[test]
    fn test_take_replace() {
        let mut cache = CacheInner::new();
        for i in 0..3 {
            let mut writer = CacheWriter::new(cache);
            let (index, value) = writer.take_value(false, || 0);
            writer.tagged_compare_and_update_value(CallKey(0), 123);
            writer.dump();
            writer.replace_value(index, i);
            cache = writer.finish();
        }
    }*/

    #[test]
    fn test_insert_remove() {
        use rand::prelude::SliceRandom;

        let mut cache = CacheInner::new();
        let mut rng = rand::thread_rng();

        #[derive(Clone, Debug, Eq, PartialEq)]
        struct Item {
            value: u64,
        }

        impl Data for Item {
            fn same(&self, other: &Self) -> bool {
                self.value == other.value
            }
        }

        impl Item {
            pub fn new(value: u64) -> Item {
                eprintln!("creating Item #{}", value);
                Item { value }
            }
        }

        impl Drop for Item {
            fn drop(&mut self) {
                eprintln!("dropping Item #{}", self.value);
            }
        }

        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7];

        for i in 0..10 {
            let num_items = rng.gen_range(0..10);
            let items = (0..num_items).map(|_| rng.gen_range(0..10)).collect::<Vec<_>>();

            eprintln!("Items: {:?}", items);

            let mut writer = CacheWriter::new(cache);
            for &item in items.iter() {
                //eprintln!(" ==== Iteration {} - item {} =========================", i, item);
                writer.compare_and_update_value(CallId(item), Item::new(item), None);
                //writer.dump();
            }
            //writer.dump();
            cache = writer.finish();
        }
    }
}
