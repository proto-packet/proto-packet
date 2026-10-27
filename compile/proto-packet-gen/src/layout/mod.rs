//! The layouts of the encodings, shared by the languages. (see `FORMAT.md`)

pub use bit_layout::*;
pub use field_layout::*;
pub use struct_layout::*;
pub use value_layout::*;

mod bit_layout;
mod field_layout;
mod struct_layout;
mod value_layout;
