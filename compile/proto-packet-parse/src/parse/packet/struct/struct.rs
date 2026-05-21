use crate::parse::{StructFieldTree, parse_struct_field};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed struct.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub fields: Vec<StructFieldTree>,
}

impl StructTree {
    //! Fields

    /// Gets the fields with the `field_name`.
    pub fn fields_with_name(&self, field_name: &str, source: &str) -> Vec<&StructFieldTree> {
        self.fields
            .iter()
            .filter(|f| f.field_name.text(source) == field_name)
            .collect()
    }
}

/// Parses a struct.
pub fn parse_struct(p: &mut Parser<Kind>) -> Result<StructTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    p.expect(Kind::Struct).ok_or_else(|| {
        ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDeclaration)
    })?;
    let type_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDecName))?
        .span();
    p.expect(Kind::LBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedOpenBrace))?;
    let mut fields: Vec<StructFieldTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        fields.push(parse_struct_field(p)?);
    }
    p.expect(Kind::RBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCloseBrace))?;
    Ok(StructTree {
        comments,
        type_name,
        fields,
    })
}
