use lex::lexer::matchers::{digits, ident, whitespace};
use lex::lexer::{Lexer, Token};
use lex::{keyword, lexer, line_comment, literal};
use std::sync::LazyLock;

/// Lexer the `schema` into tokens.
pub fn lex(schema: &str) -> Vec<Token<Kind>> {
    static LEXER: LazyLock<Lexer<Kind>> = LazyLock::new(Kind::lexer);
    LEXER.lex(schema)
}

lexer! {
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub enum Kind {
        LineComment: line_comment!("//"),
        Whitespace: whitespace,

        Import: keyword!("import"),

        Struct: keyword!("struct"),
        Message: keyword!("message"),
        Enum: keyword!("enum"),
        Variant: keyword!("variant"),
        Service: keyword!("service"),
        Returns: keyword!("returns"),

        Optional: keyword!("optional"),

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
