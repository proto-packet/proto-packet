use crate::Writer;
use clerr::Report;
use proto_packet_tree::Project;

/// Responsible for generating code.
pub trait Generator {
    /// Generates the code from the `project`.
    fn generate(&self, project: &Project, writer: &impl Writer) -> Result<(), Report>;
}
