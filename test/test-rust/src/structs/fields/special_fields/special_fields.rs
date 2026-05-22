/// ```pps
/// // A struct with special fields.
/// struct SpecialFields {
///    
///     // A `uuid` field.
///     one: uuid;
///    
///     // A `string` field.
///     two: string;
/// }
/// ```
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct SpecialFields {
    one: proto_packet::types::Uuid,
    two: String,
}

impl SpecialFields {
    //! Construction

    /// Creates a new [SpecialFields].
    pub const fn new(one: proto_packet::types::Uuid, two: String) -> Self {
        Self { one, two }
    }

    /// Creates a new [SpecialFields].
    pub fn from<F1, F2>(one: F1, two: F2) -> Self where F1: Into<proto_packet::types::Uuid>, F2: Into<String> {
        Self::new(one.into(), two.into())
    }
}

impl SpecialFields {
    //! Field: `one`

    /// Gets the field: `one`.
    #[must_use]
    pub fn one(&self) -> proto_packet::types::Uuid {
        self.one
    }

    /// Sets the field: `one`. Returns the previous value.
    pub fn set_one(&mut self, one: proto_packet::types::Uuid) -> proto_packet::types::Uuid {
        std::mem::replace(&mut self.one, one)
    }

    /// Sets the field: `one`. Returns the struct itself.
    #[must_use]
    pub fn with_one(mut self, one: proto_packet::types::Uuid) -> Self {
        self.set_one(one);
        self
    }
}

impl SpecialFields {
    //! Field: `two`

    /// Gets the field: `two`.
    #[must_use]
    pub fn two(&self) -> &String {
        &self.two
    }

    /// Sets the field: `two`. Returns the previous value.
    pub fn set_two(&mut self, two: String) -> String {
        std::mem::replace(&mut self.two, two)
    }

    /// Sets the field: `two`. Returns the struct itself.
    #[must_use]
    pub fn with_two(mut self, two: String) -> Self {
        self.set_two(two);
        self
    }
}
