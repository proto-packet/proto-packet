use crate::parse::expect::expect;
use crate::{Kind, NameTree, ParseError, ParseErrorReason, parse_name};
use lex::parser::Parser;

/// A parsed type tag.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeTagTree {
    /// A named type. (ex: `u8` or `a.b.Foo`)
    Name(NameTree),

    /// A slice type. (ex: `[]u8`)
    Slice { base: Box<TypeTagTree> },

    /// A map type. (ex: `map<string, u8>`)
    Map {
        key: Box<TypeTagTree>,
        value: Box<TypeTagTree>,
    },
}

/// Parses a type tag.
///
/// The `map` is only a keyword when a `<` follows, so `map` is a mod name in `map.Foo`.
pub fn parse_type_tag(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    if p.accept(Kind::LBracket).is_some() {
        expect(p, Kind::RBracket, ParseErrorReason::ExpectedCloseBracket)?;
        let base: TypeTagTree = parse_type_tag(p)?;
        Ok(TypeTagTree::Slice {
            base: Box::new(base),
        })
    } else if p.check_text(Kind::Ident, "map")
        && p.lookahead(1)
            .is_some_and(|next| matches!(next.kind(), Kind::LAngle))
    {
        p.advance();
        p.advance();
        let key: TypeTagTree = parse_type_tag(p)?;
        expect(p, Kind::Comma, ParseErrorReason::ExpectedComma)?;
        let value: TypeTagTree = parse_type_tag(p)?;
        expect(p, Kind::RAngle, ParseErrorReason::ExpectedCloseAngle)?;
        Ok(TypeTagTree::Map {
            key: Box::new(key),
            value: Box::new(value),
        })
    } else {
        let name: NameTree = parse_name(p, ParseErrorReason::ExpectedTypeTag)?;
        Ok(TypeTagTree::Name(name))
    }
}
