use crate::io::Element;

/// An element that an array reads in place as a list.
///
/// # Note
/// - Every element has it but a byte, since an array of bytes is read in place as its bytes.
pub trait ArrayElement: Element {}
