use crate::CompileError;
use crate::CompileError::{ParseSchema, ValidateSchema};
use file_storage::FilePath;
use proto_packet_parse::{SchemaFileTree, parse};
use proto_packet_tree::SchemaFile;
use proto_packet_validate::validate;

/// Responsible for reading schema files.
#[derive(Debug)]
pub struct SchemaReader {
    schema_file: FilePath,
    display_name: String,
}

impl SchemaReader {
    //! Construction

    /// Creates a new schema reader.
    ///
    /// The `display_name` is the path used in error messages (typically the path relative to the
    /// project source root).
    pub const fn new(schema_file: FilePath, display_name: String) -> Self {
        Self {
            schema_file,
            display_name,
        }
    }
}

impl SchemaReader {
    //! Read

    /// Reads the schema file.
    pub fn read_schema(&self) -> Result<SchemaFile, CompileError> {
        let source: String = self
            .schema_file
            .read_as_string()
            .map_err(CompileError::ReadSchema)?;
        let schema: SchemaFileTree =
            parse(self.display_name.as_str(), source.clone()).map_err(ParseSchema)?;
        let schema: SchemaFile = validate(self.display_name.as_str(), &schema, source.as_str())
            .map_err(ValidateSchema)?;
        Ok(schema)
    }
}
