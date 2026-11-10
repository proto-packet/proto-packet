use crate::parse::expect::{expect, expect_end};
use crate::parse::var::{parse_type_modifier, reject_type_modifier};
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

    /// An array of the `len` of the `element` type, with the span of its `[N]`. (ex: `[16]u8`)
    Array {
        brackets: Span,
        len: Span,
        element: Box<TypeTagTree>,
    },

    /// An optional `value`, with the span of its keyword. (ex: the `optional u8` of `[]optional u8`)
    Optional {
        keyword: Span,
        value: Box<TypeTagTree>,
    },

    /// A map of the `key` type to the `value` type, with its whole span. (ex: `map<u8, string>`)
    Map {
        span: Span,
        key: Box<TypeTagTree>,
        value: Box<TypeTagTree>,
    },

    /// A decimal with the `scale` count of fraction digits, with its whole span. (ex: `decimal<2>`)
    Decimal { span: Span, scale: Span },
}

impl TypeTagTree {
    //! Properties

    /// Gets the span.
    pub fn span(&self) -> Span {
        match self {
            Self::Name(name) => name.span(),
            Self::List { brackets, element }
            | Self::Array {
                brackets, element, ..
            } => brackets.combine(element.span()),
            Self::Optional { keyword, value } => keyword.combine(value.span()),
            Self::Map { span, .. } | Self::Decimal { span, .. } => *span,
        }
    }
}

/// Parses a type tag.
pub fn parse_type_tag(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    if let Some(open) = p.accept(Kind::LBracket) {
        let open: Span = open.span();
        let len: Option<Span> = p.accept(Kind::Integer).map(|len| len.span());
        let close: Span = expect_end(p, Kind::RBracket, ParseErrorReason::ExpectedCloseBracket)?;
        let brackets: Span = open.combine(close);
        let element: Box<TypeTagTree> = Box::new(parse_element(p)?);
        return Ok(match len {
            Some(len) => TypeTagTree::Array {
                brackets,
                len,
                element,
            },
            None => TypeTagTree::List { brackets, element },
        });
    }
    if is_generic(p, "map") {
        return parse_map(p);
    }
    if is_generic(p, "decimal") {
        return parse_decimal(p);
    }
    let name: NameTree = parse_name(p, ParseErrorReason::ExpectedTypeTag)?;
    Ok(TypeTagTree::Name(name))
}

/// Parses the type tag of an element of a list or an array, or of a value of a map.
///
/// # Note
/// - Only these can be `optional`, since an optional field has its own keyword.
fn parse_element(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    let keyword: Span = p.peek().span();
    if !parse_type_modifier(p, "optional") {
        return parse_type_tag(p);
    }
    reject_type_modifier(p, "optional", ParseErrorReason::DuplicateOptional)?;
    Ok(TypeTagTree::Optional {
        keyword,
        value: Box::new(parse_type_tag(p)?),
    })
}

/// Checks if the generic type with the `keyword` is next. (ex: `map`)
///
/// # Note
/// - It is only a keyword before a `<`, so `map` is a mod name in `map.Foo`.
fn is_generic(p: &Parser<'_, Kind>, keyword: &str) -> bool {
    p.check_text(Kind::Ident, keyword)
        && p.lookahead(1).is_some_and(|next| next.kind() == Kind::Less)
}

/// Parses a decimal.
fn parse_decimal(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    let keyword: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeTag)?;
    expect(p, Kind::Less, ParseErrorReason::ExpectedTypeTag)?;
    let scale: Span = expect(p, Kind::Integer, ParseErrorReason::ExpectedDecimalScale)?;
    let close: Span = expect_end(p, Kind::Greater, ParseErrorReason::ExpectedCloseAngle)?;
    Ok(TypeTagTree::Decimal {
        span: keyword.combine(close),
        scale,
    })
}

/// Parses a map.
fn parse_map(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    let keyword: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeTag)?;
    expect(p, Kind::Less, ParseErrorReason::ExpectedTypeTag)?;
    reject_type_modifier(p, "optional", ParseErrorReason::OptionalMapKey)?;
    let key: TypeTagTree = parse_type_tag(p)?;
    expect_end(p, Kind::Comma, ParseErrorReason::ExpectedComma)?;
    let value: TypeTagTree = parse_element(p)?;
    let close: Span = expect_end(p, Kind::Greater, ParseErrorReason::ExpectedCloseAngle)?;
    Ok(TypeTagTree::Map {
        span: keyword.combine(close),
        key: Box::new(key),
        value: Box::new(value),
    })
}
