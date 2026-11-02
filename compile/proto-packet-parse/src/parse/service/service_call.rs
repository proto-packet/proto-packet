use crate::parse::expect::{expect, expect_keyword};
use crate::{Kind, ParseError, ParseErrorReason, TypeTagTree, parse_type_tag};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed service call. (ex: `greet: GreetRequest returns GreetResponse;`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ServiceCallTree {
    pub comments: Vec<Span>,
    pub call_name: Span,
    pub request: TypeTagTree,
    pub response: TypeTagTree,
}

/// Parses a service call.
pub fn parse_service_call(p: &mut Parser<'_, Kind>) -> Result<ServiceCallTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let call_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedCallName)?;
    expect(p, Kind::Colon, ParseErrorReason::ExpectedColon)?;
    let request: TypeTagTree = parse_type_tag(p)?;
    expect_keyword(p, "returns", ParseErrorReason::ExpectedReturns)?;
    let response: TypeTagTree = parse_type_tag(p)?;
    expect(p, Kind::Semi, ParseErrorReason::ExpectedSemicolon)?;
    Ok(ServiceCallTree {
        comments,
        call_name,
        request,
        response,
    })
}
