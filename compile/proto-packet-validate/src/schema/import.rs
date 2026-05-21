use crate::ValidateError;
use proto_packet_parse::ImportTree;
use proto_packet_tree::{Import, ModName, QualifiedName, TypeName};

/// Validates the import `tree`.
pub fn validate_import(tree: &ImportTree, source: &str) -> Result<Import, ValidateError> {
    let mut iter = tree.idents.iter().peekable();
    while let Some(span) = iter.next() {
        let text: &str = span.text(source);
        if iter.peek().is_some() {
            ModName::validate(text).map_err(|e| ValidateError::InvalidName {
                span: *span,
                message: e.message(),
            })?;
        } else {
            TypeName::validate(text).map_err(|e| ValidateError::InvalidName {
                span: *span,
                message: e.message(),
            })?;
        }
    }
    let qualified_name_text: String = tree
        .idents
        .iter()
        .map(|s| s.text(source))
        .collect::<Vec<_>>()
        .join(".");
    let name: QualifiedName =
        QualifiedName::new(qualified_name_text).unwrap_or_else(|_| unreachable!());
    Ok(Import::from(name))
}
