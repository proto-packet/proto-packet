/// The reason for a parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParseErrorReason {
    /// Expected a name. (ex: after a `.`)
    ExpectedName,

    /// Expected a `]`.
    ExpectedCloseBracket,

    /// Expected a `>`.
    ExpectedCloseAngle,

    /// Expected a type tag.
    ExpectedTypeTag,

    /// Expected an import.
    ExpectedImport,

    /// An import after a type declaration. (imports come first)
    ImportAfterTypeDeclaration,

    /// Expected a type declaration. (ex: `struct`)
    ExpectedTypeDeclaration,

    /// Expected the type's name.
    ExpectedTypeName,

    /// Expected the field's name.
    ExpectedFieldName,

    /// Expected the case's name.
    ExpectedCaseName,

    /// Expected the call's name.
    ExpectedCallName,

    /// Expected the `returns` keyword.
    ExpectedReturns,

    /// Expected a `{`.
    ExpectedOpenBrace,

    /// Expected a `}`.
    ExpectedCloseBrace,

    /// Expected a `:`.
    ExpectedColon,

    /// Expected a `,`.
    ExpectedComma,

    /// Expected a `;`.
    ExpectedSemicolon,

    /// Expected a `=`.
    ExpectedEqual,

    /// Expected an integer tag number.
    ExpectedTagNumber,
}

impl ParseErrorReason {
    //! Properties

    /// Gets the message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::ExpectedName => "expected a name",
            Self::ExpectedCloseBracket => "expected a `]`",
            Self::ExpectedCloseAngle => "expected a `>`",
            Self::ExpectedTypeTag => "expected a type",
            Self::ExpectedImport => "expected an import",
            Self::ImportAfterTypeDeclaration => "imports must come before type declarations",
            Self::ExpectedTypeDeclaration => "expected a type declaration",
            Self::ExpectedTypeName => "expected the type's name",
            Self::ExpectedFieldName => "expected the field's name",
            Self::ExpectedCaseName => "expected the case's name",
            Self::ExpectedCallName => "expected the call's name",
            Self::ExpectedReturns => "expected the `returns` keyword",
            Self::ExpectedOpenBrace => "expected a `{`",
            Self::ExpectedCloseBrace => "expected a `}`",
            Self::ExpectedColon => "expected a `:`",
            Self::ExpectedComma => "expected a `,`",
            Self::ExpectedSemicolon => "expected a `;`",
            Self::ExpectedEqual => "expected a `=`",
            Self::ExpectedTagNumber => "expected an integer tag number",
        }
    }

    /// Gets the error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExpectedName => "P_EXPECTED_NAME",
            Self::ExpectedCloseBracket => "P_EXPECTED_CLOSE_BRACKET",
            Self::ExpectedCloseAngle => "P_EXPECTED_CLOSE_ANGLE",
            Self::ExpectedTypeTag => "P_EXPECTED_TYPE_TAG",
            Self::ExpectedImport => "P_EXPECTED_IMPORT",
            Self::ImportAfterTypeDeclaration => "P_IMPORT_AFTER_TYPE_DECLARATION",
            Self::ExpectedTypeDeclaration => "P_EXPECTED_TYPE_DECLARATION",
            Self::ExpectedTypeName => "P_EXPECTED_TYPE_NAME",
            Self::ExpectedFieldName => "P_EXPECTED_FIELD_NAME",
            Self::ExpectedCaseName => "P_EXPECTED_CASE_NAME",
            Self::ExpectedCallName => "P_EXPECTED_CALL_NAME",
            Self::ExpectedReturns => "P_EXPECTED_RETURNS",
            Self::ExpectedOpenBrace => "P_EXPECTED_OPEN_BRACE",
            Self::ExpectedCloseBrace => "P_EXPECTED_CLOSE_BRACE",
            Self::ExpectedColon => "P_EXPECTED_COLON",
            Self::ExpectedComma => "P_EXPECTED_COMMA",
            Self::ExpectedSemicolon => "P_EXPECTED_SEMICOLON",
            Self::ExpectedEqual => "P_EXPECTED_EQUAL",
            Self::ExpectedTagNumber => "P_EXPECTED_TAG_NUMBER",
        }
    }
}
