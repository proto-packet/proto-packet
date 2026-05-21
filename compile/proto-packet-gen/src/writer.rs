use clerr::Report;
use code_gen::Block;

/// Responsible for writing generated code.
pub trait Writer {
    /// Writes the `source` code to the `file_path`.
    fn write(&self, source: &Block, file_path: &str) -> Result<(), Report>;
}
