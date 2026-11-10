use crate::parse::expect::expect_end;
use crate::{Kind, NameTree, ParseError, ParseErrorReason, parse_name};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed type tag.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeTagTree {
    /// A named type. (ex: `u8` or `a.b.Foo`)
    Name(NameTree),

    /// A list of the `element` type, with the span of its `[]`. (ex: `[]u8`)
    List {
        brackets: Span,
        element: Box<TypeTagTree>,
    },
}

impl TypeTagTree {
    //! Properties

    /// Gets the span.
    pub fn span(&self) -> Span {
        match self {
            Self::Name(name) => name.span(),
            Self::List { brackets, element } => brackets.combine(element.span()),
        }
    }
}

/// Parses a type tag.
pub fn parse_type_tag(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    if let Some(open) = p.accept(Kind::LBracket) {
        let open: Span = open.span();
        let close: Span = expect_end(p, Kind::RBracket, ParseErrorReason::ExpectedCloseBracket)?;
        let element: TypeTagTree = parse_type_tag(p)?;
        return Ok(TypeTagTree::List {
            brackets: open.combine(close),
            element: Box::new(element),
        });
    }
    let name: NameTree = parse_name(p, ParseErrorReason::ExpectedTypeTag)?;
    Ok(TypeTagTree::Name(name))
}
