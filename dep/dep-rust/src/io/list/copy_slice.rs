use crate::io::ListView;

/// A list of the copies of the elements of a slice.
///
/// # Note
/// - A slice of any `Copy` type is a list for a `View` trait with it. (ex: a slice of `&str`)
#[derive(Debug)]
pub struct CopySlice<'a, T> {
    items: &'a [T],
}

impl<'a, T> CopySlice<'a, T>
where
    T: Copy,
{
    //! Construction

    /// Creates the [CopySlice].
    #[must_use]
    pub const fn new(items: &'a [T]) -> Self {
        Self { items }
    }
}

impl<T> Copy for CopySlice<'_, T> {}

impl<T> Clone for CopySlice<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> ListView for CopySlice<'_, T>
where
    T: Copy,
{
    type Item = T;

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, index: usize) -> Option<T> {
        self.items.get(index).copied()
    }

    fn iter(&self) -> impl Iterator<Item = T> {
        self.items.iter().copied()
    }
}
