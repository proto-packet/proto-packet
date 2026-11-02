use crate::{Kind, NameTree, ParseError, ParseErrorReason, parse_name};
use lex::parser::Parser;

/// A parsed type tag.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeTagTree {
    /// A named type. (ex: `u8` or `a.b.Foo`)
    Name(NameTree),
}

/// Parses a type tag.
pub fn parse_type_tag(p: &mut Parser<'_, Kind>) -> Result<TypeTagTree, ParseError> {
    let name: NameTree = parse_name(p, ParseErrorReason::ExpectedTypeTag)?;
    Ok(TypeTagTree::Name(name))
}
