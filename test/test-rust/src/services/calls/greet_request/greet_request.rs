/// ```pps
/// // A greeting request.
/// message GreetRequest {
///    
///     // The name to greet.
///     name: string = 1;
/// }
/// ```
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Default,
    proto_packet::serde::Serialize,
    proto_packet::serde::Deserialize,
)]
#[serde(crate = "proto_packet::serde")]
pub struct GreetRequest {
    name: Option<String>,
}

impl GreetRequest {
    //! Field: `name`

    /// Gets the field: `name`.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets the field: `name`. Returns the previous value.
    pub fn set_name(&mut self, name: Option<String>) -> Option<String> {
        std::mem::replace(&mut self.name, name)
    }

    /// Sets the field: `name`. Returns the struct itself.
    #[must_use]
    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.set_name(name);
        self
    }
}
