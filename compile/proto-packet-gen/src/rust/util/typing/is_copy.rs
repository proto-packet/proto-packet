use crate::rust::GenRust;
use proto_packet_tree::{QualifiedNameRef, Struct, TypeDec, TypeTag, WithTypeTag};

impl GenRust<'_> {
    //! Typing: Is Copy

    /// Checks if the `tag` is a `Copy` type.
    pub fn is_copy(&self, tag: &impl WithTypeTag) -> bool {
        match tag.type_tag() {
            TypeTag::Primitive(_primitive) => true,
            TypeTag::Named(name) => self.is_copy_named(name.to_ref()),
        }
    }

    /// Checks if the `named` type is `Copy`.
    pub fn is_copy_named(&self, named: QualifiedNameRef) -> bool {
        if let Some(mod_path) = named.mod_path() {
            if let Some(schema_file) = self.project.schema_file_with_mod_path(mod_path) {
                if let Some(type_dec) = schema_file.type_dec_with_type_name(named.type_name()) {
                    match type_dec {
                        TypeDec::Struct(s) => self.is_copy_struct(s),
                        TypeDec::Message(_) => false,
                        TypeDec::Variant(_) => todo!(),
                        TypeDec::Enum(_) => todo!(),
                    }
                } else {
                    unreachable!(
                        "no type name in schema file: schema-file={}, type-name={}",
                        mod_path,
                        named.type_name()
                    )
                }
            } else {
                unreachable!("no schema file: {}", mod_path)
            }
        } else {
            unreachable!("type name without mod path: {}", named);
        }
    }

    /// Checks if the struct `s` is `Copy`.
    pub fn is_copy_struct(&self, s: &Struct) -> bool {
        s.fields().iter().all(|field| self.is_copy(field))
    }
}
