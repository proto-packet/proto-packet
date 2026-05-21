use crate::{LinkError, NameResolver, TypeLinker};
use proto_packet_tree::{ModPathRef, SchemaFile, TypeDec, WithComments};

/// Responsible for linking schema files.
#[derive(Copy, Clone, Debug)]
pub struct SchemaLinker<'a> {
    mod_path: ModPathRef<'a>,
}

impl<'a> From<ModPathRef<'a>> for SchemaLinker<'a> {
    fn from(mod_path: ModPathRef<'a>) -> Self {
        Self { mod_path }
    }
}

impl SchemaLinker<'_> {
    //! Link

    /// Links the `schema_file`.
    pub fn link(self, schema_file: &SchemaFile) -> Result<SchemaFile, LinkError> {
        let resolver: NameResolver = NameResolver::new(self.mod_path, schema_file);
        let linker: TypeLinker = TypeLinker::from(resolver);

        let mut result: SchemaFile = SchemaFile::default();
        for comment in schema_file.comments() {
            result.add_comment(comment);
        }
        for type_dec in schema_file.type_decs() {
            let type_dec: TypeDec = linker.link(type_dec)?;
            result.add_type_dec(type_dec).expect("duplicate type dec");
        }
        Ok(result)
    }
}
