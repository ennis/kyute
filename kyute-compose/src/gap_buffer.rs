use std::{
    alloc,
    alloc::{handle_alloc_error, Layout},
    cmp::Ordering,
    collections::Bound,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut, Range, RangeBounds},
    ptr,
    ptr::{slice_from_raw_parts_mut, NonNull},
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
            //let elem_size = mem::size_of::<T>();

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
                alloc::dealloc(self.ptr.as_ptr().cast(), Layout::array::<T>(self.cap).unwrap());
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

    pub fn move_gap(&mut self, pos: usize, grow: bool) {
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

        match pos.cmp(&self.gap_pos) {
            Ordering::Greater => {
                unsafe {
                    // SAFETY: TODO
                    ptr::copy_nonoverlapping(
                        self.slot_ptr(self.gap_pos + self.gap_size),
                        self.slot_ptr(self.gap_pos),
                        pos - self.gap_pos,
                    );
                }
                self.gap_pos = pos;
            }
            Ordering::Less => {
                unsafe {
                    // SAFETY: TODO
                    ptr::copy_nonoverlapping(
                        self.slot_ptr(pos),
                        self.slot_ptr(pos + self.gap_size),
                        self.gap_pos - pos,
                    );
                }
                self.gap_pos = pos;
            }
            Ordering::Equal => {}
        }
    }

    pub(crate) fn slice(&self, bounds: impl RangeBounds<usize>) -> (&[T], &[T]) {
        let (a, b) = self.raw_slices(bounds);
        unsafe { (&*a, &*b) }
    }

    pub(crate) fn slice_mut(&mut self, bounds: impl RangeBounds<usize>) -> (&mut [T], &mut [T]) {
        let (a, b) = self.raw_slices(bounds);
        unsafe { (&mut *a, &mut *b) }
    }

    fn raw_slices(&self, bounds: impl RangeBounds<usize>) -> (*mut [T], *mut [T]) {
        let (start, end, gap_start, gap_end) = self.iter_bounds(bounds);

        if self.gap_size != 0 && (start..end).contains(&gap_start) {
            // SAFETY: gap_start >= start due to the condition above
            // SAFETY: end >= gap_end
            unsafe {
                let mut ranges = (
                    slice_from_raw_parts_mut(start, gap_start.offset_from(start) as usize),
                    slice_from_raw_parts_mut(gap_end, end.offset_from(gap_end) as usize),
                );
                if (&*ranges.0).is_empty() {
                    mem::swap(&mut ranges.0, &mut ranges.1);
                }
                ranges
            }
        } else {
            unsafe {
                (
                    slice_from_raw_parts_mut(start, end.offset_from(start) as usize),
                    &mut [] as *mut _,
                )
            }
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

    /// Removes a range of elements.
    ///
    /// TODO drain
    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) {
        let range = self.resolve_bounds(range);
        for _ in 0..range.len() {
            self.remove(range.start);
        }
    }

    fn resolve_bounds(&self, bounds: impl RangeBounds<usize>) -> Range<usize> {
        let start = match bounds.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i + 1,
            Bound::Unbounded => 0,
        };
        let end = match bounds.end_bound() {
            Bound::Included(&i) => i + 1,
            Bound::Excluded(&i) => i,
            Bound::Unbounded => self.len(),
        };
        start..end
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

impl<T> Default for GapBuffer<T> {
    fn default() -> Self {
        GapBuffer::new()
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

#[cfg(test)]
mod tests {
    use crate::gap_buffer::GapBuffer;
    use rand::{thread_rng, Rng};

    fn insert_string(buf: &mut GapBuffer<char>, mut at: usize, str: &str) {
        for c in str.chars() {
            buf.insert(at, c);
            at += 1;
        }
    }

    fn extract_string(buf: &GapBuffer<char>) -> String {
        let (a, b) = buf.slice(..);
        a.iter().chain(b.iter()).collect::<String>()
    }

    #[test]
    fn test_insertion_removal() {
        let mut buf = GapBuffer::<char>::new();

        // the gap shouldn't be in the middle
        insert_string(&mut buf, 0, "hello world");
        assert_eq!(extract_string(&buf), "hello world");
        assert!(buf.slice(..).1.is_empty());

        // insert in the middle
        insert_string(&mut buf, 5, " crazy");
        assert_eq!(&extract_string(&buf), "hello crazy world");

        // removal
        buf.remove_range(0..3);
        assert_eq!(&extract_string(&buf), "lo crazy world");
    }

    #[test]
    fn stress_test() {
        let mut buf = GapBuffer::new();
        let mut ref_vec = Vec::new();

        let mut rng = thread_rng();

        for i in 0..100 {
            let v: i32 = rng.gen_range(-10000..10000);
            buf.insert(buf.len(), v);
            ref_vec.push(v);
        }

        // random insertion/deletions
        for i in 0..100 {
            let pos: usize = rng.gen_range(0..ref_vec.len());
            let v: i32 = rng.gen_range(-10000..10000);
            match rng.gen_range(0..3) {
                0i32 => {
                    // single element insert
                    buf.insert(pos, v);
                    ref_vec.insert(pos, v);
                }
                1 => {
                    // batch insert
                    let batch_len: usize = rng.gen_range(0..ref_vec.len() / 5);
                    for i in pos..(pos + batch_len) {
                        buf.insert(i, v);
                        ref_vec.insert(pos, v);
                    }
                }
                2 => {
                    // single element remove
                    buf.remove(pos);
                    ref_vec.remove(pos);
                }
                3 => {
                    // batch remove
                    let batch_len: usize = rng.gen_range(0..ref_vec.len() / 5);
                    for i in 0..batch_len {
                        if pos < buf.len() {
                            buf.remove(pos);
                        }
                        if pos < ref_vec.len() {
                            ref_vec.remove(pos);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        buf.move_gap(buf.len(), false);
        eprintln!("{:?},{:?}", &ref_vec[..], buf.slice(..).0);
        assert_eq!(&ref_vec[..], buf.slice(..).0);
    }
}
