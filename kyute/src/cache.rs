//! GUI positional cache.
use crate::{
    application::ExtEvent,
    call_id::{CallId, CallIdStack, CallNode},
    composable, Data, Environment,
    ValueRef::Env,
};
use kyute_shell::winit::event_loop::EventLoopProxy;
use replace_with::replace_with_or_abort;
use slotmap::SlotMap;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashSet,
    convert::TryInto,
    fmt,
    future::Future,
    hash::Hash,
    marker::PhantomData,
    mem,
    panic::Location,
    rc::Rc,
    task::Poll,
};
use tracing::{trace, warn};

slotmap::new_key_type! {
    struct KeyInner;
}

/// Entry representing a mutable state slot inside a composition cache.
struct StateEntry {
    call_id: CallId,
    /// For debugging purposes.
    call_node: Option<Rc<CallNode>>,
    /// Whether the value has been invalidated because a dependency has changed.
    dirty: Cell<bool>,
    dependents: HashSet<KeyInner>,
    value: Box<dyn Any>,
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
        call_id: CallId,
        key: KeyInner,
    },
}

/// A key used to access a state variable stored in a `Cache`.

// TODO right now the get/set methods on `Key` can only be called in a composition context,
// but not outside. To set the value of a cache entry outside of a composition context,
// we have to call `cache.set_state(key)`.
// A possible change would be to keep some kind of weak ref to the cache inside the key,
// and allow calling `Key::set` outside of a composition context.
// This way there would be a single function to call, regardless of the calling context, streamlining the
// API.
// Counterpoint: this bloats the struct with an Arc pointer.
// Counter-counterpoint: but at least this makes the whole system self-contained.
//
// What if the cache was put in TLS *also* during layout, recomp, etc?
// Heck, what if the cache was *always* in TLS?

pub struct Key<T> {
    inner: KeyInner,
    // TODO: `cache: Arc<Cache>`. When setting values, schedule a recomp on the main event loop. Cache gets an EventLoopProxy on construction to send recomp events.
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Copy for Key<T> {}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key {
            inner: self.inner,
            _phantom: Default::default(),
        }
    }
}

impl<T> fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T: 'static> Key<T> {
    ///
    fn from_inner(key: KeyInner) -> Key<T> {
        Key {
            inner: key,
            _phantom: PhantomData,
        }
    }

    /// Returns the value of the cache entry and replaces it by the given value.
    /// Always invalidates.
    /// Can be called outside of recomposition.
    pub fn replace(&self, new_value: T) -> T {
        with_cache_cx(|cx| cx.replace(*self, new_value, true))
    }

    /// Returns the value of the cache entry and replaces it by the default value.
    /// Does not invalidate the dependent entries.
    pub fn replace_without_invalidation(&self, new_value: T) -> T {
        with_cache_cx(|cx| cx.replace(*self, new_value, false))
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

impl<T: Data + 'static> Key<T> {
    ///
    pub fn update(&self, new_value: T) -> Option<T> {
        with_cache_cx(|cx| cx.update(*self, new_value))
    }
}

impl<T: Clone + 'static> Key<T> {
    pub fn get(&self) -> T {
        with_cache_cx(|cx| cx.get(*self))
    }
}

impl<T: Default + 'static> Key<T> {
    /// Returns the value of the cache entry and replaces it by the default value.
    pub fn take(&self) -> T {
        self.replace(T::default())
    }

    /// Returns the value of the cache entry and replaces it by the default value. Does not invalidate dependent entries.
    pub fn take_without_invalidation(&self) -> T {
        self.replace_without_invalidation(T::default())
    }
}

/// Composition cache. Contains the recorded call tree and state entries.
struct Cache {
    /// The call tree, represented as an array of slots.
    slots: Vec<Slot>,
    /// Cache entries.
    entries: SlotMap<KeyInner, StateEntry>,
    /// The number of times `Cache::run` has been called.
    revision: usize,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            slots: vec![],
            revision: 0,
            entries: SlotMap::with_key(),
        }
    }

    /// Gets the value of a state entry.
    fn get<T: Clone + 'static>(&self, key: Key<T>) -> Option<&T> {
        self.entries
            .get(key.inner)?
            .value
            .downcast_ref::<T>()
            .expect("type mismatch")
            .into()
    }

    /*/// Sets the value of a state entry and invalidates all dependent entries.
    fn set<T: 'static>(&mut self, key: Key<T>, new_value: T) {
        if !self.entries.contains_key(key.inner) {
            warn!("set_state: entry deleted: {:?}", key.inner);
            return;
        }

        let value = self.entries[key.inner]
            .value
            .downcast_mut::<T>()
            .expect("type mismatch");
        *value = new_value;
        self.invalidate_dependents(key.inner);
    }*/

    /// Replaces the value of a state entry and invalidates all dependent entries.
    fn replace<T: 'static>(&mut self, key: Key<T>, new_value: T, invalidate: bool) -> T {
        let value = self
            .entries
            .get_mut(key.inner)
            .expect("entry deleted")
            .value
            .downcast_mut::<T>()
            .expect("type mismatch");
        let ret = mem::replace(value, new_value);
        if invalidate {
            self.invalidate_dependents(key.inner);
        }
        ret
    }

    /// Replaces the value of a state entry and invalidates all dependent entries if the new value is different.
    fn update<T: Data>(&mut self, key: Key<T>, new_value: T) -> Option<T> {
        let value = self
            .entries
            .get_mut(key.inner)
            .expect("entry deleted")
            .value
            .downcast_mut::<T>()
            .expect("type mismatch");

        if !new_value.same(value) {
            let ret = mem::replace(value, new_value);
            self.invalidate_dependents(key.inner);
            Some(ret)
        } else {
            None
        }
    }

    fn invalidate_dependents(&self, entry_key: KeyInner) {
        for &d in self.entries[entry_key].dependents.iter() {
            self.invalidate_dependents_recursive(d);
        }
    }

    fn invalidate_dependents_recursive(&self, entry_key: KeyInner) {
        assert!(
            self.entries.contains_key(entry_key),
            "invalidate_dependents_recursive: no such entry"
        );
        /*if !self.entries.contains_key(entry) {
            tracing::warn!("invalidate_dependents_recursive: no such entry");
            return;
        }*/
        let entry = &self.entries[entry_key];
        /*if let Some(ref call_node) = entry.call_node {
            trace!(
                "invalidate_dependents_recursive: {} (#{})",
                call_node.location,
                call_node.index
            );
        }*/

        //if !entry.dirty.replace(true) {
        entry.dirty.set(true);
        for &d in entry.dependents.iter() {
            self.invalidate_dependents_recursive(d);
        }
        //}
    }

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
                Slot::Value { call_id, key } => {
                    let entry = &self.entries[*key];
                    if let Some(ref node) = entry.call_node {
                        eprintln!(
                            "{:3} Value      call_id={:?} key={:?} dirty={:?} [{}]",
                            i,
                            call_id,
                            key,
                            entry.dirty.get(),
                            node.location
                        )
                    } else {
                        eprintln!(
                            "{:3} Value      call_id={:?} key={:?} dirty={:?}",
                            i,
                            call_id,
                            key,
                            entry.dirty.get()
                        )
                    }
                }
            }
        }
    }
}

struct CacheEntryInsertResult<T> {
    key: Key<T>,
    dirty: bool,
    inserted: bool,
}

struct CompositionContext<'a> {
    cache: &'a mut Cache,
    writer: &'a mut CacheWriter,
}

/// Holds the state during cache updates (`Cache::run`).
struct CacheWriter {
    cache: Cache,
    /// Current position in the slot table (`self.cache.slots`)
    pos: usize,
    /// Stack of call IDs.
    id_stack: CallIdStack,
    /// Stack of group start positions.
    /// The top element is the start of the current group.
    group_stack: Vec<usize>,
    /// Stack of state entries.
    state_stack: Vec<KeyInner>,
}

impl CacheWriter {
    fn new(root_location: &'static Location<'static>, cache: Cache) -> CacheWriter {
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
        writer.state_stack.push(root_cache_key.inner);
        writer.cache.entries[root_cache_key.inner].dirty.set(false);
        writer
    }

    fn finish(mut self) -> (Cache, bool) {
        let root_state_key = self.state_stack.pop().expect("unbalanced state scopes");
        let should_rerun = self.cache.entries.get(root_state_key).unwrap().dirty.get();
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

    fn start_state<T>(&mut self, key: Key<T>) {
        self.state_stack.push(key.inner);
        self.cache.entries[key.inner].dirty.set(false);
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
                Slot::Value {
                    call_id: this_call_id, ..
                } if this_call_id == call_id => {
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
                Slot::Value { key, .. } => {
                    let entry = self.cache.entries.get(key).expect("cache entry not found");
                    trace!(
                        "removing cache entry cache_key={:?} depends={:?} call_id={:?}, call_node={:#?}",
                        key,
                        entry.dependents,
                        entry.call_id,
                        entry.call_node
                    );
                    self.cache.entries.remove(key).unwrap();
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
    fn insert_entry<T: 'static>(&mut self, initial_value: T) -> Key<T> {
        let call_id = self.id_stack.current();
        let call_node = self.id_stack.current_call_node();
        let key = self.cache.entries.insert(StateEntry {
            call_id,
            call_node,
            dependents: HashSet::new(),
            dirty: Cell::new(false),
            value: Box::new(initial_value),
        });
        self.cache.slots.insert(self.pos, Slot::Value { call_id, key });
        Key::from_inner(key)
    }

    fn invalidate_dependents<T: 'static>(&mut self, cache_key: Key<T>) {
        self.cache.invalidate_dependents(cache_key.inner);
    }

    fn get_or_insert_entry<T: 'static, Init: FnOnce() -> T>(&mut self, init: Init) -> CacheEntryInsertResult<T> {
        let result = if self.sync() {
            match self.cache.slots[self.pos] {
                Slot::Value { key, .. } => CacheEntryInsertResult {
                    key: Key::from_inner(key),
                    dirty: self.cache.entries[key].dirty.get(),
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

    fn get<T: Clone + 'static>(&mut self, key: Key<T>) -> T {
        let entry = &mut self.cache.entries[key.inner];
        if let Some(parent_state) = self.state_stack.last().cloned() {
            //trace!("{:?} made dependent of {:?}", parent_state, key.key);
            entry.dependents.insert(parent_state);
        }
        entry.value.downcast_ref::<T>().expect("unexpected type").clone()
    }

    fn replace<T: 'static>(&mut self, key: Key<T>, new_value: T, invalidate: bool) -> T {
        let entry = &mut self.cache.entries[key.inner];
        if let Some(parent_state) = self.state_stack.last().cloned() {
            //trace!("{:?} made dependent of {:?}", parent_state, key.key);
            entry.dependents.insert(parent_state);
        }
        self.cache.replace(key, new_value, invalidate)
    }

    fn update<T: Data>(&mut self, key: Key<T>, new_value: T) -> Option<T> {
        let entry = &mut self.cache.entries[key.inner];
        if let Some(parent_state) = self.state_stack.last().cloned() {
            //trace!("{:?} made dependent of {:?}", parent_state, key.key);
            entry.dependents.insert(parent_state);
        }
        self.cache.update(key, new_value)
    }

    fn compare_and_update<T: Data>(&mut self, new_value: T) -> bool {
        let CacheEntryInsertResult { key, inserted, .. } = self.get_or_insert_entry(|| new_value.clone());
        inserted || { self.update(key, new_value).is_some() }
    }
}

/// TODO Document this stuff.
/// FIXME: verify that the automatic clone impl doesn't have sketchy implications w.r.t. cache invalidation
#[derive(Clone, Debug)]
pub struct Signal<T> {
    fetched: Cell<bool>,
    value: RefCell<Option<T>>,
    key: Key<Option<T>>,
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

    pub fn set(&self, value: T) {
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

#[derive(Clone, Debug)]
pub struct State<T>(Key<T>);

enum CacheCompositionStatus {
    Idle(Cache),
    Recomposing(CacheWriter),
}

/// Context stored in TLS when running a function within the positional cache.
struct CacheContext {
    event_loop_proxy: Option<EventLoopProxy<ExtEvent>>,
    cache: CacheCompositionStatus,
    env: Environment,
}

impl CacheContext {
    fn new() -> CacheContext {
        CacheContext {
            event_loop_proxy: None,
            cache: CacheCompositionStatus::Idle(Cache::new()),
            env: Environment::new(),
        }
    }

    fn cache_mut(&mut self) -> &mut Cache {
        match self.cache {
            CacheCompositionStatus::Idle(ref mut cache) => cache,
            CacheCompositionStatus::Recomposing(ref mut writer) => &mut writer.cache,
        }
    }

    fn get<T: Clone + 'static>(&mut self, key: Key<T>) -> T {
        match self.cache {
            CacheCompositionStatus::Idle(ref cache) => {
                // outside composition, we just read the value
                cache.get(key).unwrap().clone()
            }
            CacheCompositionStatus::Recomposing(ref mut writer) => {
                // during recomposition, reading a state entry makes the parent state dependent on it
                writer.get(key)
            }
        }
    }

    fn replace<T: 'static>(&mut self, key: Key<T>, new_value: T, invalidate: bool) -> T {
        match self.cache {
            CacheCompositionStatus::Idle(ref mut cache) => cache.replace(key, new_value, invalidate),
            CacheCompositionStatus::Recomposing(ref mut writer) => writer.replace(key, new_value, invalidate),
        }
    }

    fn update<T: Data>(&mut self, key: Key<T>, new_value: T) -> Option<T> {
        match self.cache {
            CacheCompositionStatus::Idle(ref mut cache) => cache.update(key, new_value),
            CacheCompositionStatus::Recomposing(ref mut writer) => writer.update(key, new_value),
        }
    }
}

thread_local! {
    // The cache context is put in TLS so that we don't have to pass an additional parameter
    // to all functions.
    // A less hack-ish solution would be to rewrite composable function calls, but we need
    // more than a proc macro to be able to do that (must resolve function paths and rewrite call sites)
    // The cache context lives on the main thread.
    static CACHE_CONTEXT: RefCell<CacheContext> = RefCell::new(CacheContext::new());
}

/// Runs a cached function with the cache.
#[track_caller]
pub fn recompose<T>(event_loop_proxy: EventLoopProxy<ExtEvent>, env: &Environment, function: impl Fn() -> T) -> T {
    let root_location = Location::caller();
    CACHE_CONTEXT.with(|cx_cell| {
        let mut result;

        loop {
            {
                let mut cx = cx_cell.borrow_mut();
                // sleight-of-hand
                replace_with_or_abort(&mut cx.cache, |status| match status {
                    CacheCompositionStatus::Idle(mut cache) => {
                        cache.revision += 1;
                        CacheCompositionStatus::Recomposing(CacheWriter::new(root_location, cache))
                    }
                    other => other,
                });
                cx.event_loop_proxy = Some(event_loop_proxy.clone());
                cx.env = env.clone();
            }

            // run the function
            enter(0);
            result = function();
            exit();

            let should_rerun = {
                let mut should_rerun = false;
                let mut cx = cx_cell.borrow_mut();
                replace_with_or_abort(&mut cx.cache, |status| match status {
                    CacheCompositionStatus::Recomposing(writer) => {
                        // finish writing to the cache
                        let (cache, rerun) = writer.finish();
                        should_rerun = rerun;
                        CacheCompositionStatus::Idle(cache)
                    }
                    other => other,
                });
                should_rerun
            };

            if should_rerun {
                // internal state within the cache is not consistent, run again
                continue;
            }

            break;
        }

        result
    })
}

//--------------------------------------------------------------------------------------------------

fn with_cache_writer<R>(f: impl FnOnce(&mut CacheWriter) -> R) -> R {
    CACHE_CONTEXT.with(|cx_cell| {
        let mut cx = cx_cell.borrow_mut();
        match cx.cache {
            CacheCompositionStatus::Idle(_) => {
                panic!("this function cannot be called outside of recomposition");
            }
            CacheCompositionStatus::Recomposing(ref mut writer) => f(writer),
        }
    })
}

fn with_cache_cx<R>(f: impl FnOnce(&mut CacheContext) -> R) -> R {
    CACHE_CONTEXT.with(|cx_cell| {
        let mut cx = cx_cell.borrow_mut();
        f(&mut *cx)
    })
}

/// Returns the current call identifier.
pub fn current_call_id() -> CallId {
    with_cache_writer(|cx| cx.id_stack.current())
}

/// Returns the current revision.
pub fn revision() -> usize {
    with_cache_writer(|cx| cx.cache.revision)
}

/// Must be called inside `Cache::run`.
#[track_caller]
fn enter(index: usize) {
    let location = Location::caller();
    with_cache_writer(move |cx| cx.enter_scope(location, index));
}

/// Must be called inside `Cache::run`.
fn exit() {
    with_cache_writer(move |cx| cx.exit_scope());
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
    with_cache_writer(move |cx| {
        cx.enter_scope(location, 0);
        let changed = cx.compare_and_update(value);
        cx.exit_scope();
        changed
    })
}

#[track_caller]
fn state_inner<T: 'static, Init: FnOnce() -> T>(init: Init) -> CacheEntryInsertResult<T> {
    let location = Location::caller();
    with_cache_writer(move |cx| {
        cx.enter_scope(location, 0);
        let r = cx.get_or_insert_entry(init);
        cx.exit_scope();
        r
    })
}

/// TODO document
#[track_caller]
pub fn state<T: 'static, Init: FnOnce() -> T>(init: Init) -> Key<T> {
    state_inner(init).key
}

//#[track_caller]
//pub fn

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
            let el = cx.event_loop_proxy.clone().unwrap();

            // spawn task that will set the value
            let handle = tokio::spawn(async move {
                let result = future.await;
                // TODO I'd really like to just do `key.set(...)` regardless of whether we're in or out the cache,
                // instead of having to do weird things like this
                // FIXME it's coming up
                el.send_event(ExtEvent::Recompose {
                    cache_fn: Box::new(move || {
                        result_key.set(Poll::Ready(result));
                    }),
                });
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
    with_cache_writer(|cx| {
        cx.enter_scope(location, 0);
        cx.start_group()
    });
    let r = f();
    with_cache_writer(|cx| {
        cx.end_group();
        cx.exit_scope();
    });
    r
}

pub fn skip_to_end_of_group() {
    with_cache_writer(|cx| {
        cx.skip_until_end_of_group();
    })
}

pub fn event_loop_proxy() -> EventLoopProxy<ExtEvent> {
    with_cache_cx(|cx| cx.event_loop_proxy.clone().unwrap())
}

/// Memoizes the result of a function at this call site.
#[track_caller]
pub fn memoize<Args: Data, T: Clone + 'static>(args: Args, f: impl FnOnce() -> T) -> T {
    group(move || {
        let (result_key, result_dirty) = with_cache_writer(move |cx| {
            let args_changed = cx.compare_and_update(args);
            let CacheEntryInsertResult { key, dirty, .. } = cx.get_or_insert_entry(|| None);
            /*if args_changed {
                trace!("memoize: recomputing because arguments have changed {:#?}", call_node);
            }
            if dirty {
                trace!("memoize: recomputing because state entry is dirty {:#?}", call_node);
            }*/
            (key, args_changed || dirty)
        });

        if result_dirty {
            with_cache_writer(|cx| {
                cx.start_state(result_key);
            });
            let result = f();
            with_cache_writer(|cx| {
                cx.end_state();
                cx.replace(result_key, Some(result), true);
            });
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
