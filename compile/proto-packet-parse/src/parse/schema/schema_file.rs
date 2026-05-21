use crate::parse::{ImportTree, TypeDecTree, parse_import, parse_type_dec};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed schema file.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct SchemaFileTree {
    pub comments: Vec<Span>,
    pub imports: Vec<ImportTree>,
    pub type_decs: Vec<TypeDecTree>,
}

impl SchemaFileTree {
    //! Type Declarations

    /// Gets the type declarations with the `name`.
    pub fn type_decs_with_name<N>(&self, name: N, source: &str) -> Vec<&TypeDecTree>
    where
        N: AsRef<str>,
    {
        self.type_decs
            .iter()
            .filter(|d| d.type_name().text(source) == name.as_ref())
            .collect()
    }
}

/// Parses a schema file to the end of input.
pub fn parse_schema_file(p: &mut Parser<Kind>) -> Result<SchemaFileTree, ParseError> {
    let mut imports: Vec<ImportTree> = Vec::default();
    while p.check(Kind::Import) {
        imports.push(parse_import(p)?);
    }
    let mut type_decs: Vec<TypeDecTree> = Vec::default();
    while !p.check(Kind::EndOfFile) {
        if p.check(Kind::Import) {
            return Err(ParseError::new(
                p.peek().span(),
                ParseErrorReason::ImportAfterTypeDeclaration,
            ));
        }
        type_decs.push(parse_type_dec(p)?);
    }
    Ok(SchemaFileTree {
        comments: Vec::default(),
        imports,
        type_decs,
    })
}
