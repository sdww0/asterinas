

extern "C" {
    /// Copies `size` bytes from `src` to `dst`. This function works with exception handling
    /// and can recover from page fault.
    /// Returns number of bytes that failed to copy.
    pub(crate) fn __memcpy_fallible(dst: *mut u8, src: *const u8, size: usize) -> usize;
    /// Fills `size` bytes in the memory pointed to by `dst` with the value `value`.
    /// This function works with exception handling and can recover from page fault.
    /// Returns number of bytes that failed to set.
    pub(crate) fn __memset_fallible(dst: *mut u8, value: u8, size: usize) -> usize;
}



