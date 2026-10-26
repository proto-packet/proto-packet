use crate::{ValidateError, ValidateErrorReason, validate_import, validate_type_dec};
use lex::lexer::Span;
use proto_packet_parse::SchemaFileTree;
use proto_packet_tree::{Import, SchemaFile, TypeDec};

/// Validates the schema file `tree` of the `source`.
pub fn validate_schema_file(
    tree: &SchemaFileTree,
    source: &str,
) -> Result<SchemaFile, ValidateError> {
    let mut schema_file: SchemaFile = SchemaFile::default();
    for import_tree in &tree.imports {
        let import: Import = validate_import(import_tree, source)?;
        let span: Span = import_tree.name.span();
        schema_file
            .add_import(import)
            .map_err(|error| ValidateError::new(span, ValidateErrorReason::Tree(error)))?;
    }
    for type_dec_tree in &tree.type_decs {
        let type_dec: TypeDec = validate_type_dec(type_dec_tree, source)?;
        let span: Span = type_dec_tree.type_name();
        schema_file
            .add_type_dec(type_dec)
            .map_err(|error| ValidateError::new(span, ValidateErrorReason::Tree(error)))?;
    }
    Ok(schema_file)
}
