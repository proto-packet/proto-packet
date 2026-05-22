use crate::Writer;
use crate::rust::{GenRust, ModTree};
use clerr::Report;
use code_gen::{Block, WithStatements};
use proto_packet_tree::{ModName, ModPathRef};
use std::collections::HashMap;

impl GenRust<'_> {
    //! Write: Module Files

    /// Writes the mod files to the `writer`.
    pub(super) fn write_mod_files<W>(self, writer: &W) -> Result<(), Report>
    where
        W: Writer,
    {
        let tree: ModTree = self.create_mod_tree();
        let types: HashMap<String, Vec<ModName>> = self.types_by_dir();
        Self::write_node(writer, &tree, "", &types)
    }

    /// Recursively writes the mod file for `node` and each of its descendants. `dir` is the
    /// directory prefix from the source root (empty for the root). Non-leaf nodes emit a
    /// `pub mod NAME;` for each child. Leaf nodes are schema files: each types in the schema
    /// file is emitted as a `pub use NAME::*; mod NAME;` pair, and each type also gets its own
    /// type-dec dir mod file flattening the type file.
    fn write_node<W>(
        writer: &W,
        node: &ModTree,
        dir: &str,
        types: &HashMap<String, Vec<ModName>>,
    ) -> Result<(), Report>
    where
        W: Writer,
    {
        if node.is_leaf() {
            let names: &[ModName] = types.get(dir).map(Vec::as_slice).unwrap_or(&[]);
            Self::write_block(writer, &Self::flatten_block(names), dir)?;
            for name in names {
                let type_dir: String = format!("{dir}{name}/");
                Self::write_block(
                    writer,
                    &Self::flatten_block(std::slice::from_ref(name)),
                    &type_dir,
                )?;
            }
        } else {
            let block: Block = Self::pub_mod_block(node);
            Self::write_block(writer, &block, dir)?;
            for (child_name, child) in node.children() {
                let child_dir: String = format!("{dir}{child_name}/");
                Self::write_node(writer, child, &child_dir, types)?;
            }
        }
        Ok(())
    }

    /// Builds a `pub mod NAME;` block for the `node`'s children.
    fn pub_mod_block(node: &ModTree) -> Block {
        let mut block: Block = Block::default();
        for (child_name, _) in node.children() {
            block.add_semi(format!("pub mod {child_name}"));
        }
        block
    }

    /// Builds a flatten block (`pub use NAME::*;` for each, then a blank line, then `mod NAME;`
    /// for each).
    fn flatten_block(names: &[ModName]) -> Block {
        let mut block: Block = Block::default();
        for name in names {
            block.add_semi(format!("pub use {name}::*"));
        }
        block.add_empty_line();
        for name in names {
            block.add_semi(format!("mod {name}"));
        }
        block
    }

    /// Writes the `block` as the mod file at `dir`. Uses `lib.rs` at the root and `mod.rs`
    /// otherwise.
    fn write_block<W>(writer: &W, block: &Block, dir: &str) -> Result<(), Report>
    where
        W: Writer,
    {
        let file_name: &str = if dir.is_empty() { "lib.rs" } else { "mod.rs" };
        let path: String = format!("{dir}{file_name}");
        writer.write(block, &path)
    }

    /// Creates the [ModTree] of schema-file leaves under intermediate directory nodes.
    fn create_mod_tree(&self) -> ModTree {
        let mut tree: ModTree = ModTree::new();
        for (schema_path, _) in self.project.schema_files() {
            tree.insert_schema_file(schema_path.to_ref());
        }
        tree
    }

    /// Builds a lookup from schema-file directory (e.g., `"structs/fields/"`) to the type mod
    /// names declared in that schema file.
    fn types_by_dir(&self) -> HashMap<String, Vec<ModName>> {
        let mut map: HashMap<String, Vec<ModName>> = HashMap::new();
        for (schema_path, schema_file) in self.project.schema_files() {
            let dir: String = Self::mod_path_to_dir(schema_path.to_ref());
            let names: Vec<ModName> = schema_file
                .type_decs()
                .iter()
                .map(|td| self.mod_name(td))
                .collect();
            map.insert(dir, names);
        }
        map
    }

    /// Joins `path`'s mod names with `/`, ending with a trailing `/`. Empty path yields `""`.
    fn mod_path_to_dir(path: ModPathRef) -> String {
        let mut s: String = String::new();
        for name in path.mod_names() {
            s.push_str(name.as_ref());
            s.push('/');
        }
        s
    }
}
