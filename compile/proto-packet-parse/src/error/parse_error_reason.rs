/// The reason for a parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParseErrorReason {
    /// Expected a name. (ex: after a `.`)
    ExpectedName,

    /// Expected a type tag.
    ExpectedTypeTag,

    /// Expected a type declaration. (ex: `struct`)
    ExpectedTypeDeclaration,

    /// Expected the type's name.
    ExpectedTypeName,

    /// Expected the field's name.
    ExpectedFieldName,

    /// Expected a `{`.
    ExpectedOpenBrace,

    /// Expected a `}`.
    ExpectedCloseBrace,

    /// Expected a `:`.
    ExpectedColon,

    /// Expected a `;`.
    ExpectedSemicolon,
}

impl ParseErrorReason {
    //! Properties

    /// Gets the message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::ExpectedName => "expected a name",
            Self::ExpectedTypeTag => "expected a type",
            Self::ExpectedTypeDeclaration => "expected a type declaration",
            Self::ExpectedTypeName => "expected the type's name",
            Self::ExpectedFieldName => "expected the field's name",
            Self::ExpectedOpenBrace => "expected a `{`",
            Self::ExpectedCloseBrace => "expected a `}`",
            Self::ExpectedColon => "expected a `:`",
            Self::ExpectedSemicolon => "expected a `;`",
        }
    }

    /// Gets the error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExpectedName => "P_EXPECTED_NAME",
            Self::ExpectedTypeTag => "P_EXPECTED_TYPE_TAG",
            Self::ExpectedTypeDeclaration => "P_EXPECTED_TYPE_DECLARATION",
            Self::ExpectedTypeName => "P_EXPECTED_TYPE_NAME",
            Self::ExpectedFieldName => "P_EXPECTED_FIELD_NAME",
            Self::ExpectedOpenBrace => "P_EXPECTED_OPEN_BRACE",
            Self::ExpectedCloseBrace => "P_EXPECTED_CLOSE_BRACE",
            Self::ExpectedColon => "P_EXPECTED_COLON",
            Self::ExpectedSemicolon => "P_EXPECTED_SEMICOLON",
        }
    }
}
