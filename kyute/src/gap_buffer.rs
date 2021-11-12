use std::{
    alloc,
    alloc::{handle_alloc_error, Layout},
    cmp::Ordering,
    collections::{Bound, VecDeque},
    marker::PhantomData,
    mem,
    ops::{Deref, Index, IndexMut, RangeBounds},
    ptr,
    ptr::NonNull,
};

struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> RawVec<T> {
    fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    fn grow(&mut self) {
        unsafe {
            let elem_size = mem::size_of::<T>();

            let (new_cap, ptr, new_layout) = if self.cap == 0 {
                let new_layout = Layout::array::<T>(1).unwrap();
                let ptr = alloc::alloc(new_layout);
                (1, ptr, new_layout)
            } else {
                let new_cap = 2 * self.cap;
                let old_layout = Layout::array::<T>(self.cap).unwrap();
                let new_layout = Layout::array::<T>(new_cap).unwrap();
                let new_byte_size = new_layout.size();

                assert!(new_byte_size < isize::MAX as usize);
                let ptr = alloc::realloc(self.ptr.as_ptr().cast(), old_layout, new_byte_size);
                (new_cap, ptr, new_layout)
            };

            // If allocate or reallocate fail, oom
            if ptr.is_null() {
                handle_alloc_error(new_layout)
            }

            self.ptr = NonNull::new_unchecked(ptr as *mut _);
            self.cap = new_cap;
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr().cast(),
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}

pub struct GapBuffer<T> {
    buf: RawVec<T>,
    gap_pos: usize,
    gap_size: usize,
}

impl<T> GapBuffer<T> {
    /// Creates an empty gap buffer.
    pub fn new() -> GapBuffer<T> {
        GapBuffer {
            buf: RawVec::new(),
            gap_pos: 0,
            gap_size: 0,
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.buf.cap - self.gap_size
    }

    fn base_ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    unsafe fn slot_ptr(&self, offset: usize) -> *mut T {
        self.buf.ptr.as_ptr().add(offset)
    }

    fn move_gap(&mut self, pos: usize, grow: bool) {
        if self.gap_size == 0 {
            if grow {
                let len = self.buf.cap;
                self.buf.grow();
                self.gap_pos = len;
                self.gap_size = self.buf.cap - len;
            } else {
                // empty gap, but did not ask to grow: just move the position
                self.gap_pos = pos;
                return;
            }
        }

        unsafe {
            match pos.cmp(&self.gap_pos) {
                Ordering::Greater => ptr::copy_nonoverlapping(
                    self.slot_ptr(self.gap_pos + self.gap_size),
                    self.slot_ptr(self.gap_pos),
                    pos - self.gap_pos,
                ),
                Ordering::Less => ptr::copy_nonoverlapping(
                    self.slot_ptr(pos),
                    self.slot_ptr(pos + self.gap_size),
                    self.gap_pos - pos,
                ),
                Ordering::Equal => {}
            }
            self.gap_pos = pos;
        }
    }

    /// Moves the gap at the given location and inserts the element
    pub fn insert(&mut self, pos: usize, elem: T) {
        self.move_gap(pos, true);

        unsafe {
            ptr::write(self.slot_ptr(pos), elem);
        }

        self.gap_pos += 1;
        self.gap_size -= 1;
    }

    /// Moves the gap to the given position and removes the element
    pub fn remove(&mut self, pos: usize) -> T {
        assert!(pos < self.len());
        self.move_gap(pos, false);
        let val = unsafe { ptr::read(self.slot_ptr(self.gap_pos + self.gap_size)) };
        self.gap_size += 1;
        val
    }

    fn get_elem_ptr(&self, pos: usize) -> *mut T {
        assert!(pos <= self.len());
        unsafe {
            if pos < self.gap_pos {
                self.slot_ptr(pos)
            } else {
                self.slot_ptr(self.gap_size + pos)
            }
        }
    }

    // start, end, gap_start, gap_end
    fn iter_bounds(&self, bounds: impl RangeBounds<usize>) -> (*mut T, *mut T, *mut T, *mut T) {
        let start = match bounds.start_bound() {
            Bound::Included(&i) => self.get_elem_ptr(i),
            Bound::Excluded(&i) => self.get_elem_ptr(i + 1),
            Bound::Unbounded => self.get_elem_ptr(0),
        };
        let end = match bounds.end_bound() {
            Bound::Included(&i) => self.get_elem_ptr(i + 1),
            Bound::Excluded(&i) => self.get_elem_ptr(i),
            Bound::Unbounded => self.get_elem_ptr(self.len()),
        };
        let gap_start = unsafe { self.slot_ptr(self.gap_pos) };
        let gap_end = unsafe { self.slot_ptr(self.gap_pos + self.gap_size) };
        (start, end, gap_start, gap_end)
    }

    /// Returns an iterator over a range of elements
    pub fn iter(&self, bounds: impl RangeBounds<usize>) -> Iter<T> {
        let (start, end, gap_start, gap_end) = self.iter_bounds(bounds);
        Iter {
            start,
            end,
            gap_start,
            gap_end,
            _phantom: PhantomData,
        }
    }

    /// Returns an iterator over a range of elements
    pub fn iter_mut(&mut self, bounds: impl RangeBounds<usize>) -> IterMut<T> {
        let (start, end, gap_start, gap_end) = self.iter_bounds(bounds);
        IterMut {
            start,
            end,
            gap_start,
            gap_end,
            _phantom: PhantomData,
        }
    }
}

impl<T> Index<usize> for GapBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.get_elem_ptr(index) }
    }
}

impl<T> IndexMut<usize> for GapBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.get_elem_ptr(index) }
    }
}

impl<T> Drop for GapBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.gap_pos {
                ptr::drop_in_place(self.base_ptr().add(i))
            }
            for i in (self.gap_pos + self.gap_size)..self.buf.cap {
                ptr::drop_in_place(self.base_ptr().add(i))
            }
        }
    }
}

pub struct Iter<'a, T> {
    start: *const T,
    end: *const T,
    gap_start: *const T,
    gap_end: *const T,
    _phantom: PhantomData<&'a GapBuffer<T>>,
}


impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let p = unsafe { &*self.start };
        self.start = unsafe { self.start.offset(1) };
        if self.start == self.gap_start {
            self.start = self.gap_end;
        }

        Some(p)
    }
}

pub struct IterMut<'a, T> {
    start: *mut T,
    end: *mut T,
    gap_start: *mut T,
    gap_end: *mut T,
    _phantom: PhantomData<&'a mut GapBuffer<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let p = unsafe { &mut *self.start };
        self.start = unsafe { self.start.offset(1) };
        if self.start == self.gap_start {
            self.start = self.gap_end;
        }

        Some(p)
    }
}
