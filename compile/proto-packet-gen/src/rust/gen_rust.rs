use proto_packet_tree::Project;

/// Responsible for generating rust code.
#[derive(Copy, Clone, Debug)]
pub struct GenRust<'a> {
    pub(super) project: &'a Project,
}

impl<'a> GenRust<'a> {
    //! Construction

    /// Creates a new [RustGen].
    pub const fn new(project: &'a Project) -> Self {
        Self { project }
    }
}
