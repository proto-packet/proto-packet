use crate::LineIndex;
use lex::lexer::Span;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};
use proto_packet_parse::SchemaFileTree;

/// The source of the diagnostics, which an editor shows beside each one.
const SOURCE: &str = "packet";

/// Gets the diagnostics of the schema file `source`.
///
/// # Note
/// - The file is checked alone, so its imports & the names of other files aren't resolved.
/// - The check stops at the first error, so there is at most one diagnostic.
pub(crate) fn diagnostics(source: &str) -> Vec<Diagnostic> {
    let tree: SchemaFileTree = match proto_packet_parse::parse(source) {
        Ok(tree) => tree,
        Err(error) => {
            let code: &str = error.reason().code();
            return vec![diagnostic(source, error.span(), code, error.to_string())];
        }
    };
    match proto_packet_validate::validate_schema_file(&tree, source) {
        Ok(_) => Vec::default(),
        Err(error) => {
            let code: &str = error.reason().code();
            vec![diagnostic(source, error.span(), code, error.to_string())]
        }
    }
}

/// Creates the diagnostic of the error with the `code` & the `message` at the `span`.
fn diagnostic(source: &str, span: Span, code: &str, message: String) -> Diagnostic {
    Diagnostic {
        range: LineIndex::new(source).range(span),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(code.to_string())),
        source: Some(SOURCE.to_string()),
        message,
        ..Diagnostic::default()
    }
}
