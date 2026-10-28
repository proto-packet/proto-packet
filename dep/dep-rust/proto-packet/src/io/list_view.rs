/// A borrowed list, read in place. (see `FORMAT.md`)
///
/// The list views & `&[u8]` implement it, so a `MapRef` reads its keys & values from any of them.
pub trait ListView {
    /// The type of the elements.
    type Item;

    /// Gets the number of elements.
    fn len(&self) -> usize;

    /// Checks if there are no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the element at the `index`, without checking the bounds.
    ///
    /// # Safety
    /// The `index` must be less than the number of elements.
    unsafe fn get_unchecked(&self, index: usize) -> Self::Item;
}

impl ListView for &[u8] {
    type Item = u8;

    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    unsafe fn get_unchecked(&self, index: usize) -> u8 {
        *unsafe { <[u8]>::get_unchecked(self, index) }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{BitListRef, FixedListRef, ListView, VarListRef, read_u32};

    /// Reads the elements of the `list`.
    fn elements<L>(list: &L) -> Vec<L::Item>
    where
        L: ListView,
    {
        (0..list.len())
            .map(|index| unsafe { list.get_unchecked(index) })
            .collect()
    }

    /// Reads a `u32` element.
    fn number(bytes: &[u8]) -> u32 {
        read_u32(bytes, 0)
    }

    /// Reads an element as its bytes.
    fn element(bytes: &[u8]) -> &[u8] {
        bytes
    }

    #[test]
    fn list_views_read_their_elements() {
        let bytes: &[u8] = &[1, 2, 3];
        assert_eq!(elements(&bytes), [1, 2, 3]);
        let empty: &[u8] = &[];
        assert!(ListView::is_empty(&empty));

        let bytes: Vec<u8> = [1_u32, 2].iter().flat_map(|n| n.to_le_bytes()).collect();
        let list = FixedListRef::new(&bytes, 4, number).unwrap();
        assert_eq!(elements(&list), [1, 2]);

        let list: BitListRef = BitListRef::new(&[2, 0, 0, 0, 0b0000_0010]).unwrap();
        assert_eq!(elements(&list), [false, true]);

        let bytes: Vec<u8> = [&1_u32.to_le_bytes()[..], &2_u32.to_le_bytes(), b"ab"].concat();
        let list = VarListRef::new(&bytes, element).unwrap();
        assert_eq!(elements(&list), [b"ab"]);
    }
}
