use crate::naming::{validate_name, validate_name_text};
use crate::{ValidateError, ValidateErrorReason};
use lex::lexer::Span;
use proto_packet_parse::ImportTree;
use proto_packet_tree::{Import, ModPath, TypeName};

/// Validates the import `tree`.
pub fn validate_import(tree: &ImportTree, source: &str) -> Result<Import, ValidateError> {
    let (type_name, mod_names): (&Span, &[Span]) = match tree.name.idents.split_last() {
        Some((type_name, mod_names)) if !mod_names.is_empty() => (type_name, mod_names),
        _ => {
            return Err(ValidateError::new(
                tree.name.span(),
                ValidateErrorReason::ImportWithoutModPath,
            ));
        }
    };

    let mod_path_text: String = mod_names
        .iter()
        .map(|mod_name| mod_name.text(source))
        .collect::<Vec<&str>>()
        .join(".");
    let mod_path_span: Span = mod_names[0].combine(mod_names[mod_names.len() - 1]);
    let mod_path: ModPath = validate_name_text(&mod_path_text, mod_path_span)?;
    let type_name: TypeName = validate_name(*type_name, source)?;
    Ok(Import::new(mod_path.to_ref(), type_name.to_ref()))
}
