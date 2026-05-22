use crate::parse::{TypeTagTree, parse_type_tag};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed service call.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ServiceCallTree {
    pub comments: Vec<Span>,
    pub call_name: Span,
    pub request: TypeTagTree,
    pub response: TypeTagTree,
}

/// Parses a service call.
pub fn parse_service_call(p: &mut Parser<Kind>) -> Result<ServiceCallTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let call_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCallName))?
        .span();
    p.expect(Kind::Colon)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedColon))?;
    let request: TypeTagTree = parse_type_tag(p)?;
    p.expect(Kind::Returns)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedReturns))?;
    let response: TypeTagTree = parse_type_tag(p)?;
    p.expect(Kind::Semi)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedSemicolon))?;
    Ok(ServiceCallTree {
        comments,
        call_name,
        request,
        response,
    })
}
