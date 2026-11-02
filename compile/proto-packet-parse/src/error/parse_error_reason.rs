/// The reason for a parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParseErrorReason {
    /// Expected a name. (ex: after a `.`)
    ExpectedName,
}

impl ParseErrorReason {
    //! Properties

    /// Gets the message.
    pub const fn message(self) -> &'static str {
        match self {
            Self::ExpectedName => "expected a name",
        }
    }

    /// Gets the error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::ExpectedName => "P_EXPECTED_NAME",
        }
    }
}
