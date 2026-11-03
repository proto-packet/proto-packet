use crate::{Kind, ParseError, ParseErrorReason};
use lex::parser::Parser;

/// Parses the type modifier `keyword` if present. (ex: `optional` or `box`)
///
/// It is only a keyword when a type follows, so `optional` is a mod name in `optional.Foo`.
pub(crate) fn parse_type_modifier(p: &mut Parser<'_, Kind>, keyword: &str) -> bool {
    let is_keyword: bool = is_type_modifier(p, keyword);
    if is_keyword {
        p.advance();
    }
    is_keyword
}

/// Fails with the `reason` if the type modifier `keyword` is present. (ex: `optional` in a message)
pub(crate) fn reject_type_modifier(
    p: &Parser<'_, Kind>,
    keyword: &str,
    reason: ParseErrorReason,
) -> Result<(), ParseError> {
    if is_type_modifier(p, keyword) {
        Err(ParseError::new(p.peek().span(), reason))
    } else {
        Ok(())
    }
}

/// Checks if the type modifier `keyword` is next.
fn is_type_modifier(p: &Parser<'_, Kind>, keyword: &str) -> bool {
    p.check_text(Kind::Ident, keyword)
        && p.lookahead(1)
            .is_some_and(|next| next.kind() == Kind::Ident)
}
