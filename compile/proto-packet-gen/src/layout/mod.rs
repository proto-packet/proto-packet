//! The language-neutral layouts of encoded types. (see `format/binary.md`)

pub use bit_layout::*;
pub use entry_layout::*;
pub use field_layout::*;
pub use message_layout::*;
pub use struct_layout::*;
pub use type_layout::*;
pub use unit_layout::*;
pub use value_layout::*;

mod bit_layout;
mod entry_layout;
mod field_layout;
mod message_layout;
mod struct_layout;
mod type_layout;
mod unit_layout;
mod value_layout;
