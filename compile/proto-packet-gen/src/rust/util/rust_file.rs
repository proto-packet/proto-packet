use crate::Writer;
use clerr::Report;
use code_gen::Block;
use proto_packet_tree::{ModNameRef, ModPathRef};

/// Represents a generated Rust file.
#[derive(Copy, Clone)]
pub struct RustFile<'a> {
    schema_path: ModPathRef<'a>,
    type_mod: ModNameRef<'a>,
    file_name: ModNameRef<'a>,
    source: &'a Block,
}

impl<'a> RustFile<'a> {
    //! Construction

    /// Creates a new [RustFile].
    pub const fn new(
        schema_path: ModPathRef<'a>,
        type_mod: ModNameRef<'a>,
        file_name: ModNameRef<'a>,
        source: &'a Block,
    ) -> Self {
        Self {
            schema_path,
            type_mod,
            file_name,
            source,
        }
    }
}

impl RustFile<'_> {
    //! Write

    /// Writes the file to the `writer`.
    pub fn write<W>(self, writer: &W) -> Result<(), Report>
    where
        W: Writer,
    {
        writer.write(self.source, &self.file_path())
    }
}

impl RustFile<'_> {
    //! File Path

    /// The Rust file extension.
    const EXTENSION: &'static str = "rs";

    /// Gets the file path. '{schema_path}/{type_mod}/{file_name}.rs'
    fn file_path(self) -> String {
        let capacity: usize = self.schema_path.len()
            + 1
            + self.type_mod.len()
            + 1
            + self.file_name.len()
            + 1
            + Self::EXTENSION.len();
        let mut s: String = String::with_capacity(capacity);
        for folder_name in self.schema_path.mod_names() {
            s.push_str(&folder_name);
            s.push('/');
        }
        s.push_str(&self.type_mod);
        s.push('/');
        s.push_str(&self.file_name);
        s.push('.');
        s.push_str(Self::EXTENSION);
        debug_assert_eq!(capacity, s.len());
        s
    }
}
