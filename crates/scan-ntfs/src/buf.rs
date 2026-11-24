//! A heap buffer aligned to 4 KiB, as unbuffered volume reads require (`FILE_FLAG_NO_BUFFERING`:
//! address, offset and length multiples of the sector size).

use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::ptr::NonNull;

pub const ALIGN: usize = 4096;

pub struct AlignedBuf {
    ptr: NonNull<u8>,
    len: usize,
}

// SAFETY: the buffer owns its memory exclusively, like a `Vec<u8>`.
unsafe impl Send for AlignedBuf {}

impl AlignedBuf {
    /// A zeroed buffer of `len` bytes (rounded up to `ALIGN`). `None` if the allocation fails.
    pub fn new(len: usize) -> Option<Self> {
        let len = len.max(ALIGN).div_ceil(ALIGN) * ALIGN;
        let layout = Layout::from_size_align(len, ALIGN).ok()?;
        // SAFETY: `layout` has a non-zero size.
        let ptr = NonNull::new(unsafe { alloc_zeroed(layout) })?;
        Some(Self { ptr, len })
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: `ptr` points to `len` initialised (zeroed) bytes owned by `self`.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        if let Ok(layout) = Layout::from_size_align(self.len, ALIGN) {
            // SAFETY: allocated in `new` with this same layout.
            unsafe { dealloc(self.ptr.as_ptr(), layout) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ALIGN, AlignedBuf};

    #[test]
    fn aligned_and_zeroed() {
        let mut b = AlignedBuf::new(10_000).expect("alloc");
        let s = b.as_mut_slice();
        assert_eq!(s.len(), 12_288);
        assert_eq!(s.as_ptr() as usize % ALIGN, 0);
        assert!(s.iter().all(|x| *x == 0));
    }
}
