/// The reason for a parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParseErrorReason {
    /// Expected a name. (ex: after a `.`)
    ExpectedName,

    /// Expected a `]`.
    ExpectedCloseBracket,

    /// Expected a type tag.
    ExpectedTypeTag,

    /// Expected an import.
    ExpectedImport,

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
            Self::ExpectedTypeTag => "expected a type",
            Self::ExpectedImport => "expected an import",
            Self::ExpectedTypeDeclaration => "expected a type declaration",
            Self::ExpectedTypeName => "expected the type's name",
            Self::ExpectedFieldName => "expected the field's name",
            Self::ExpectedCaseName => "expected the case's name",
            Self::ExpectedCallName => "expected the call's name",
            Self::ExpectedReturns => "expected the `returns` keyword",
            Self::ExpectedOpenBrace => "expected a `{`",
            Self::ExpectedCloseBrace => "expected a `}`",
            Self::ExpectedColon => "expected a `:`",
            Self::ExpectedSemicolon => "expected a `;`",
            Self::ExpectedEqual => "expected a `=`",
            Self::ExpectedTagNumber => "expected an integer tag number",
        }
    }
}
