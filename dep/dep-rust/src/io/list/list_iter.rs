use crate::io::{Element, ListRef};

/// An iterator of the elements of an encoded list.
pub struct ListIter<'a, T> {
    list: ListRef<'a, T>,
    index: usize,
    len: usize,
}

impl<'a, T> ListIter<'a, T>
where
    T: Element,
{
    //! Construction

    /// Creates the iterator of the elements of the `list`.
    pub(crate) fn new(list: ListRef<'a, T>) -> Self {
        Self {
            list,
            index: 0,
            len: list.len(),
        }
    }
}

impl<'a, T> Iterator for ListIter<'a, T>
where
    T: Element,
{
    type Item = T::Ref<'a>;

    fn next(&mut self) -> Option<T::Ref<'a>> {
        if self.index == self.len {
            return None;
        }
        let element: T::Ref<'a> = unsafe { self.list.get_unchecked(self.index) };
        self.index += 1;
        Some(element)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining: usize = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for ListIter<'_, T> where T: Element {}
