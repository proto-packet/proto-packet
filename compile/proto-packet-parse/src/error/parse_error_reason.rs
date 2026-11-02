/// The reason for a parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParseErrorReason {
    /// Expected a name. (ex: after a `.`)
    ExpectedName,

    /// Expected a type tag.
    ExpectedTypeTag,

    /// Expected the field's name.
    ExpectedFieldName,

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
            Self::ExpectedFieldName => "expected the field's name",
            Self::ExpectedColon => "expected a `:`",
            Self::ExpectedSemicolon => "expected a `;`",
        }
    }

    /// Gets the error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExpectedName => "P_EXPECTED_NAME",
            Self::ExpectedTypeTag => "P_EXPECTED_TYPE_TAG",
            Self::ExpectedFieldName => "P_EXPECTED_FIELD_NAME",
            Self::ExpectedColon => "P_EXPECTED_COLON",
            Self::ExpectedSemicolon => "P_EXPECTED_SEMICOLON",
        }
    }
}
