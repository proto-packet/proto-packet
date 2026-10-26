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
}

/// Parses a type tag.
pub fn parse_type_tag(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    if p.accept(Kind::LBracket).is_some() {
        expect(p, Kind::RBracket, ParseErrorReason::ExpectedCloseBracket)?;
        let base: TypeTagTree = parse_type_tag(p)?;
        Ok(TypeTagTree::Slice {
            base: Box::new(base),
        })
    } else {
        let name: NameTree = parse_name(p, ParseErrorReason::ExpectedTypeTag)?;
        Ok(TypeTagTree::Name(name))
    }
}
