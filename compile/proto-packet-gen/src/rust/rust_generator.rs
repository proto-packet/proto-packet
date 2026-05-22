use crate::rust::GenRust;
use crate::{Generator, Writer};
use clerr::Report;
use proto_packet_tree::Project;

/// The [Generator] implementation for Rust code.
#[derive(Copy, Clone, Debug, Default)]
#[non_exhaustive]
pub struct RustGenerator;

impl Generator for RustGenerator {
    fn generate(&self, project: &Project, writer: &impl Writer) -> Result<(), Report> {
        let generator: GenRust = GenRust::new(project);
        generator.write_mod_files(writer)?;
        generator.write_type_decs(writer)?;
        Ok(())
    }
}
