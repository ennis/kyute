//! GUI positional cache.
use crate::{
    cache_cx,
    cache_cx::{CacheContextTLS, CACHE_CONTEXT},
    call_id::{CallId, CallIdStack},
    gap_buffer::GapBuffer,
};
use kyute_common::Data;
use smallvec::SmallVec;
use std::{
    any::Any,
    cell::RefCell,
    fmt, mem,
    panic::Location,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    task::Waker,
};

//==================================================================================================

/// Entry representing a mutable cache variable inside a composition cache.
pub struct CacheVar<T: ?Sized = dyn Any> {
    /// Whether this state entry is dirty (the value must be recomputed)
    dirty: AtomicBool,
    waker: Waker,
    //call_id: CallId,
    /// Dependent CacheVars.
    dependents: RefCell<SmallVec<[Rc<CacheVar>; 4]>>,
    pub(crate) value: RefCell<T>,
}

impl<T> CacheVar<T> {
    fn new(initial_value: T, waker: Waker) -> CacheVar<T> {
        CacheVar {
            dirty: AtomicBool::new(false),
            waker,
            dependents: RefCell::new(Default::default()),
            value: RefCell::new(initial_value),
        }
    }
}

impl<T: ?Sized> fmt::Debug for CacheVar<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VarCell").finish_non_exhaustive()
    }
}

impl CacheVar {
    fn downcast<U: Any + 'static>(self: Rc<Self>) -> Result<Rc<CacheVar<U>>, Rc<CacheVar>> {
        if (*self).value.borrow_mut().is::<U>() {
            unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as *const CacheVar<U>)) }
        } else {
            Err(self)
        }
    }
}

impl<T: ?Sized> CacheVar<T> {
    fn set_dirty(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst)
    }

    fn invalidate_dependents(&self) {
        let dependents = self.dependents.borrow();
        for d in dependents.iter() {
            d.invalidate();
        }
    }

    fn add_dependent(&self, dep: Rc<CacheVar>) {
        if &dep.waker as *const _ == &self.waker as *const _ {
            eprintln!("dependency cycle detected");
            return;
        }
        let mut deps = self.dependents.borrow_mut();
        for d in deps.iter() {
            if Rc::ptr_eq(d, &dep) {
                return;
            }
        }
        deps.push(dep);
    }

    /// Sets this cache variable as a dependency of the current parent cache variable.
    ///
    /// This does nothing if executed outside of a caching context.
    pub fn set_dependency(&self) {
        CACHE_CONTEXT.with(|cx_cell| {
            let cx = cx_cell.borrow();
            if let Some(ref cx) = &*cx {
                self.add_dependent(cx.cx.parent_var());
            }
        });
    }

    fn invalidate(&self) {
        self.set_dirty(true);
        self.invalidate_dependents();
    }
}

impl<T: ?Sized> CacheVar<T> {
    /// Returns whether this cache variable is dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
}

/*
impl<T: Data> CacheVar<T> {
    /// Updates the value contained in the cache variable.
    ///
    /// If the previous value is different from the new value, the dependents of this variable are invalidated.
    pub fn update(&self, new_value: T) -> Option<T> {
        let mut value = self.value.borrow_mut();
        if !new_value.same(&*value) {
            let ret = mem::replace(&mut *value, new_value);
            self.invalidate_dependents();
            self.waker.wake_by_ref();
            Some(ret)
        } else {
            None
        }
    }
}*/

impl<T: 'static> CacheVar<T> {
    /// Sets the value of this cache variable and returns the previous value.
    pub fn replace(&self, new_value: T, invalidate: bool) -> T {
        let mut value = self.value.borrow_mut();
        let ret = mem::replace(&mut *value, new_value);
        if invalidate {
            self.invalidate_dependents();
            self.waker.wake_by_ref();
        }
        ret
    }

    pub fn update_with(&self, f: impl FnOnce(&mut T) -> bool) {
        let mut value = self.value.borrow_mut();
        if f(&mut *value) {
            self.invalidate_dependents();
            self.waker.wake_by_ref();
        }
    }
}

impl<T: Clone + 'static> CacheVar<T> {
    /// Sets the value of this cache variable and returns the previous value.
    ///
    /// # Dependency
    ///
    /// If run inside a caching context, the parent variable becomes dependent on this one.
    pub fn get(&self) -> T {
        self.value.borrow_mut().clone()
    }
}

//==================================================================================================

/// A state variable in the positional cache.
// 32b
struct VarNode {
    /// Call site identification
    call_id: CallId, // 8b
    /// Variable ID.
    var: Rc<CacheVar>, // 16b
    /// Number of slots to go to the next state entry at the same level.
    next: i32, // 4b
    /// Offset to the parent state entry (always negative).
    parent: i32, // 4b
}

//==================================================================================================

/// Composition cache. Contains the recorded call tree and state variables.
struct CacheInner {
    waker: Waker,
    /// State variables.
    nodes: GapBuffer<VarNode>,
    /// The number of times `Cache::run` has been called.
    revision: usize,
    /// Root variable.
    root: Rc<CacheVar>,
}

impl CacheInner {
    fn new(waker: Waker) -> CacheInner {
        let root_var = Rc::new(CacheVar::new((), waker.clone()));
        let root_node = VarNode {
            call_id: CallId::DUMMY,
            var: root_var.clone(),
            next: 1,
            parent: 0,
        };
        let mut nodes = GapBuffer::new();
        nodes.insert(0, root_node);
        CacheInner {
            waker,
            nodes,
            revision: 0,
            root: root_var,
        }
    }

    fn dump(&self, current_position: usize) {
        for (i, s) in self.nodes.iter(..).enumerate() {
            if i == current_position {
                eprint!("* ");
            } else {
                eprint!("  ");
            }
            eprint!(
                "{:3} Var call_id={:?} next={} (end={}) dirty={:5} var={:p} dependents=[",
                i,
                s.call_id,
                s.next,
                i + s.next as usize,
                s.var.is_dirty(),
                s.var.as_ref(),
            );
            for d in s.var.dependents.borrow_mut().iter() {
                eprint!("{:p},", d.as_ref());
            }
            eprintln!("]");
        }
    }
}

/// Holds the state during cache updates (`Cache::run`).
pub(crate) struct CacheContext {
    cache: CacheInner,

    /// Current position in the slot table (`self.cache.slots`)
    pos: usize,

    prev_node_count: i32,
    node_count: i32,

    /// Stack of call IDs.
    id_stack: CallIdStack,

    /// Saves the position of the current parent variable node.
    node_stack: Vec<(usize, i32, i32)>,
}

impl CacheContext {
    fn new(root_location: &'static Location<'static>, cache: CacheInner) -> CacheContext {
        let mut cx = CacheContext {
            cache,
            pos: 0,
            prev_node_count: 0,
            node_count: 0,
            id_stack: CallIdStack::new(),
            node_stack: vec![],
        };

        // enter root var
        cx.enter_call_scope(root_location, 0);
        let root_node = &cx.cache.nodes[0];
        root_node.var.set_dirty(false);
        cx.node_count = root_node.next;
        cx.prev_node_count = root_node.next;
        cx.node_stack.push((0, 0, 0));
        cx.pos += 1;
        cx
    }

    fn finish(mut self) -> (CacheInner, bool) {
        assert_eq!(self.node_stack.pop(), Some((0, 0, 0)), "unbalanced calls");
        assert!(self.node_stack.is_empty(), "unbalanced calls");
        // trim removed nodes
        self.cache.nodes.remove_range(self.pos..);
        self.cache.nodes[0].next = self.pos as i32;
        self.exit_call_scope();
        assert!(self.id_stack.is_empty(), "unbalanced calls");
        let should_rerun = self.cache.nodes[0].var.is_dirty();
        (self.cache, should_rerun)
    }

    /// Returns the current cache revision.
    pub(crate) fn revision(&self) -> usize {
        self.cache.revision
    }

    /// Finds a var scope at the current level with the specified call ID, starting from the current position.
    ///
    /// # Return value
    ///
    /// The position of the variable scope if found, or None.
    fn find_node(&self, call_id: CallId) -> Option<usize> {
        let mut i = self.pos;

        let end = self.node_end_pos();
        //eprintln!("parent={}, len={}", self.node_stack.last().unwrap().0, self.node_count);
        assert!(end <= self.cache.nodes.len());

        while i < end {
            let scope = &self.cache.nodes[i];
            if scope.call_id == call_id {
                return Some(i);
            }
            i += scope.next as usize;
        }

        None
    }

    /// Returns the position at the end of the current scope.
    fn node_end_pos(&self) -> usize {
        self.node_stack.last().unwrap().0 + self.node_count as usize
    }

    /// Rotates the specified var node into the current insertion position.
    fn rotate_in_current_position(&mut self, pos: usize) {
        if pos == self.pos {
            return;
        }
        let end = self.node_end_pos();
        assert!((self.pos..end).contains(&pos));
        self.cache.nodes.move_gap(self.pos, false);
        let (slice, remainder) = self.cache.nodes.slice_mut(self.pos..end);
        assert!(remainder.is_empty());
        slice.rotate_left(pos - self.pos);
    }

    /// TODO docs
    fn sync(&mut self) -> bool {
        let call_id = self.id_stack.current();
        let pos = self.find_node(call_id);
        match pos {
            Some(pos) => {
                self.rotate_in_current_position(pos);
                true
            }
            None => false,
        }
    }

    /// Inserts a new variable scope.
    fn insert_var<T: 'static>(&mut self, initial_value: T) -> Rc<CacheVar<T>> {
        let call_id = self.id_stack.current();
        let var = Rc::new(CacheVar::new(initial_value, self.cache.waker.clone()));
        self.cache.nodes.insert(
            self.pos,
            VarNode {
                call_id,
                var: var.clone(),
                next: 1,
                parent: 0, // TODO?
            },
        );
        //eprintln!("insert_var");
        var
    }

    //===========================================================

    pub(crate) fn enter_var<T: 'static, Init: FnOnce() -> T>(&mut self, init: Init) -> (Rc<CacheVar<T>>, bool) {
        let result;
        let next;

        if self.sync() {
            let node = &self.cache.nodes[self.pos];
            next = node.next;
            result = (node.var.clone().downcast().expect("unexpected variable type"), false);
        } else {
            let var = self.insert_var(init());
            self.node_count += 1;
            next = 1;
            result = (var, true);
        };

        // save current node count
        self.node_stack.push((self.pos, self.prev_node_count, self.node_count));

        self.prev_node_count = next;
        self.node_count = next;
        self.pos += 1;

        //eprintln!("enter_var: parent={}, pos={}, len={}", self.pos - 1, self.pos, next);

        result
    }

    pub(crate) fn exit_var(&mut self) {
        // all remaining var nodes in the group are now considered dead in this revision

        // remove the extra nodes
        let end = self.node_end_pos();
        self.cache.nodes.remove_range(self.pos..end);

        let (node_pos, popped_prev_node_count, popped_node_count) =
            self.node_stack.pop().expect("unbalanced calls to enter_var/exit_var");
        // new node count
        let new_node_count = (self.pos - node_pos) as i32;
        self.cache.nodes[node_pos].next = new_node_count;
        // added/removed node count
        let delta = new_node_count - self.prev_node_count;

        // restore previous state
        self.prev_node_count = popped_prev_node_count;
        // take into account delta of nodes into the size of the parent node
        self.node_count = popped_node_count + delta;

        /*eprintln!(
            "exit_var: parent={}, pos={}, len={}",
            node_pos, self.pos, self.node_count
        );*/
    }

    pub(crate) fn enter_call_scope(&mut self, location: &'static Location<'static>, index: usize) {
        self.id_stack.enter(location, index);
    }

    pub(crate) fn exit_call_scope(&mut self) {
        self.id_stack.exit();
    }

    /*/// Skips the next entry or the next group.
    pub(crate) fn skip(&mut self) {
        self.pos += self.cache.nodes[self.pos].next as usize;
    }*/

    pub(crate) fn skip_until_end_of_group(&mut self) {
        self.pos = self.node_end_pos();
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn compare_and_update<T: Data>(&mut self, new_value: T) -> bool {
        let (var, inserted) = self.enter_var(|| new_value.clone());
        self.exit_var();
        inserted || {
            let mut value = var.value.borrow_mut();
            if !new_value.same(&*value) {
                *value = new_value;
                true
            } else {
                false
            }
        }
    }

    /// Returns the current parent variable.
    pub(crate) fn parent_var(&self) -> Rc<CacheVar> {
        self.cache.nodes[self.node_stack.last().unwrap().0].var.clone()
    }

    /// Returns the current caller ID.
    pub(crate) fn caller_id(&self) -> CallId {
        self.id_stack.current()
    }
}

//==================================================================================================

pub struct Cache {
    inner: Option<CacheInner>,
}

impl Cache {
    pub fn new(waker: Waker) -> Cache {
        Cache {
            inner: Some(CacheInner::new(waker)),
        }
    }

    /// Returns whether the cached state has been dirtied from external sources.
    ///
    /// Typically, this is set when a CacheVar has been modified and before the function is re-run.
    pub fn is_dirty(&self) -> bool {
        self.inner
            .as_ref()
            .expect("`is_dirty` should not be called inside `run`")
            .root
            .is_dirty()
    }

    /// Runs a cached function with the cache.
    pub fn run<T>(&mut self, mut function: impl FnMut() -> T) -> T {
        let root_location = Location::caller();

        CACHE_CONTEXT.with(|cx_cell| {
            let mut result;
            let mut inner = self.inner.take().unwrap();

            loop {
                inner.revision += 1;

                {
                    let mut cx = cx_cell.borrow_mut();
                    cx.replace(CacheContextTLS {
                        cx: CacheContext::new(root_location, inner),
                        //env: env.clone(),
                    });
                }

                // run the function
                cache_cx::enter_call(0);
                result = function();
                cache_cx::exit_call();

                let mut cx = cx_cell.borrow_mut();
                let (cache, should_rerun) = cx.take().unwrap().cx.finish();
                inner = cache;

                if should_rerun {
                    // internal state within the cache is not consistent, run again
                    eprintln!("running again");
                    continue;
                }

                break;
            }

            self.inner = Some(inner);
            result
        })
    }

    pub fn dump(&self) {
        self.inner.as_ref().unwrap().dump(0)
    }
}

/*
/// Trait for values that can appear in memoizable functions (`#[composable(cached)]`)
pub trait ToMemoizeArg {
    type Target: Data;
    fn to_memoize_arg(&self) -> Self::Target;
}

impl<T, B> ToMemoizeArg for B
where
    T: Data,
    B: Borrow<T>,
{
    type Target = T;
    fn to_memoize_arg(&self) -> Self::Target {
        self.clone()
    }
}*/

/*#[cfg(test)]
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
}*/
