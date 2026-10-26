use crate::Kind;
use lex::parser::Parser;

/// Parses the type modifier `keyword` if present. (ex: `optional` or `box`)
///
/// It is only a keyword when a type follows, so `optional` is a mod name in `optional.Foo`.
pub(crate) fn parse_type_modifier(p: &mut Parser<'_, Kind>, keyword: &str) -> bool {
    let is_keyword: bool = p.check_text(Kind::Ident, keyword)
        && p.lookahead(1)
            .is_some_and(|next| matches!(next.kind(), Kind::Ident | Kind::LBracket));
    if is_keyword {
        p.advance();
    }
    is_keyword
}
