use crate::{Kind, NameTree, ParseError, ParseErrorReason, parse_name};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed type tag.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeTagTree {
    /// A named type. (ex: `u8` or `a.b.Foo`)
    Name(NameTree),
}

impl TypeTagTree {
    //! Properties

    /// Gets the span.
    pub fn span(&self) -> Span {
        match self {
            Self::Name(name) => name.span(),
        }
    }
}

/// Parses a type tag.
pub fn parse_type_tag(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    let name: NameTree = parse_name(p, ParseErrorReason::ExpectedTypeTag)?;
    Ok(TypeTagTree::Name(name))
}
