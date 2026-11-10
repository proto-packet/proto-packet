use crate::GenError;
use crate::layout::{LayoutCache, StructLayout};
use crate::rust::derives::{add_derives, add_serde_derives};
use crate::rust::docs::{PpsLine, add_pps_docs};
use crate::rust::encoding::{IO, gen_encoding};
use crate::rust::fields::{Field, check_field_names};
use crate::rust::gen_buf::{buf_name, gen_buf};
use crate::rust::gen_element::gen_element;
use crate::rust::gen_eq::gen_native_eq;
use crate::rust::gen_struct_mut::{
    check_mut_names, gen_buf_accessors, gen_struct_mut, is_struct, mut_name,
};
use crate::rust::gen_struct_ref::{can_be_invalid, gen_struct_ref, ref_name};
use crate::rust::gen_view::{gen_view, view_name};
use crate::rust::gen_view_encoding::gen_struct_view_encoding;
use crate::rust::gen_view_mut::{gen_view_mut, view_mut_name};
use crate::rust::naming::{ident, is_upper_camel_case};
use crate::rust::prelude::{Prelude, PreludeName};
use crate::rust::typing::Typing;
use code_gen::rust::{
    Access, Function, ImplBlock, Receiver, Signature, Struct as RustStruct, Var, WithAccess,
    WithAttributes, WithComments, WithFunctions, WithReceiver, WithResult, WithStructFields,
    WithUnsafeFlag, WithVarParams,
};
use code_gen::{Block, EmptyLine, Statement, WithStatements};
use proto_packet_tree::{
    ModPathRef, Project, Struct, StructField as SchemaField, TypeTag, WithComments as _,
    WithFieldName, WithTypeName, WithTypeTag,
};

/// The name of the constructor.
const CONSTRUCTOR: &str = "new";

/// The names of the generated methods of a struct & its `Ref` type.
const METHODS: [&str; 11] = [
    CONSTRUCTOR,
    "encoded_len",
    "encode",
    "encode_to",
    "encode_to_with_depth",
    "decode",
    "decode_unchecked",
    "new_unchecked",
    "new_with_depth",
    "as_bytes",
    "to_buf",
];

/// A generated file of a struct or message.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeFile {
    /// The suffix of the file's module name. (ex: `_ref`, empty for the type's own file)
    pub suffix: &'static str,

    /// The source code.
    pub code: String,
}

/// The most constructor parameters before clippy's `too_many_arguments` lint.
const MAX_ARGUMENTS: usize = 7;

/// Generates the Rust files of the `structure` in the schema file at the `mod_path`.
///
/// No field can be named like a generated method. (ex: the constructor `new`) Each of the struct's
/// generated types has a file.
pub fn gen_struct(
    project: &Project,
    mod_path: ModPathRef<'_>,
    structure: &Struct,
) -> Result<Vec<TypeFile>, GenError> {
    let type_name: String = ident(&structure.type_name());
    let typing: Typing = Typing::new(project, Prelude::new(&type_name));
    let fields: Vec<Field> = structure.fields().iter().map(field).collect();
    check_list_elements(
        project,
        &format!("{}.{}", mod_path, structure.type_name()),
        fields.iter().map(|field| (field.name.value(), field.tag)),
    )?;
    let layout: StructLayout = StructLayout::new(project, structure);
    if layout.fixed_size > u32::MAX as usize {
        return Err(GenError::StructTooLarge {
            type_name: format!("{}.{}", mod_path, structure.type_name()),
        });
    }
    let accessors: Vec<String> = fields
        .iter()
        .filter(|field| is_struct(typing, **field))
        .map(|field| field.accessor_name())
        .collect();
    let methods: Vec<&str> = METHODS
        .into_iter()
        .chain(accessors.iter().map(String::as_str))
        .collect();
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
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_native_eq(
        Prelude::new(&type_name),
        &type_name,
        &ref_name(&structure.type_name()),
        &buf_name(&structure.type_name()),
    ));
    for field in &fields {
        let mut field_impl: ImplBlock = field.gen_impl(typing, &type_name);
        if is_struct(typing, *field) {
            field_impl.add_function(gen_accessor(typing, *field));
        }
        block.add_statement(EmptyLine::default());
        block.add_statement(field_impl);
    }
    let schema_name: String = structure.type_name().to_string();
    let schema_name: &str = &schema_name;
    check_type_name(project, mod_path, &ref_name(schema_name))?;
    check_type_name(project, mod_path, &buf_name(schema_name))?;
    check_type_name(project, mod_path, &mut_name(schema_name))?;
    check_type_name(project, mod_path, &view_name(schema_name))?;
    check_type_name(project, mod_path, &view_mut_name(schema_name))?;
    check_mut_names(typing, &format!("{}.{}", mod_path, schema_name), &fields)?;
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_encoding(typing, &type_name, &fields, &layout));
    block.add_statement(EmptyLine::default());
    block.add_statement(gen_decoding(
        typing,
        &type_name,
        &ref_name(schema_name),
        "struct",
    ));
    block.add_statement(EmptyLine::default());
    block.add_literal(gen_element(
        Prelude::new(&type_name),
        &type_name,
        &ref_name(schema_name),
        layout.is_fixed().then_some(layout.fixed_size),
        layout.alignment,
        can_be_invalid(typing, &fields, &layout),
        fields
            .iter()
            .any(|field| typing.is_ordered_by_bytes(field.tag)),
    ));
    let view_mut_fields: Vec<Field> = view_mut_fields(&fields);
    Ok(vec![
        TypeFile {
            suffix: "",
            code: block.to_code(),
        },
        TypeFile {
            suffix: "_buf",
            code: gen_buf(
                schema_name,
                &type_name,
                gen_buf_accessors(typing, &fields, &layout, &buf_name(schema_name)),
            ),
        },
        TypeFile {
            suffix: "_mut",
            code: gen_struct_mut(typing, structure, &fields, &layout, &type_name),
        },
        TypeFile {
            suffix: "_ref",
            code: gen_struct_ref(typing, structure, &fields, &layout, &type_name),
        },
        TypeFile {
            suffix: "_view",
            code: gen_view(
                typing,
                schema_name,
                &fields,
                &type_name,
                &gen_struct_view_encoding(typing, &fields, &layout),
            ),
        },
        TypeFile {
            suffix: "_view_mut",
            code: gen_view_mut(typing, schema_name, &view_mut_fields, &type_name, false),
        },
    ])
}

/// Checks no member of the type with the `type_name` is a list with elements of no size.
///
/// The `members` are the names & the types of the fields or the cases.
pub(crate) fn check_list_elements<'a, I>(
    project: &Project,
    type_name: &str,
    members: I,
) -> Result<(), GenError>
where
    I: Iterator<Item = (&'a str, &'a TypeTag)>,
{
    let mut cache: LayoutCache = LayoutCache::new(project);
    for (member_name, tag) in members {
        if cache.has_empty_element(tag) {
            return Err(GenError::EmptyListElement {
                type_name: type_name.to_string(),
                member_name: member_name.to_string(),
            });
        }
    }
    Ok(())
}

/// Gets the `fields` that the `ViewMut` trait has.
///
/// # Note
/// - A list or a map can't change its length in place, so the trait leaves it out.
pub(crate) fn view_mut_fields<'a>(fields: &[Field<'a>]) -> Vec<Field<'a>> {
    fields
        .iter()
        .copied()
        .filter(|field| !field.tag.is_collection())
        .collect()
}

/// Checks no type of the schema file at the `mod_path` has the `name` of a generated type.
pub(crate) fn check_type_name(
    project: &Project,
    mod_path: ModPathRef<'_>,
    name: &str,
) -> Result<(), GenError> {
    let is_conflict: bool = project
        .schema_file_with_mod_path(mod_path.as_ref())
        .is_some_and(|schema_file| {
            schema_file
                .type_decs()
                .iter()
                .any(|type_dec| type_dec.type_name().value() == name)
        });
    if is_conflict {
        return Err(GenError::TypeNameConflict {
            mod_path: mod_path.to_string(),
            type_name: name.to_string(),
        });
    }
    Ok(())
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
        true,
        fields.is_empty(),
    );
    add_serde_derives(&mut result);
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
///
/// It is `const`, unless a field is boxed, since `Box::new` is not.
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
        .with_const(!fields.iter().any(|field| field.is_boxed))
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

/// Generates the mutable accessor of the nested struct `field`. (boxes are hidden)
pub(crate) fn gen_accessor(typing: Typing<'_>, field: Field<'_>) -> Function {
    let name: String = ident(&field.name);
    let reference: String = format!("&mut {}", typing.rust_type(field.tag));
    let (result, expr): (String, String) = if !field.is_optional {
        (reference, format!("&mut self.{}", name))
    } else {
        let method: &str = if field.is_boxed {
            "as_deref_mut"
        } else {
            "as_mut"
        };
        (
            format!("{}<{}>", typing.prelude(PreludeName::Option), reference),
            format!("self.{}.{}()", name, method),
        )
    };
    let signature: Signature = Signature::from(field.accessor_name())
        .with_receiver(Receiver::BorrowedMut)
        .with_result(result);
    let mut function: Function = Function::from(signature)
        .with_comment(format!(" Gets the mutable field: `{}`.", field.name))
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(expr);
    if field.is_non_snake_case() {
        function.add_attribute("allow(non_snake_case)");
    }
    function
}

/// Generates the impl block decoding the `kind` of type with the `type_name` through its `ref_name`
/// type. (ex: a `struct`)
pub(crate) fn gen_decoding(
    typing: Typing<'_>,
    type_name: &str,
    ref_name: &str,
    kind: &str,
) -> ImplBlock {
    let signature: Signature = Signature::from("decode")
        .with_param(Var::from(("bytes", "&[u8]")))
        .with_result(format!(
            "{}<Self, {}::DecodeError>",
            typing.prelude(PreludeName::Result),
            IO
        ));
    let decode: Function = Function::from(signature)
        .with_comment(format!(" Decodes the {} from the `bytes`.", kind))
        .with_comment("")
        .with_comment(" # Errors")
        .with_comment(" Returns an error if the `bytes` are not a valid encoding.")
        .with_access(Access::Public)
        .with_literal(format!("Ok(super::{}::new(bytes)?.decode())", ref_name));

    let signature: Signature = Signature::from("decode_unchecked")
        .with_unsafe(true)
        .with_param(Var::from(("bytes", "&[u8]")))
        .with_result("Self");
    let decode_unchecked: Function = Function::from(signature)
        .with_comment(format!(
            " Decodes the {} from the `bytes`, without validating them.",
            kind
        ))
        .with_comment("")
        .with_comment(" # Safety")
        .with_comment(" The `bytes` must be a valid encoding.")
        .with_attribute("must_use")
        .with_access(Access::Public)
        .with_literal(format!(
            "unsafe {{ super::{}::new_unchecked(bytes) }}.decode()",
            ref_name
        ));

    ImplBlock::from(type_name)
        .with_comment(" Decoding")
        .with_function(decode)
        .with_function(decode_unchecked)
}
