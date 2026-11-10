use crate::io::{Element, ListRef};

/// The elements of any form of a list.
///
/// # Note
/// - A slice of the elements has it, & so does an encoded list.
pub trait ListView: Copy {
    /// An element of the list.
    type Item;

    /// Gets the count of the elements.
    fn len(&self) -> usize;

    /// Checks if the list has no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the element at the `index`.
    ///
    /// Returns `None` if the `index` is not less than the count.
    fn get(&self, index: usize) -> Option<Self::Item>;

    /// Gets the iterator of the elements.
    fn iter(&self) -> impl Iterator<Item = Self::Item>;
}

impl<'a, T> ListView for &'a [T]
where
    T: Element,
{
    type Item = T::View<'a>;

    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    fn get(&self, index: usize) -> Option<T::View<'a>> {
        <[T]>::get(self, index).map(T::view)
    }

    fn iter(&self) -> impl Iterator<Item = T::View<'a>> {
        <[T]>::iter(self).map(T::view)
    }
}

impl<'a, T> ListView for ListRef<'a, T>
where
    T: Element,
{
    type Item = T::Ref<'a>;

    fn len(&self) -> usize {
        ListRef::len(*self)
    }

    fn get(&self, index: usize) -> Option<T::Ref<'a>> {
        ListRef::get(*self, index)
    }

    fn iter(&self) -> impl Iterator<Item = T::Ref<'a>> {
        ListRef::iter(*self)
    }
}
