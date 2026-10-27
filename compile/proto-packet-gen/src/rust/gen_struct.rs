use crate::GenError;
use crate::layout::StructLayout;
use crate::rust::derives::add_derives;
use crate::rust::docs::{PpsLine, add_pps_docs};
use crate::rust::encoding::{ENCODING_METHODS, gen_encoding};
use crate::rust::fields::{Field, check_field_names};
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Signature, Struct as RustStruct, Var, WithAccess, WithAttributes,
    WithComments, WithFunctions, WithResult, WithStructFields, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{
    ModPathRef, Project, Struct, StructField as SchemaField, WithComments as _, WithFieldName,
    WithTypeName, WithTypeTag,
};

/// The name of the constructor.
const CONSTRUCTOR: &str = "new";

/// The most constructor parameters before clippy's `too_many_arguments` lint.
const MAX_ARGUMENTS: usize = 7;

/// Generates the Rust code of the `structure` in the schema file at the `mod_path`.
///
/// No field can be named like a generated method. (ex: the constructor `new`) The struct has the
/// encoding methods if its fields have encodings. (see `FORMAT.md`)
pub fn gen_struct(
    project: &Project,
    mod_path: ModPathRef<'_>,
    structure: &Struct,
) -> Result<String, GenError> {
    let typing: Typing = Typing::new(project);
    let type_name: String = ident(&structure.type_name());
    let fields: Vec<Field> = structure.fields().iter().map(field).collect();
    let methods: Vec<&str> = [CONSTRUCTOR].into_iter().chain(ENCODING_METHODS).collect();
    check_field_names(
        &format!("{}.{}", mod_path, structure.type_name()),
        &fields,
        &methods,
    )?;

    let mut block: Block = Block::default().with_statement(gen_dec(typing, structure, &fields));
    if !fields.is_empty() {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_constructor(typing, &type_name, &fields));
    }
    for field in &fields {
        block.add_statement(EmptyLine::default());
        block.add_statement(field.gen_impl(typing, &type_name));
    }
    if let Some(layout) = StructLayout::new(project, structure) {
        block.add_statement(EmptyLine::default());
        block.add_statement(gen_encoding(project, structure, &layout, &type_name));
    }
    Ok(block.to_code())
}

/// Gets the generated field of the schema `field`.
fn field(field: &SchemaField) -> Field<'_> {
    Field {
        name: field.field_name(),
        tag: field.type_tag(),
        is_optional: field.is_optional(),
        is_boxed: field.is_boxed(),
    }
}

/// Generates the declaration of the `structure`.
fn gen_dec(typing: Typing<'_>, structure: &Struct, fields: &[Field<'_>]) -> RustStruct {
    let mut result: RustStruct = RustStruct::from(ident(&structure.type_name()));
    let lines: Vec<PpsLine> = structure
        .fields()
        .iter()
        .map(|schema_field| PpsLine {
            comments: schema_field.comments(),
            line: format!(
                "{}: {}{}{};",
                schema_field.field_name(),
                if schema_field.is_optional() {
                    "optional "
                } else {
                    ""
                },
                if schema_field.is_boxed() { "box " } else { "" },
                schema_field.type_tag()
            ),
        })
        .collect();
    add_pps_docs(
        &mut result,
        "struct",
        structure.type_name().value(),
        structure.comments(),
        &lines,
    );
    add_derives(
        &mut result,
        typing.is_copy_struct(structure),
        fields.is_empty(),
    );
    if !is_upper_camel_case(&structure.type_name()) {
        result.add_attribute("allow(non_camel_case_types)");
    }
    if fields.iter().any(|field| field.is_non_snake_case()) {
        result.add_attribute("allow(non_snake_case)");
    }
    result.set_access(Access::Public);
    for field in fields {
        result.add_field(field.gen_dec(typing));
    }
    result
}

/// Generates the constructor of the struct with the `fields`.
fn gen_constructor(typing: Typing<'_>, type_name: &str, fields: &[Field<'_>]) -> ImplBlock {
    let mut signature: Signature = Signature::from(CONSTRUCTOR).with_result("Self");
    for field in fields {
        signature.add_param(Var::from((ident(&field.name), field.value_type(typing))));
    }
    let inits: Vec<String> = fields
        .iter()
        .map(|field| {
            let name: String = ident(&field.name);
            let value: String = field.storage_expr(typing, &name);
            if value == name {
                name
            } else {
                format!("{}: {}", name, value)
            }
        })
        .collect();
    let mut constructor: Function = Function::from(signature)
        .with_comment(format!(" Creates a new [{}].", type_name))
        .with_access(Access::Public)
        .with_literal(format!("Self {{ {} }}", inits.join(", ")));
    if fields.len() > MAX_ARGUMENTS {
        constructor.add_attribute("allow(clippy::too_many_arguments)");
    }
    if fields.iter().any(|field| field.is_non_snake_case()) {
        constructor.add_attribute("allow(non_snake_case)");
    }
    ImplBlock::from(type_name)
        .with_comment(" Construction")
        .with_function(constructor)
}

#[cfg(test)]
mod tests {
    use crate::GenError;
    use crate::rust::gen_struct;
    use proto_packet_tree::{
        FieldName, ModPath, PrimitiveType, Project, Struct, StructField, TypeName,
    };

    #[test]
    fn gen_struct_rejects_the_constructor_name() {
        let mod_path: ModPath = ModPath::try_from("a").unwrap();
        let field: StructField = StructField::new(
            FieldName::try_from("new").unwrap(),
            PrimitiveType::UnsignedInt32,
        );
        let structure: Struct = Struct::from(TypeName::try_from("Foo").unwrap())
            .with_field(field)
            .unwrap();
        let result: Result<String, GenError> =
            gen_struct(&Project::default(), mod_path.to_ref(), &structure);
        let expected: GenError = GenError::FieldNameConflict {
            type_name: "a.Foo".to_string(),
            field_name: "new".to_string(),
        };
        assert_eq!(result, Err(expected));
    }
}
