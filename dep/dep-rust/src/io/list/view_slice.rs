use crate::io::ListView;

/// A list of the views of a slice, as references to its elements.
///
/// # Note
/// - A slice of any type is a list for a `View` trait with it. (ex: a slice of borrowed rows)
#[derive(Debug)]
pub struct ViewSlice<'a, T> {
    items: &'a [T],
}

impl<'a, T> ViewSlice<'a, T> {
    //! Construction

    /// Creates the [ViewSlice].
    #[must_use]
    pub const fn new(items: &'a [T]) -> Self {
        Self { items }
    }
}

impl<T> Copy for ViewSlice<'_, T> {}

impl<T> Clone for ViewSlice<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> ListView for ViewSlice<'a, T> {
    type Item = &'a T;

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, index: usize) -> Option<&'a T> {
        self.items.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = &'a T> {
        self.items.iter()
    }
}
