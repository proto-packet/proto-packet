use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed import.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ImportTree {
    pub comments: Vec<Span>,
    pub idents: Vec<Span>,
}

/// Parses an import.
pub fn parse_import(p: &mut Parser<Kind>) -> Result<ImportTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    p.expect(Kind::Import).ok_or_else(|| {
        ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDeclaration)
    })?;
    let mut idents: Vec<Span> = Vec::default();
    let first: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedImportIdent))?
        .span();
    idents.push(first);
    while p.check(Kind::Dot) {
        p.advance();
        let next: Span = p
            .expect(Kind::Ident)
            .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedImportIdent))?
            .span();
        idents.push(next);
    }
    p.expect(Kind::Semi)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedSemicolon))?;
    Ok(ImportTree { comments, idents })
}
