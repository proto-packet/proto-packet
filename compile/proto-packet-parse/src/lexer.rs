use lex::lexer::matchers::{digits, ident, whitespace};
use lex::lexer::{Lexer, Token};
use lex::parser::Parser;
use lex::{lexer, line_comment, literal};
use std::sync::LazyLock;

lexer! {
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub enum Kind {
        LineComment: line_comment!("//"),
        Whitespace: whitespace,

        Ident: ident,
        Integer: digits,

        LBrace: literal!("{"),
        RBrace: literal!("}"),
        LBracket: literal!("["),
        RBracket: literal!("]"),
        Semi: literal!(";"),
        Colon: literal!(":"),
        Dot: literal!("."),
        Equal: literal!("="),
    }
}

/// Lexes the `source` into tokens.
pub fn lex(source: &str) -> Vec<Token<Kind>> {
    static LEXER: LazyLock<Lexer<Kind>> = LazyLock::new(Kind::lexer);
    LEXER.lex(source)
}

/// Creates a parser for the `source`.
///
/// Whitespace & comments are skipped, and comments are available as leading comments.
pub fn parser(source: &str) -> Parser<'_, Kind> {
    Parser::new(source, lex(source))
        .with_skip(Kind::Whitespace)
        .with_skip(Kind::LineComment)
        .with_line_comment(Kind::LineComment, "//")
}
