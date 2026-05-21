use crate::parse::parse_schema_file;
use crate::{Kind, SchemaFileTree, lex};
use ::lex::lexer::Token;
use ::lex::parser::Parser;
use clerr::Report;

/// Parses the schema `source`.
pub fn parse(file_name: &str, source: String) -> Result<SchemaFileTree, Report> {
    let tokens: Vec<Token<Kind>> = lex(source.as_str());
    let mut parser: Parser<Kind> = Parser::new(source.clone(), tokens)
        .with_skip(Kind::Whitespace)
        .with_skip(Kind::LineComment)
        .with_line_comment(Kind::LineComment, "//");
    parse_schema_file(&mut parser).map_err(|e| e.report(file_name, source.as_str()))
}
