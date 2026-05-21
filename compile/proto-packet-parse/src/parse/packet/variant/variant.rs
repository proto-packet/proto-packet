use crate::parse::{VariantCaseTree, parse_variant_case};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed variant.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct VariantTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub cases: Vec<VariantCaseTree>,
}

impl VariantTree {
    //! Cases

    /// Gets the cases with the `case_name`.
    pub fn cases_with_name(&self, case_name: &str, source: &str) -> Vec<&VariantCaseTree> {
        self.cases
            .iter()
            .filter(|c| c.case_name.text(source) == case_name)
            .collect()
    }
}

/// Parses a variant.
pub fn parse_variant(p: &mut Parser<Kind>) -> Result<VariantTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    p.expect(Kind::Variant).ok_or_else(|| {
        ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDeclaration)
    })?;
    let type_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDecName))?
        .span();
    p.expect(Kind::LBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedOpenBrace))?;
    let mut cases: Vec<VariantCaseTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        cases.push(parse_variant_case(p)?);
    }
    p.expect(Kind::RBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCloseBrace))?;
    Ok(VariantTree {
        comments,
        type_name,
        cases,
    })
}
