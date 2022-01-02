use crate::{
    call_key::{CallId, CallIdStack, CallNode},
    Data,
};
use slotmap::SlotMap;
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    convert::TryInto,
    fmt,
    fmt::Formatter,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
    mem::ManuallyDrop,
    num::NonZeroU64,
    panic::Location,
    rc::Rc,
    sync::Arc,
};
use thiserror::Error;
use tracing::trace;

slotmap::new_key_type! {
    struct CacheEntryKey;
}

/// Error related to state entries.
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("state entry not found")]
    EntryNotFound,
    #[error("no value in state entry")]
    VacantEntry,
    #[error("state entry already contains a value")]
    OccupiedEntry,
    #[error("type mismatch")]
    TypeMismatch,
}

// TODO rename to `CacheEntry`?
struct StateEntry {
    call_id: CallId,
    /// For debugging purposes.
    call_node: Option<Rc<CallNode>>,
    parent: Option<CacheEntryKey>,
    dirty: bool,
    value: Option<Box<dyn Any>>,
}

impl StateEntry {
    pub fn value_mut<T: 'static>(&mut self) -> Result<&mut T, CacheError> {
        self.value
            .as_mut()
            .ok_or(CacheError::VacantEntry)?
            .downcast_mut::<T>()
            .ok_or(CacheError::TypeMismatch)
    }

    pub fn take_value<T: 'static>(&mut self) -> Result<T, CacheError> {
        if let Some(v) = self.value.take() {
            if v.is::<T>() {
                Ok(*v.downcast().unwrap())
            } else {
                // put back value
                self.value = Some(v);
                Err(CacheError::TypeMismatch)
            }
        } else {
            Err(CacheError::VacantEntry)
        }
    }
}

/// A slot in the slot table.
enum Slot {
    /// Marks the start of a group.
    /// Contains the length of the group including this slot and the `GroupEnd` marker.
    StartGroup {
        call_id: CallId,
        key: CacheEntryKey,
        len: u32,
    },
    /// Marks the end of a scope.
    EndGroup,
    Value {
        call_id: CallId,
        key: CacheEntryKey,
    },
}

/// A key used to access a state variable stored in a `Cache`.
pub struct Key<T> {
    key: CacheEntryKey,
    _phantom: PhantomData<*const T>,
}

impl<T> Copy for Key<T> {}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key {
            key: self.key,
            _phantom: Default::default(),
        }
    }
}

impl<T> fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.key, f)
    }
}

impl<T> Key<T> {
    ///
    fn from_entry_key(key: CacheEntryKey) -> Key<T> {
        Key {
            key,
            _phantom: PhantomData,
        }
    }
}

/// Cache internals. They are split from `Cache` itself so that they can be temporarily moved out.
struct CacheInner {
    /// The call tree, represented as an array of slots.
    slots: Vec<Slot>,
    ///
    entries: SlotMap<CacheEntryKey, StateEntry>,
    revision: usize,
}

impl CacheInner {
    pub fn new() -> CacheInner {
        let mut entries = SlotMap::with_key();
        let root_group_key = entries.insert(StateEntry {
            call_id: CallId(0),
            call_node: None,
            parent: None,
            dirty: false,
            value: None,
        });
        CacheInner {
            slots: vec![
                Slot::StartGroup {
                    call_id: CallId(0),
                    key: root_group_key,
                    len: 2,
                },
                Slot::EndGroup,
            ],

            revision: 0,
            entries,
        }
    }

    /// Sets the value of a state entry and invalidates all dependent entries.
    pub fn set_state<T: 'static>(&mut self, key: Key<T>, new_value: T) -> Result<(), CacheError> {
        let mut value = self.entries[key.key]
            .value
            .as_mut()
            .ok_or(CacheError::VacantEntry)?
            .downcast_mut::<T>()
            .ok_or(CacheError::TypeMismatch)?;
        *value = new_value;
        self.invalidate_entry_recursive(key.key);
        Ok(())
    }

    fn invalidate_entry_recursive(&mut self, key: CacheEntryKey) {
        if !self.entries.contains_key(key) {
            tracing::warn!("invalidate_group: no such group");
            return;
        }
        let group = &mut self.entries[key];
        group.dirty = true;
        if let Some(parent) = group.parent {
            self.invalidate_entry_recursive(parent);
        }
    }

    pub fn dump(&self, current_position: usize) {
        for (i, s) in self.slots.iter().enumerate() {
            if i == current_position {
                eprint!("* ");
            } else {
                eprint!("  ");
            }
            match s {
                Slot::StartGroup { call_id, len, key } => {
                    let entry = &self.entries[*key];
                    eprintln!(
                        "{:3} StartGroup call_id={:?} len={} (end={}) key={:?} parent={:?} dirty={}",
                        i,
                        call_id,
                        *len,
                        i + *len as usize - 1,
                        key,
                        entry.parent,
                        entry.dirty,
                    )
                }
                Slot::EndGroup => {
                    eprintln!("{:3} EndGroup", i)
                }
                Slot::Value { call_id, key } => {
                    eprintln!("{:3} Value      call_id={:?} key={:?}", i, call_id, key)
                }
            }
        }
    }
}

/// Holds the state during cache updates (`Cache::run`).
struct CacheWriter {
    /// The cache being updated
    cache: CacheInner,
    /// Current position in the slot table (`self.cache.slots`)
    pos: usize,
    /// Stack of group start positions.
    /// The top element is the start of the current group.
    group_stack: Vec<usize>,
}

impl CacheWriter {
    fn new(cache: CacheInner) -> CacheWriter {
        let mut writer = CacheWriter {
            cache,
            pos: 0,
            group_stack: vec![],
        };
        writer.start_group(CallId(0), None);
        writer
    }

    fn parent_entry_key(&self) -> Option<CacheEntryKey> {
        if let Some(&group_start) = self.group_stack.last() {
            match self.cache.slots[group_start] {
                Slot::StartGroup { key: group_key, .. } => Some(group_key),
                _ => panic!("unexpected entry type"),
            }
        } else {
            None
        }
    }

    /// Finishes writing to the cache, returns the updated cache object.
    pub fn finish(mut self) -> CacheInner {
        self.end_group();
        assert!(self.group_stack.is_empty(), "unbalanced groups");
        assert_eq!(self.pos, self.cache.slots.len());
        self.cache
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
                    call_id: this_call_id,
                    ..
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
    fn sync(&mut self, call_id: CallId) -> bool {
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

    /*fn parent_group_offset(&self) -> i32 {
        if let Some(&parent) = self.group_stack.last() {
            parent as i32 - self.pos as i32
        } else {
            0
        }
    }*/

    /*fn update_parent_group_offset(&mut self) {
        let parent = self.parent_group_offset();
        match &mut self.cache.slots[self.pos] {
            Slot::Tag(_) => {}
            Slot::StartGroup { parent: old_parent, .. } => {
                *old_parent = parent;
            }
            Slot::EndGroup => {}
            Slot::State(entry) => {
                entry.parent = parent;
            }
        }
    }*/

    pub fn start_group(&mut self, call_id: CallId, call_node: Option<Rc<CallNode>>) -> bool {
        let parent = self.parent_entry_key();
        let dirty = if self.sync(call_id) {
            match self.cache.slots[self.pos] {
                Slot::StartGroup { key, .. } => {
                    // reset the dirty flag now:
                    // if something sets it again when inside the group, it means we should run the group again
                    mem::replace(&mut self.cache.entries[key].dirty, false)
                }
                _ => panic!("unexpected slot type"),
            }
        } else {
            // insert new group - start and end markers
            let key = self.cache.entries.insert(StateEntry {
                call_id,
                call_node,
                parent,
                dirty: false,
                value: None,
            });
            self.cache.slots.insert(
                self.pos,
                Slot::StartGroup {
                    call_id,
                    key,
                    len: 2, // 2 = initial length of group (start+end slots)
                },
            );
            self.cache.slots.insert(self.pos + 1, Slot::EndGroup);
            false
        };

        // enter group
        self.group_stack.push(self.pos);
        self.pos += 1;
        dirty
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

    pub fn end_group(&mut self) {
        // all remaining slots in the group are now considered dead in this revision:
        // - find position of group end marker
        let group_end_pos = self.group_end_position();

        // remove the extra slots, and associated entries
        for slot in self.cache.slots.drain(self.pos..group_end_pos) {
            match slot {
                Slot::StartGroup { key, .. } | Slot::Value { key, .. } => {
                    let entry = self.cache.entries.get(key).expect("cache entry not found");
                    tracing::trace!("removing cache entry cache_key={:?} parent={:?} call_id={:?}, call_node={:#?}", key, entry.parent, entry.call_id, entry.call_node);
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
            Slot::StartGroup {
                ref mut len, key, ..
            } => {
                //self.cache.entries[key].dirty = false;
                *len = (self.pos - group_start_pos).try_into().unwrap();
            }
            _ => {
                panic!("expected group start")
            }
        }
    }

    /// Skips the next entry or the next group.
    pub fn skip(&mut self) {
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
    fn insert_value<T: 'static>(
        &mut self,
        call_id: CallId,
        value: Option<T>,
        call_node: Option<Rc<CallNode>>,
    ) -> Key<T> {
        let parent = self.parent_entry_key();
        let key = self.cache.entries.insert(StateEntry {
            call_id,
            call_node,
            parent,
            dirty: false,
            value: if let Some(value) = value {
                Some(Box::new(value))
            } else {
                None
            },
        });
        self.cache
            .slots
            .insert(self.pos, Slot::Value { call_id, key });
        Key::from_entry_key(key)
    }

    /// Sets the value of a cache entry. The entry can be vacant.
    fn set_value<T: 'static>(&mut self, cache_key: Key<T>, new_value: T) {
        let value = &mut self.cache.entries[cache_key.key].value;
        if let Some(v) = value {
            *v.downcast_mut::<T>().expect("type mismatch") = new_value;
        } else {
            *value = Some(Box::new(new_value));
        }
    }

    fn invalidate_cache_entry<T: 'static>(&mut self, cache_key: Key<T>) {
        self.cache.invalidate_entry_recursive(cache_key.key);
    }

    /// If the next entry is a value of type T, returns a clone of the value, otherwise inserts a vacant entry.
    fn get_value<T: Clone + 'static>(
        &mut self,
        call_key: CallId,
        call_node: Option<Rc<CallNode>>,
    ) -> (Option<T>, Key<T>) {
        let result = if self.sync(call_key) {
            match self.cache.slots[self.pos] {
                Slot::Value { key: entry_key, .. } => {
                    let value = self.cache.entries[entry_key]
                        .value_mut::<T>()
                        .unwrap()
                        .clone();
                    (Some(value), Key::from_entry_key(entry_key))
                }
                _ => panic!("unexpected entry type"),
            }
        } else {
            let k = self.insert_value(call_key, None, call_node);
            (None, k)
        };
        self.pos += 1;
        result
    }

    /// Same as `expect_value`, but instead of returning a clone of the value, takes the value and leaves a vacant entry.
    fn take_value<T: 'static>(
        &mut self,
        call_key: CallId,
        call_node: Option<Rc<CallNode>>,
    ) -> (Option<T>, Key<T>) {
        let result = if self.sync(call_key) {
            match self.cache.slots[self.pos] {
                Slot::Value { key: entry_key, .. } => {
                    // TODO allow vacant entries here?
                    let value = self.cache.entries[entry_key].take_value().unwrap();
                    (Some(value), Key::from_entry_key(entry_key))
                }
                _ => panic!("unexpected entry type"),
            }
        } else {
            let k = self.insert_value(call_key, None, call_node);
            (None, k)
        };
        self.pos += 1;
        result
    }

    /// If the next entry is a value of type T, returns a clone of the value, otherwise inserts a
    /// new value entry with `init` and returns a clone of this value.
    fn get_or_insert_value<T: Clone + 'static>(
        &mut self,
        call_key: CallId,
        call_node: Option<Rc<CallNode>>,
        init: impl FnOnce() -> T,
    ) -> (T, Key<T>) {
        let (v, k) = self.get_value(call_key, call_node);
        let v = match v {
            Some(v) => v,
            None => {
                let v = init();
                self.set_value(k, v.clone());
                v
            }
        };
        (v, k)
    }

    ///
    fn compare_and_update_value<T: Data>(
        &mut self,
        call_key: CallId,
        new_value: T,
        call_node: Option<Rc<CallNode>>,
    ) -> bool {
        let (v, k) = self.get_value::<T>(call_key, call_node);
        match v {
            Some(v) if v.same(&new_value) => false,
            _ => {
                self.set_value(k, new_value);
                true
            }
        }
    }
}

struct CacheContext {
    key_stack: CallIdStack,
    writer: CacheWriter,
}

thread_local! {
    // The cache context is put in TLS so that we don't have to pass an additional parameter
    // to all functions.
    // A less hack-ish solution would be to rewrite composable function calls, but we need
    // more than a proc macro to be able to do that (must resolve function paths and rewrite call sites)
    //
    // TODO: actually, it might be possible if we're able to rewrite all function calls into a specific
    // form
    static CURRENT_CACHE_CONTEXT: RefCell<Option<CacheContext>> = RefCell::new(None);
}

/// Positional cache.
pub struct Cache {
    inner: Option<CacheInner>,
}

impl Cache {
    /// Creates a new cache.
    pub fn new() -> Cache {
        Cache {
            inner: Some(CacheInner::new()),
        }
    }

    /// Runs a cached function with this cache.
    pub fn run<T>(&mut self, function: impl Fn() -> T) -> T {
        CURRENT_CACHE_CONTEXT.with(move |cx_cell| {
            // We can't put a reference type in a TLS.
            // As a workaround, use the classic sleight-of-hand:
            // temporarily move our internals out of self and into the TLS, and move it back to self once we've finished.
            let inner = self.inner.take().unwrap();

            // start writing to the cache
            let writer = CacheWriter::new(inner);

            // initialize the TLS cache context (which contains the cache table writer and the call key stack that maintains
            // unique IDs for each cached function call).
            let cx = CacheContext {
                key_stack: CallIdStack::new(),
                writer,
            };
            cx_cell.borrow_mut().replace(cx);

            // run the function
            let result = function();

            // finish writing to the cache
            let cx = cx_cell.borrow_mut().take().unwrap();
            // check that calls to CallKeyStack::enter and exit are balanced
            assert!(cx.key_stack.is_empty(), "unbalanced CallKeyStack");

            // finalize cache writer and put the internals back
            self.inner.replace(cx.writer.finish());

            result
        })
    }

    /// Sets the value of the state variable identified by `key`, and invalidates all dependent variables in the cache.
    pub fn set_state<T: 'static>(&mut self, key: Key<T>, value: T) -> Result<(), CacheError> {
        self.inner.as_mut().unwrap().set_state(key, value)
    }

    fn with_cx<R>(f: impl FnOnce(&mut CacheContext) -> R) -> R {
        CURRENT_CACHE_CONTEXT.with(|cx_cell| {
            let mut cx = cx_cell.borrow_mut();
            let cx = cx
                .as_mut()
                .expect("function cannot called outside of `Cache::run`");
            f(cx)
        })
    }

    /// Returns the current call identifier.
    pub fn current_call_id() -> CallId {
        Self::with_cx(|cx| cx.key_stack.current())
    }

    /// Must be called inside `Cache::run`.
    #[track_caller]
    fn enter(index: usize) {
        let location = Location::caller();
        Self::with_cx(move |cx| cx.key_stack.enter(location, index));
    }

    /// Must be called inside `Cache::run`.
    fn exit() {
        Self::with_cx(move |cx| cx.key_stack.exit());
    }

    /// Enters a
    /// Must be called inside `Cache::run`.
    #[track_caller]
    pub fn scoped<R>(index: usize, f: impl FnOnce() -> R) -> R {
        Self::enter(index);
        let r = f();
        Self::exit();
        r
    }

    #[track_caller]
    pub fn changed<T: Data>(value: T) -> bool {
        let location = Location::caller();
        Self::with_cx(move |cx| {
            cx.key_stack.enter(location, 0);
            let key = cx.key_stack.current();
            let node = cx.key_stack.current_call_node();
            let changed = cx.writer.compare_and_update_value(key, value, node);
            cx.key_stack.exit();
            changed
        })
    }

    #[track_caller]
    fn get_value<T: Clone + 'static>() -> (Option<T>, Key<T>) {
        let location = Location::caller();
        Self::with_cx(|cx| {
            cx.key_stack.enter(location, 0);
            let key = cx.key_stack.current();
            let node = cx.key_stack.current_call_node();
            let (value, entry_key) = cx.writer.get_value::<T>(key, node);
            cx.key_stack.exit();
            (value, entry_key)
        })
    }

    /// TODO document
    #[track_caller]
    pub fn state<T: Clone + 'static>(init: impl FnOnce() -> T) -> (T, Key<T>) {
        let location = Location::caller();
        Self::with_cx(move |cx| {
            cx.key_stack.enter(location, 0);
            let key = cx.key_stack.current();
            let node = cx.key_stack.current_call_node();
            let (value, cache_key) = cx.writer.get_or_insert_value(key, node, init);
            cx.key_stack.exit();
            (value, cache_key)
        })
    }

    /// TODO document.
    /// Same as `state` but for non-cloneable data. Leaves a vacant cache entry instead: put a value back with `update_state`, or `update_state_without_invalidation`.
    #[track_caller]
    pub fn take_state<T: 'static>() -> (Option<T>, Key<T>) {
        let location = Location::caller();
        Self::with_cx(move |cx| {
            cx.key_stack.enter(location, 0);
            let key = cx.key_stack.current();
            let node = cx.key_stack.current_call_node();
            let (value, cache_key) = cx.writer.take_value(key, node);
            cx.key_stack.exit();
            (value, cache_key)
        })
    }

    /// Updates a state entry.
    pub fn replace_state<T: 'static>(key: Key<T>, new_value: T) {
        Self::with_cx(move |cx| {
            cx.writer.set_value(key, new_value);
            cx.writer.invalidate_cache_entry(key);
        })
    }

    /// Updates a state entry.
    pub fn replace_state_without_invalidation<T: 'static>(key: Key<T>, new_value: T) {
        Self::with_cx(move |cx| cx.writer.set_value(key, new_value))
    }

    /*///
    #[track_caller]
    pub fn update_state<T: Data>(new_value: T) -> (Option<T>, Key<T>) {
        let location = Location::caller();
        Self::with_cx(move |cx| {
            cx.key_stack.enter(location, 0);
            let key = cx.key_stack.current();
            let (value, cache_key) = cx.writer.expect_value::<T>(key);
            match value {
                Some(ref v) if v.same(&new_value) => {
                    // same value, don't update
                }
                _ => {
                    // update
                    cx.writer.set_value(cache_key, new_value, true);
                }
            }
            cx.key_stack.exit();
            (value, cache_key)
        })
    }*/

    /// TODO document
    pub fn set_value<T: 'static>(key: Key<T>, value: T) {
        Self::with_cx(move |cx| cx.writer.set_value(key, value))
    }

    #[track_caller]
    pub fn group<R>(f: impl FnOnce(bool) -> R) -> R {
        let location = Location::caller();
        let dirty = Self::with_cx(|cx| {
            cx.key_stack
                .enter(location, 0);
            cx.writer
                .start_group(cx.key_stack.current(), cx.key_stack.current_call_node())
        });
        let r = f(dirty);
        Self::with_cx(|cx| {
            cx.writer.end_group();
            cx.key_stack.exit();
        });
        r
    }

    pub fn skip_to_end_of_group() {
        Self::with_cx(|cx| {
            cx.writer.skip_until_end_of_group();
        })
    }

    /// Memoizes the result of a function at this call site.
    #[track_caller]
    pub fn memoize<Args: Data, T: Clone + 'static>(
        args: Args,
        f: impl FnOnce() -> T,
    ) -> T {
        Self::group(move |dirty| {
            let changed = dirty | Self::changed(args);
            let (value, entry_key) = Self::get_value::<T>();
            if !changed {
                Self::skip_to_end_of_group();
                value.expect("memoize: no changes in arguments but no value calculated")
            } else {
                let value = f();
                Self::set_value(entry_key, value.clone());
                value
            }
        })
    }

    #[track_caller]
    pub fn with_state<T: Data, R>(init: impl FnOnce() -> T, update: impl Fn(&mut T) -> R) -> R {
        // load the state from the cache, or reserve a slot if it's the first time we run
        let (mut value, slot) = Self::get_value::<T>();
        let initial = value.is_none();

        let mut value = if let Some(value) = value {
            // use the existing state
            value
        } else {
            // create the initial value of the state
            init()
        };
        let mut old_value = value.clone();

        let r = update(&mut value);

        // if the state has changed, TODO
        if initial || !old_value.same(&value) {
            // TODO: re-run update? Invalidate?
            Self::set_value(slot, value);
        }

        r
    }
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
            writer.start_group(CallId(99), None);
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
                eprintln!(
                    " ==== Iteration {} - item {} =========================",
                    i, item
                );
                writer.start_group(CallId(item), None);
                writer.compare_and_update_value(CallId(100), i, None);
                writer.end_group();
                writer.dump();
            }
            //writer.dump();
            cache = writer.finish();
            items.shuffle(&mut rng)
        }
    }

    #[test]
    fn test_placeholder() {
        let mut cache = CacheInner::new();

        for _ in 0..3 {
            let mut writer = CacheWriter::new(cache);
            writer.start_group(CallId(99), None);
            let changed = writer.compare_and_update_value(CallId(100), 0, None);
            let (value, slot) = writer.get_value::<f64>(CallId(101), None);

            if !changed {
                assert!(value.is_some());
                writer.skip_until_end_of_group();
            } else {
                writer.compare_and_update_value(CallId(102), "hello world".to_string(), None);
                writer.set_value(slot, 0.0);
            }

            writer.end_group();
            cache = writer.finish();
            cache.dump(0);
        }
    }

    #[test]
    fn test_tagged_reorder() {
        use rand::prelude::SliceRandom;

        let mut cache = CacheInner::new();
        let mut rng = rand::thread_rng();
        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7];

        for i in 0..3 {
            let mut writer = CacheWriter::new(cache);
            for &item in items.iter() {
                eprintln!(
                    " ==== Iteration {} - item {} =========================",
                    i, item
                );
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
            let items = (0..num_items)
                .map(|_| rng.gen_range(0..10))
                .collect::<Vec<_>>();

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
