use crate::parse::{EnumCaseTree, parse_enum_case};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed enum.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EnumTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub cases: Vec<EnumCaseTree>,
}

impl EnumTree {
    //! Cases

    /// Gets the cases with the `case_name`.
    pub fn cases_with_name(&self, case_name: &str, source: &str) -> Vec<&EnumCaseTree> {
        self.cases
            .iter()
            .filter(|c| c.case_name.text(source) == case_name)
            .collect()
    }
}

/// Parses an enum.
pub fn parse_enum(p: &mut Parser<Kind>) -> Result<EnumTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    p.expect(Kind::Enum).ok_or_else(|| {
        ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDeclaration)
    })?;
    let type_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDecName))?
        .span();
    p.expect(Kind::LBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedOpenBrace))?;
    let mut cases: Vec<EnumCaseTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        cases.push(parse_enum_case(p)?);
    }
    p.expect(Kind::RBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCloseBrace))?;
    Ok(EnumTree {
        comments,
        type_name,
        cases,
    })
}
