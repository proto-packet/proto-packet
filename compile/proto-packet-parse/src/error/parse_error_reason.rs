/// The reason for a parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParseErrorReason {
    /// A character that isn't part of the schema language. (ex: `é` or `@`)
    UnrecognizedCharacter,

    /// Expected a name. (ex: after a `.`)
    ExpectedName,

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

    /// Expected a `}`. (reported at the token before it)
    ExpectedCloseBrace,

    /// Expected a `]`. (reported at the token before it)
    ExpectedCloseBracket,

    /// Expected a `,` between the key & the value of a map. (reported at the token before it)
    ExpectedComma,

    /// Expected a `>`. (reported at the token before it)
    ExpectedCloseAngle,

    /// Expected the count of fraction digits of a decimal. (ex: the `2` of `decimal<2>`)
    ExpectedDecimalScale,

    /// Expected a `:`.
    ExpectedColon,

    /// Expected a `;`. (reported at the token before it)
    ExpectedSemicolon,

    /// Expected a `=`.
    ExpectedEqual,

    /// Expected an integer tag number.
    ExpectedTagNumber,

    /// An `optional` message field. (they always are)
    OptionalMessageField,

    /// An `optional` variant case.
    OptionalVariantCase,

    /// An `optional` after a `box`. (ex: `box optional Foo`)
    OptionalAfterBox,

    /// A struct field type with `optional` twice.
    DuplicateOptional,
}

impl ParseErrorReason {
    //! Properties

    /// Gets the message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnrecognizedCharacter => "unrecognized character",
            Self::ExpectedName => "expected a name",
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
            Self::ExpectedCloseBrace => "expected a `}` after this",
            Self::ExpectedCloseBracket => "expected a `]` after this",
            Self::ExpectedComma => "expected a `,` after this",
            Self::ExpectedCloseAngle => "expected a `>` after this",
            Self::ExpectedDecimalScale => "expected a count of fraction digits",
            Self::ExpectedColon => "expected a `:`",
            Self::ExpectedSemicolon => "expected a `;` after this",
            Self::ExpectedEqual => "expected a `=`",
            Self::ExpectedTagNumber => "expected an integer tag number",
            Self::OptionalMessageField => {
                "message fields can't be `optional`, since they always are"
            }
            Self::OptionalVariantCase => "variant cases can't be `optional`",
            Self::OptionalAfterBox => "`optional` must come before `box`",
            Self::DuplicateOptional => "`optional` can only appear once",
        }
    }

    /// Gets the error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnrecognizedCharacter => "P_UNRECOGNIZED_CHARACTER",
            Self::ExpectedName => "P_EXPECTED_NAME",
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
            Self::ExpectedCloseBracket => "P_EXPECTED_CLOSE_BRACKET",
            Self::ExpectedComma => "P_EXPECTED_COMMA",
            Self::ExpectedCloseAngle => "P_EXPECTED_CLOSE_ANGLE",
            Self::ExpectedDecimalScale => "P_EXPECTED_DECIMAL_SCALE",
            Self::ExpectedColon => "P_EXPECTED_COLON",
            Self::ExpectedSemicolon => "P_EXPECTED_SEMICOLON",
            Self::ExpectedEqual => "P_EXPECTED_EQUAL",
            Self::ExpectedTagNumber => "P_EXPECTED_TAG_NUMBER",
            Self::OptionalMessageField => "P_OPTIONAL_MESSAGE_FIELD",
            Self::OptionalVariantCase => "P_OPTIONAL_VARIANT_CASE",
            Self::OptionalAfterBox => "P_OPTIONAL_AFTER_BOX",
            Self::DuplicateOptional => "P_DUPLICATE_OPTIONAL",
        }
    }
}
