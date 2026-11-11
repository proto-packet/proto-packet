use crate::rust::naming::{ident, is_snake_case, snake_case};
use crate::rust::{TypeFile, gen_enum, gen_message, gen_service, gen_struct, gen_variant};
use crate::{GenError, GeneratedFile};
use proto_packet_tree::{ModPathRef, Project, SchemaFile, TypeDec, WithTypeName};
use std::collections::{BTreeMap, BTreeSet};

/// Generates the Rust source files of the linked `project`.
///
/// The root module is `lib.rs`, each schema file is a module folder with a `mod.rs`, & each type
/// is a module in its schema's folder that the `mod.rs` re-exports. A struct, message or variant is
/// a folder, for the files of its generated types, & any other type is a file.
pub fn generate(project: &Project) -> Result<Vec<GeneratedFile>, GenError> {
    let mut root: ModNode = ModNode::default();
    for (mod_path, schema_file) in project.schema_files() {
        let mut node: &mut ModNode = &mut root;
        for mod_name in mod_path.mod_names() {
            node = node.children.entry(mod_name.to_string()).or_default();
        }
        node.schema_file = Some((mod_path.to_ref(), schema_file));
    }
    let mut files: Vec<GeneratedFile> = Vec::default();
    root.generate(project, "", &mut files)?;
    Ok(files)
}

/// A node of the module tree, with its optional schema file & its child modules.
#[derive(Default)]
struct ModNode<'a> {
    schema_file: Option<(ModPathRef<'a>, &'a SchemaFile)>,
    children: BTreeMap<String, ModNode<'a>>,
}

impl ModNode<'_> {
    /// Generates the files of the node & its descendants in the `dir`. (empty for the root)
    fn generate(
        &self,
        project: &Project,
        dir: &str,
        files: &mut Vec<GeneratedFile>,
    ) -> Result<(), GenError> {
        let mut code: String = header();
        let mut mod_names: BTreeSet<String> = BTreeSet::default();
        if let Some((mod_path, schema_file)) = self.schema_file {
            let mut uses: String = String::default();
            let mut mods: String = String::default();
            for type_dec in schema_file.type_decs() {
                let mod_name: String = snake_case(&type_dec.type_name());
                if mod_name == "mod" || !mod_names.insert(mod_name.clone()) {
                    return Err(conflict(mod_path.as_ref(), &mod_name));
                }
                uses.push_str(&format!("pub use {}::*;\n", ident(&mod_name)));
                if mod_path
                    .mod_names()
                    .last()
                    .is_some_and(|parent| parent == mod_name)
                {
                    mods.push_str("#[allow(clippy::module_inception)]\n");
                }
                mods.push_str(&format!("mod {};\n", ident(&mod_name)));
                files.extend(gen_type_files(project, mod_path, dir, &mod_name, type_dec)?);
            }
            if !uses.is_empty() {
                code.push_str(&uses);
                code.push('\n');
                code.push_str(&mods);
            }
        }
        if !self.children.is_empty() && !mod_names.is_empty() {
            code.push('\n');
        }
        for (mod_name, child) in &self.children {
            if mod_name == "mod" || !mod_names.insert(mod_name.clone()) {
                return Err(conflict(dir.trim_end_matches('/'), mod_name));
            }
            if !is_snake_case(mod_name) {
                code.push_str("#[allow(non_snake_case)]\n");
            }
            let child_file_name: String = file_name(mod_name);
            if dir.trim_end_matches('/').rsplit('/').next() == Some(child_file_name.as_str()) {
                code.push_str("#[allow(clippy::module_inception)]\n");
            }
            code.push_str(&format!("pub mod {};\n", ident(mod_name)));
            let child_dir: String = format!("{}{}/", dir, child_file_name);
            child.generate(project, &child_dir, files)?;
        }
        let path: String = if dir.is_empty() {
            "lib.rs".to_string()
        } else {
            format!("{}mod.rs", dir)
        };
        files.push(GeneratedFile { path, code });
        Ok(())
    }
}

/// Generates the files of the `type_dec` in the schema file at the `mod_path`, for the module
/// `mod_name` in the `dir`.
fn gen_type_files(
    project: &Project,
    mod_path: ModPathRef<'_>,
    dir: &str,
    mod_name: &str,
    type_dec: &TypeDec,
) -> Result<Vec<GeneratedFile>, GenError> {
    let code: String = match type_dec {
        TypeDec::Struct(structure) => {
            let files: Vec<TypeFile> = gen_struct(project, mod_path, structure)?;
            return Ok(folder_files(dir, mod_name, files));
        }
        TypeDec::Message(message) => {
            let files: Vec<TypeFile> = gen_message(project, mod_path, message)?;
            return Ok(folder_files(dir, mod_name, files));
        }
        TypeDec::Enum(enumeration) => gen_enum(enumeration),
        TypeDec::Variant(variant) => {
            let files: Vec<TypeFile> = gen_variant(project, mod_path, variant)?;
            return Ok(folder_files(dir, mod_name, files));
        }
        TypeDec::Service(service) => gen_service(project, mod_path, service),
    };
    Ok(vec![GeneratedFile {
        path: format!("{}{}.rs", dir, file_name(mod_name)),
        code: format!("{}{}", header(), code),
    }])
}

/// Gets the folder of the type with the `files`, for the module `mod_name` in the `dir`.
fn folder_files(dir: &str, mod_name: &str, files: Vec<TypeFile>) -> Vec<GeneratedFile> {
    let folder: String = format!("{}{}/", dir, file_name(mod_name));
    let mut uses: String = String::default();
    let mut mods: String = String::from("#[allow(clippy::module_inception)]\n");
    let mut result: Vec<GeneratedFile> = Vec::default();
    for file in files {
        let module: String = format!("{}{}", mod_name, file.suffix);
        uses.push_str(&format!("pub use {}::*;\n", ident(&module)));
        mods.push_str(&format!("mod {};\n", ident(&module)));
        result.push(GeneratedFile {
            path: format!("{}{}.rs", folder, file_name(&module)),
            code: format!("{}{}", header(), file.code),
        });
    }
    result.push(GeneratedFile {
        path: format!("{}mod.rs", folder),
        code: format!("{}{}\n{}", header(), uses, mods),
    });
    result
}

/// Gets the header of every generated file, with a blank line after it.
fn header() -> String {
    format!("// {}\n\n", GeneratedFile::MARKER)
}

/// Gets the file name of the module with the `mod_name`. (the identifier without `r#`)
fn file_name(mod_name: &str) -> String {
    ident(mod_name).trim_start_matches("r#").to_string()
}

/// Creates the error for the conflicting `mod_name` in the `mod_path`.
fn conflict(mod_path: &str, mod_name: &str) -> GenError {
    GenError::ModNameConflict {
        mod_path: mod_path.replace('/', "."),
        mod_name: mod_name.to_string(),
    }
}
