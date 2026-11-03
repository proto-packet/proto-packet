use crate::{
    ImportTree, Kind, ParseError, ParseErrorReason, TypeDecTree, parse_import, parse_type_dec,
    parser,
};
use lex::parser::Parser;

/// A parsed schema file.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SchemaFileTree {
    pub imports: Vec<ImportTree>,
    pub type_decs: Vec<TypeDecTree>,
}

/// Parses the schema file `source`.
///
/// An error at an unrecognized character is reported as that character. (ex: `é`)
pub fn parse(source: &str) -> Result<SchemaFileTree, ParseError> {
    let mut p: Parser<'_, Kind> = parser(source);
    parse_schema_file(&mut p).map_err(|error| {
        if p.peek().kind() == Kind::Unrecognized {
            ParseError::new(p.peek().span(), ParseErrorReason::UnrecognizedCharacter)
        } else {
            error
        }
    })
}

/// Parses a schema file to the end of the input.
pub fn parse_schema_file(p: &mut Parser<'_, Kind>) -> Result<SchemaFileTree, ParseError> {
    let mut imports: Vec<ImportTree> = Vec::default();
    while p.check_text(Kind::Ident, "import") {
        imports.push(parse_import(p)?);
    }

    let mut type_decs: Vec<TypeDecTree> = Vec::default();
    while !p.check(Kind::EndOfFile) {
        if p.check_text(Kind::Ident, "import") {
            return Err(ParseError::new(
                p.peek().span(),
                ParseErrorReason::ImportAfterTypeDeclaration,
            ));
        }
        type_decs.push(parse_type_dec(p)?);
    }

    Ok(SchemaFileTree { imports, type_decs })
}
