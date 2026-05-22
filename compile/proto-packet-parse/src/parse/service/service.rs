use crate::parse::{ServiceCallTree, parse_service_call};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed service.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ServiceTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub calls: Vec<ServiceCallTree>,
}

impl ServiceTree {
    //! Calls

    /// Gets the calls with the `call_name`.
    pub fn calls_with_name(&self, call_name: &str, source: &str) -> Vec<&ServiceCallTree> {
        self.calls
            .iter()
            .filter(|c| c.call_name.text(source) == call_name)
            .collect()
    }
}

/// Parses a service.
pub fn parse_service(p: &mut Parser<Kind>) -> Result<ServiceTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    p.expect(Kind::Service).ok_or_else(|| {
        ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDeclaration)
    })?;
    let type_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDecName))?
        .span();
    p.expect(Kind::LBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedOpenBrace))?;
    let mut calls: Vec<ServiceCallTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        calls.push(parse_service_call(p)?);
    }
    p.expect(Kind::RBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCloseBrace))?;
    Ok(ServiceTree {
        comments,
        type_name,
        calls,
    })
}
