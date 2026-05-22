use crate::Writer;
use crate::rust::{GenRust, RustFile};
use clerr::Report;
use code_gen::Block;
use proto_packet_tree::{ModName, TypeDec};

impl GenRust<'_> {
    //! Write: Type Declarations

    /// Writes the type declaration files to the `writer`.
    pub(super) fn write_type_decs<W>(self, writer: &W) -> Result<(), Report>
    where
        W: Writer,
    {
        // todo -- make parallel
        for (schema_path, schema_file) in self.project.schema_files() {
            for type_dec in schema_file.type_decs() {
                let type_mod: ModName = self.mod_name(type_dec);
                for (file_name, source) in self.gen_type_dec(type_dec) {
                    let file: RustFile = RustFile::new(
                        schema_path.to_ref(),
                        type_mod.to_ref(),
                        file_name.to_ref(),
                        &source,
                    );
                    file.write(writer)?;
                }
            }
        }
        Ok(())
    }

    /// Generates the `type_dec` files.
    fn gen_type_dec(self, type_dec: &TypeDec) -> Vec<(ModName, Block)> {
        match type_dec {
            TypeDec::Struct(s) => self.gen_struct(s),
            TypeDec::Message(m) => self.gen_message(m),
            TypeDec::Variant(v) => self.gen_variant(v),
            TypeDec::Enum(e) => self.gen_enum(e),
        }
    }
}
