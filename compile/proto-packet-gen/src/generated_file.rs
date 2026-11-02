/// A generated source file.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct GeneratedFile {
    /// The file path, relative to the generated source folder. (ex: `a/b/foo.rs`)
    pub path: String,

    /// The source code.
    pub code: String,
}
