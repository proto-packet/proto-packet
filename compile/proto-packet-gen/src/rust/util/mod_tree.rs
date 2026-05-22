use proto_packet_tree::{ModName, ModPathRef};
use std::collections::HashMap;

/// A module tree.
#[derive(Clone, Debug, Default)]
pub struct ModTree {
    children: HashMap<ModName, ModTree>,
}

impl ModTree {
    //! Construction

    /// Creates a new [ModTree].
    pub fn new() -> Self {
        Self::default()
    }
}

impl ModTree {
    //! Properties

    /// Gets the children keyed by mod name.
    pub fn children(&self) -> impl Iterator<Item = (&ModName, &ModTree)> {
        self.children.iter()
    }

    /// Returns true if this node has no children. Leaf nodes are schema files.
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

impl ModTree {
    //! Mutations

    /// Inserts the `schema_path`, creating intermediate nodes as needed. The deepest node is the
    /// schema file (a leaf).
    pub fn insert_schema_file(&mut self, schema_path: ModPathRef) {
        let mut current: &mut ModTree = self;
        for mod_name in schema_path.mod_names() {
            current = current.children.entry(mod_name.into_owned()).or_default();
        }
    }
}
