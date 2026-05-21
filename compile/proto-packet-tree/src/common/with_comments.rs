/// An element with comment lines.
pub trait WithComments: Sized {
    /// Gets the comment lines.
    fn comments(&self) -> &[String];

    /// Adds the `comment` line.
    fn add_comment<S>(&mut self, comment: S)
    where
        S: Into<String>;

    /// Adds the `comment` line.
    #[must_use]
    fn with_comment<S>(mut self, comment: S) -> Self
    where
        S: Into<String>,
    {
        self.add_comment(comment);
        self
    }
}

/// Implements [WithComments] for types with a `comments: Vec<String>` field.
macro_rules! impl_with_comments {
    ($($type:ty),+ $(,)?) => {
        $(
            impl WithComments for $type {
                fn comments(&self) -> &[String] {
                    self.comments.as_slice()
                }

                fn add_comment<S>(&mut self, comment: S)
                where
                    S: Into<String>,
                {
                    self.comments.push(comment.into());
                }
            }
        )+
    };
}
pub(crate) use impl_with_comments;
