/// The reason for a parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParseErrorReason {
    /// Expected a name. (ex: after a `.`)
    ExpectedName,

    /// Expected a `]`.
    ExpectedCloseBracket,

    /// Expected a type tag.
    ExpectedTypeTag,
}

impl ParseErrorReason {
    //! Properties

    /// Gets the message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::ExpectedName => "expected a name",
            Self::ExpectedCloseBracket => "expected a `]`",
            Self::ExpectedTypeTag => "expected a type",
        }
    }
}
