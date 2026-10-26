use crate::parse::expect::expect;
use crate::{Kind, ParseError, ParseErrorReason};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed name. (ex: `Foo` or `a.b.Foo`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct NameTree {
    pub idents: Vec<Span>,
}

/// Parses a name, failing with the `reason` if there is no first identifier.
pub fn parse_name(
    p: &mut Parser<'_, Kind>,
    reason: ParseErrorReason,
) -> Result<NameTree, ParseError> {
    let mut idents: Vec<Span> = vec![expect(p, Kind::Ident, reason)?];
    while p.accept(Kind::Dot).is_some() {
        idents.push(expect(p, Kind::Ident, ParseErrorReason::ExpectedName)?);
    }
    Ok(NameTree { idents })
}
