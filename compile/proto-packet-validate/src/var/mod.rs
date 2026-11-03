pub use tag_number::*;
pub use type_tag::*;

pub(crate) use tag_number::tagged_member_error;
pub(crate) use type_tag::validate_boxed;

mod tag_number;
mod type_tag;
