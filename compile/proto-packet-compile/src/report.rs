use clerr::{Code, Properties, Report};
use std::fmt::Display;
use std::path::Path;

/// Creates the report for the `error` with the `path`.
pub(crate) fn path_report<E>(code: &str, message: &str, path: &Path, error: E) -> Report
where
    E: Display,
{
    let properties: Properties = Properties::default()
        .with_property("path", path.display().to_string())
        .with_property("error", error.to_string());
    Report::from(Code::error(code, message)).with_entry(properties)
}
