extern "C" {
    /// Returns the length of the given null-terminated byte string, that is,
    /// the number of characters in a character array whose first element is
    /// pointed to by `str` up to and not including the first null character.
    ///
    /// The behavior is undefined if `str` is not a pointer to a null-terminated
    /// byte string.
    pub(crate) fn strlen(str: *const u8) -> usize;
}
