use crate::parse::expect::{expect, expect_keyword};
use crate::{Kind, NameTree, ParseError, ParseErrorReason, parse_name};
use lex::parser::Parser;

/// A parsed import. (ex: `import the.mod.path.TypeName;`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ImportTree {
    pub name: NameTree,
}

/// Parses an import.
pub fn parse_import(p: &mut Parser<'_, Kind>) -> Result<ImportTree, ParseError> {
    expect_keyword(p, "import", ParseErrorReason::ExpectedImport)?;
    let name: NameTree = parse_name(p, ParseErrorReason::ExpectedName)?;
    expect(p, Kind::Semi, ParseErrorReason::ExpectedSemicolon)?;
    Ok(ImportTree { name })
}
