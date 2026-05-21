use crate::{ValidateError, validate_import, validate_type_dec};
use clerr::Report;
use proto_packet_parse::SchemaFileTree;
use proto_packet_tree::{Import, SchemaFile, TypeDec};

/// Validates the schema file `tree` to a [SchemaFile], converting any error to a [Report].
pub fn validate(
    file_name: &str,
    tree: &SchemaFileTree,
    source: &str,
) -> Result<SchemaFile, Report> {
    validate_schema_file_tree(tree, source).map_err(|e| e.report(file_name, tree, source))
}

/// Validates the schema file `tree` to a [SchemaFile].
pub fn validate_schema_file_tree(
    tree: &SchemaFileTree,
    source: &str,
) -> Result<SchemaFile, ValidateError> {
    let mut schema_file: SchemaFile = SchemaFile::default();

    for import_tree in &tree.imports {
        let import: Import = validate_import(import_tree, source)?;
        schema_file
            .add_import(import)
            .map_err(|error| ValidateError::Tree { error })?;
    }

    for type_dec_tree in &tree.type_decs {
        let type_dec: TypeDec = validate_type_dec(type_dec_tree, source)?;
        schema_file
            .add_type_dec(type_dec)
            .map_err(|error| ValidateError::Tree { error })?;
    }

    Ok(schema_file)
}
