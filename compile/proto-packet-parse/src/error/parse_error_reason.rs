/// The reason a [crate::ParseError] was raised.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ParseErrorReason {
    // Tokens
    /// Expected `{`.
    ExpectedOpenBrace,
    /// Expected `}`.
    ExpectedCloseBrace,
    /// Expected `:`.
    ExpectedColon,
    /// Expected `;`.
    ExpectedSemicolon,
    /// Expected `=`.
    ExpectedEqual,
    /// Expected `]`.
    ExpectedCloseBracket,

    // Type tags
    /// An unrecognized type tag.
    UnrecognizedTypeTag,

    // Type declarations
    /// Expected a type declaration. (e.g. `struct`)
    ExpectedTypeDeclaration,
    /// Expected the type declaration's name.
    ExpectedTypeDecName,

    // Fields and cases
    /// Expected the field's name.
    ExpectedFieldName,
    /// Expected the case's name.
    ExpectedCaseName,
    /// Expected an integer for the tag number.
    ExpectedTagNumber,

    // Schema file
    /// Expected an identifier in the import path.
    ExpectedImportIdent,
    /// An import statement appeared after a type declaration.
    ImportAfterTypeDeclaration,
}

impl ParseErrorReason {
    //! Properties

    /// Gets the human-readable message.
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Self::ExpectedOpenBrace => "expected a `{`",
            Self::ExpectedCloseBrace => "expected a `}`",
            Self::ExpectedColon => "expected a `:`",
            Self::ExpectedSemicolon => "expected a `;`",
            Self::ExpectedEqual => "expected a `=`",
            Self::ExpectedCloseBracket => "expected a `]`",
            Self::UnrecognizedTypeTag => "unrecognized type tag",
            Self::ExpectedTypeDeclaration => "expected a type declaration",
            Self::ExpectedTypeDecName => "expected the type's name",
            Self::ExpectedFieldName => "expected the field's name",
            Self::ExpectedCaseName => "expected the case's name",
            Self::ExpectedTagNumber => "expected an integer tag number",
            Self::ExpectedImportIdent => "expected an identifier in the import path",
            Self::ImportAfterTypeDeclaration => "imports must appear before type declarations",
        }
    }

    /// Gets the short error code suffix.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::ExpectedOpenBrace => "EXPECTED_OPEN_BRACE",
            Self::ExpectedCloseBrace => "EXPECTED_CLOSE_BRACE",
            Self::ExpectedColon => "EXPECTED_COLON",
            Self::ExpectedSemicolon => "EXPECTED_SEMI",
            Self::ExpectedEqual => "EXPECTED_EQUAL",
            Self::ExpectedCloseBracket => "EXPECTED_CLOSE_BRACKET",
            Self::UnrecognizedTypeTag => "UNRECOGNIZED_TYPE_TAG",
            Self::ExpectedTypeDeclaration => "EXPECTED_TYPE_DEC",
            Self::ExpectedTypeDecName => "EXPECTED_TYPE_DEC_NAME",
            Self::ExpectedFieldName => "EXPECTED_FIELD_NAME",
            Self::ExpectedCaseName => "EXPECTED_CASE_NAME",
            Self::ExpectedTagNumber => "EXPECTED_TAG_NUMBER",
            Self::ExpectedImportIdent => "EXPECTED_IMPORT_IDENT",
            Self::ImportAfterTypeDeclaration => "IMPORT_AFTER_TYPE_DEC",
        }
    }
}
