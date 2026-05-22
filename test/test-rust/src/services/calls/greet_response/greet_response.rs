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
    serde::Serialize,
    serde::Deserialize,
)]
pub struct GreetResponse {
    greeting: Option<String>,
}

impl GreetResponse {
    //! Field: `greeting`

    /// Gets the field: `greeting`.
    #[must_use]
    pub fn greeting(&self) -> Option<&String> {
        self.greeting.as_ref()
    }

    /// Sets the field: `greeting`. Returns the previous value.
    pub fn set_greeting(&mut self, greeting: Option<String>) -> Option<String> {
        std::mem::replace(&mut self.greeting, greeting)
    }

    /// Sets the field: `greeting`. Returns the struct itself.
    #[must_use]
    pub fn with_greeting(mut self, greeting: Option<String>) -> Self {
        self.set_greeting(greeting);
        self
    }
}
