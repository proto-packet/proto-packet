use crate::parse::expect::{expect, expect_keyword, parse_members};
use crate::{Kind, ParseError, ParseErrorReason, ServiceCallTree, parse_service_call};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed service.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ServiceTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub calls: Vec<ServiceCallTree>,
}

/// Parses a service.
pub fn parse_service(p: &mut Parser<'_, Kind>) -> Result<ServiceTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    expect_keyword(p, "service", ParseErrorReason::ExpectedTypeDeclaration)?;
    let type_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeName)?;
    let calls: Vec<ServiceCallTree> = parse_members(p, parse_service_call)?;
    Ok(ServiceTree {
        comments,
        type_name,
        calls,
    })
}
