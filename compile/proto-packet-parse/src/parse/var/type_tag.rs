use crate::parse::expect::{expect, expect_end};
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

    /// A map of the `key` type to the `value` type, with its whole span. (ex: `map<u8, string>`)
    Map {
        span: Span,
        key: Box<TypeTagTree>,
        value: Box<TypeTagTree>,
    },
}

impl TypeTagTree {
    //! Properties

    /// Gets the span.
    pub fn span(&self) -> Span {
        match self {
            Self::Name(name) => name.span(),
            Self::List { brackets, element } => brackets.combine(element.span()),
            Self::Map { span, .. } => *span,
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
    if is_map(p) {
        return parse_map(p);
    }
    let name: NameTree = parse_name(p, ParseErrorReason::ExpectedTypeTag)?;
    Ok(TypeTagTree::Name(name))
}

/// Checks if a map is next.
///
/// # Note
/// - The `map` is only a keyword before a `<`, so it is a mod name in `map.Foo`.
fn is_map(p: &Parser<'_, Kind>) -> bool {
    p.check_text(Kind::Ident, "map") && p.lookahead(1).is_some_and(|next| next.kind() == Kind::Less)
}

/// Parses a map.
fn parse_map(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    let keyword: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeTag)?;
    expect(p, Kind::Less, ParseErrorReason::ExpectedTypeTag)?;
    let key: TypeTagTree = parse_type_tag(p)?;
    expect_end(p, Kind::Comma, ParseErrorReason::ExpectedComma)?;
    let value: TypeTagTree = parse_type_tag(p)?;
    let close: Span = expect_end(p, Kind::Greater, ParseErrorReason::ExpectedCloseAngle)?;
    Ok(TypeTagTree::Map {
        span: keyword.combine(close),
        key: Box::new(key),
        value: Box::new(value),
    })
}
