/// ```pps
/// // A greeting response.
/// message GreetResponse {
///    
///     // The full greeting text.
///     greeting: string = 1;
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
pub struct GreetResponse {
    greeting: Option<String>,
}

impl GreetResponse {
    //! Field: `greeting`

    /// Gets the field: `greeting`.
    #[must_use]
    pub fn greeting(&self) -> Option<&str> {
        self.greeting.as_deref()
    }

    /// Sets the field: `greeting`. Returns the previous value.
    pub fn set_greeting<F>(&mut self, greeting: F) -> Option<String>
    where
        F: Into<Option<String>>,
    {
        let greeting: Option<String> = greeting.into();
        std::mem::replace(&mut self.greeting, greeting)
    }

    /// Sets the field: `greeting`. Returns the struct itself.
    #[must_use]
    pub fn with_greeting<F>(mut self, greeting: F) -> Self
    where
        F: Into<Option<String>>,
    {
        self.set_greeting(greeting);
        self
    }
}
