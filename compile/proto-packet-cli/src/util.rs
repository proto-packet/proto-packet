use clerr::{Code, Properties, Report};
use std::path::Path;

/// Reads the file at `path` to a string.
pub fn read_file(path: &str) -> Result<String, Report> {
    std::fs::read_to_string(path).map_err(|e| {
        Report::from(Code::error("CLI_READ", "failed to read file")).with_entry(
            Properties::default()
                .with_property("path", path)
                .with_property("error", e.to_string()),
        )
    })
}

/// Gets the file name portion of the `path` (best effort, falls back to the path itself).
pub fn file_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
}
