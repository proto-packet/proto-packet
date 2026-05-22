pub use common::*;
pub use error::*;
pub use naming::*;
pub use packet::*;
pub use schema::*;
pub use service::*;
pub use var::*;

mod common;
mod error;
mod naming;
mod packet;
mod schema;
mod service;
mod var;

impl_with_comments!(
    SchemaFile,
    Struct,
    StructField,
    Message,
    MessageField,
    Enum,
    EnumCase,
    Variant,
    VariantCase,
    Service,
    ServiceCall,
);
