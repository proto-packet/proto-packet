//! The encoding of any form of a type, through the getters of its `View` trait.
//!
//! The `View` trait of a type has provided functions that encode it from its getters, so any type
//! with the getters is written with no native value in between. The generated forms override them,
//! since a native value has its own encoding & an encoded form is already its bytes.

use crate::layout::{FieldLayout, StructLayout, UnitLayout, ValueLayout};
use crate::rust::encoding::IO;
use crate::rust::fields::Field;
use crate::rust::gen_struct_ref::generated_path;
use crate::rust::gen_variant::Case;
use crate::rust::message_encoding::TaggedField;
use crate::rust::naming::ident;
use crate::rust::typing::Typing;
use code_gen::WithStatements;
use code_gen::rust::{
    Function, ImplBlock, Receiver, Signature, Var, WithComments, WithFunctions, WithReceiver,
    WithResult, WithVarParams,
};
use proto_packet_tree::{PrimitiveType, SpecialType, TypeTag};

/// The panic of a view of a variant with a case its trait doesn't have.
const UNKNOWN_CASE: &str = "panic!(\"a view of an unknown case has no encoding\")";

/// The bodies of the functions that encode a type from the getters of its `View` trait.
pub(crate) struct ViewEncoding {
    /// The kind of the type. (ex: `struct`)
    pub(crate) kind: &'static str,

    /// If the type is a level, which is a message, a variant or a struct with a variable tail.
    pub(crate) is_level: bool,

    /// If the encoding writes to the `out` bytes. (ex: false for a struct with no fields)
    pub(crate) uses_out: bool,

    /// The statements getting the length of the encoding.
    pub(crate) encoded_len: Vec<String>,

    /// The statements appending the encoding to the `out` bytes, with the `depth` of levels left.
    pub(crate) encode: Vec<String>,
}

/// A form of a type that has its `View` trait.
#[derive(Copy, Clone)]
pub(crate) enum ViewForm<'a> {
    /// A reference to another form.
    Reference,

    /// The native type with the name.
    Native(&'a str),

    /// The `Ref` type.
    Ref,

    /// The `Buf` or `Mut` type, which has the `Ref` type through the `View` trait with the name.
    Encoded(&'a str),
}

/// A value of a view, as an expression that is the value or a reference to it.
#[derive(Copy, Clone)]
struct Item<'a> {
    expr: &'a str,
    is_ref: bool,
}

impl<'a> Item<'a> {
    //! Construction

    /// Creates the item of the `expr` of a value.
    const fn value_of(expr: &'a str) -> Self {
        Self {
            expr,
            is_ref: false,
        }
    }

    /// The item of a closure, which is a reference.
    const CLOSURE: Item<'static> = Item {
        expr: "item",
        is_ref: true,
    };
}

impl Item<'_> {
    //! Expressions

    /// Gets the expression of the value.
    fn value(self) -> String {
        if self.is_ref {
            format!("*{}", self.expr)
        } else {
            self.expr.to_string()
        }
    }

    /// Gets the expression of a reference to the value.
    fn reference(self) -> String {
        if self.is_ref {
            self.expr.to_string()
        } else {
            format!("&{}", self.expr)
        }
    }
}

/// Checks if a value of the `tag` is a fixed-size scalar, which a list never measures.
fn is_scalar(typing: Typing<'_>, tag: &TypeTag) -> bool {
    match tag {
        TypeTag::Primitive(_) | TypeTag::Special(SpecialType::Uuid) => true,
        TypeTag::Named(_) => typing.is_enum(tag),
        _ => false,
    }
}

/// Generates the statement appending the `item` of the `tag` to the `out` expression.
///
/// The `depth` is the expression of the levels the item can nest.
fn gen_write(typing: Typing<'_>, tag: &TypeTag, item: Item<'_>, out: &str, depth: &str) -> String {
    match tag {
        TypeTag::Primitive(PrimitiveType::Boolean) => {
            format!("{}.push(u8::from({}));", out, item.value())
        }
        TypeTag::Primitive(_) => {
            format!("{}.extend_from_slice(&{}.to_le_bytes());", out, item.expr)
        }
        TypeTag::Special(_) => format!("{}.extend_from_slice({}.as_bytes());", out, item.expr),
        TypeTag::Named(_) if typing.is_enum(tag) => format!(
            "{}.extend_from_slice(&{}.tag_number().to_le_bytes());",
            out, item.expr
        ),
        TypeTag::Named(_) => format!(
            "{}::encode_to_with_depth({}, {}, {});",
            generated_path(tag, "View"),
            item.reference(),
            out,
            depth
        ),
        TypeTag::List(_) if tag.is_bytes() => {
            format!("{}.extend_from_slice({});", out, item.expr)
        }
        TypeTag::List(element) => format!(
            "{io}::write_list_view({list}, {out}, <{native} as {io}::Element>::SIZE, <{native} as {io}::Element>::ALIGNMENT, |item, out| {{ {write} }});",
            io = IO,
            list = item.value(),
            out = out,
            native = typing.rust_type(element),
            write = gen_write(typing, element, Item::CLOSURE, "out", depth)
        ),
        TypeTag::Map { key, value } => format!(
            "{io}::write_map_view::<{key}, {value}, _, _, _>({map}, {out}, |item, out| {{ {write_key} }}, |item, out| {{ {write_value} }});",
            io = IO,
            key = typing.rust_type(key),
            value = typing.rust_type(value),
            map = item.value(),
            out = out,
            write_key = gen_write(typing, key, Item::CLOSURE, "out", depth),
            write_value = gen_write(typing, value, Item::CLOSURE, "out", depth)
        ),
    }
}

/// Generates the expression of the length of the encoding of the variable-size `item` of the `tag`.
fn gen_len(typing: Typing<'_>, tag: &TypeTag, item: Item<'_>) -> String {
    match tag {
        TypeTag::Named(_) => format!(
            "{}::encoded_len({})",
            generated_path(tag, "View"),
            item.reference()
        ),
        TypeTag::List(element) if !tag.is_bytes() => format!(
            "{io}::list_view_len({list}, <{native} as {io}::Element>::SIZE, <{native} as {io}::Element>::ALIGNMENT, {len})",
            io = IO,
            list = item.value(),
            native = typing.rust_type(element),
            len = gen_len_closure(typing, element)
        ),
        TypeTag::Map { key, value } => format!(
            "{io}::map_view_len::<{key_type}, {value_type}, _, _, _, _>({map}, {key_len}, {value_len}, |item, out| {{ {write_key} }})",
            io = IO,
            key_type = typing.rust_type(key),
            value_type = typing.rust_type(value),
            map = item.value(),
            key_len = gen_len_closure(typing, key),
            value_len = gen_len_closure(typing, value),
            write_key = gen_write(typing, key, Item::CLOSURE, "out", "usize::MAX")
        ),
        _ => format!("{}.len()", item.expr),
    }
}

/// Generates the closure getting the length of the encoding of an element of the `tag`.
///
/// # Note
/// - A list never measures a fixed-size scalar, so its closure is never called.
fn gen_len_closure(typing: Typing<'_>, tag: &TypeTag) -> String {
    if is_scalar(typing, tag) {
        "|_| 0".to_string()
    } else if matches!(tag, TypeTag::Named(_)) {
        format!("{}::encoded_len", generated_path(tag, "View"))
    } else {
        format!("|item| {}", gen_len(typing, tag, Item::CLOSURE))
    }
}

/// Generates the statement of the `field`, from the statement for its present value.
///
/// An absent field has no statement.
fn gen_present<F>(field: Field<'_>, statement: F) -> String
where
    F: Fn(Item<'_>) -> String,
{
    let getter: String = format!("self.{}()", ident(&field.name));
    if field.is_optional {
        format!(
            "if let Some(value) = {} {{ {} }}",
            getter,
            statement(Item::value_of("value"))
        )
    } else {
        statement(Item::value_of(&getter))
    }
}

/// Generates the encoding of a struct from the getters of its `View` trait.
pub(crate) fn gen_struct_view_encoding(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    layout: &StructLayout,
) -> ViewEncoding {
    let mut encoded_len: Vec<String> = Vec::default();
    if layout.is_fixed() {
        encoded_len.push(layout.fixed_size.to_string());
    } else {
        encoded_len.push(format!("let mut len: usize = {};", layout.fixed_size));
        for entry in &layout.entries {
            let field: Field<'_> = fields[entry.field];
            encoded_len.push(gen_present(field, |item| {
                let size: String = gen_len(typing, field.tag, item);
                if entry.alignment > 1 {
                    format!(
                        "{{ let size: usize = {}; if size > 0 {{ len = {}::align_up(len, {}) + size; }} }}",
                        size, IO, entry.alignment
                    )
                } else {
                    format!("len += {};", size)
                }
            }));
        }
        encoded_len.push("len".to_string());
    }

    let mut encode: Vec<String> = layout
        .units
        .iter()
        .flat_map(|unit| gen_unit(typing, fields, layout, *unit))
        .collect();
    let uses_start: bool =
        layout.ends() > 0 || layout.entries.iter().any(|entry| entry.alignment > 1);
    if uses_start {
        encode.insert(0, "let start: usize = out.len();".to_string());
    }
    for (index, entry) in layout.entries.iter().enumerate() {
        let field: Field<'_> = fields[entry.field];
        encode.push(gen_present(field, |item| {
            let write: String = gen_write(typing, field.tag, item, "out", "depth - 1");
            if entry.alignment > 1 {
                format!(
                    "{{ let before: usize = out.len(); {}::write_padding(out, start, {}); let padded: usize = out.len(); {} {}::trim_padding(out, before, padded); }}",
                    IO, entry.alignment, write, IO
                )
            } else {
                write
            }
        }));
        if index < layout.ends() {
            let offset: usize = layout.table_offset + index * 4;
            let item: String = if offset == 0 {
                "start".to_string()
            } else {
                format!("start + {}", offset)
            };
            encode.push(format!("{}::write_end(out, {}, start);", IO, item));
        }
    }
    ViewEncoding {
        kind: "struct",
        is_level: !layout.is_fixed(),
        uses_out: !encode.is_empty(),
        encoded_len,
        encode,
    }
}

/// Generates the statements appending the `unit` of the fixed block.
fn gen_unit(
    typing: Typing<'_>,
    fields: &[Field<'_>],
    layout: &StructLayout,
    unit: UnitLayout,
) -> Vec<String> {
    match unit {
        UnitLayout::Field { index } => {
            vec![gen_fixed_field(typing, fields[index], layout.fields[index])]
        }
        UnitLayout::EndTable => vec![format!("{}::write_zeros(out, {});", IO, layout.ends() * 4)],
        UnitLayout::Bits { offset, size, .. } => (offset..offset + size)
            .map(|offset| gen_bits(fields, layout, offset))
            .collect(),
        UnitLayout::Padding { size, .. } => {
            vec![format!("{}::write_zeros(out, {});", IO, size)]
        }
    }
}

/// Generates the statement appending the fixed-size `field`.
///
/// An absent field keeps its place as zeros.
fn gen_fixed_field(typing: Typing<'_>, field: Field<'_>, layout: FieldLayout) -> String {
    let getter: String = format!("self.{}()", ident(&field.name));
    let value: String = if field.is_optional {
        format!("{}.unwrap_or_default()", getter)
    } else {
        getter.clone()
    };
    match field.tag {
        TypeTag::Special(SpecialType::Uuid) => {
            format!("out.extend_from_slice({}.as_bytes());", value)
        }
        TypeTag::Primitive(_) => format!("out.extend_from_slice(&{}.to_le_bytes());", value),
        _ if typing.is_enum(field.tag) => {
            let tag_number: String = if field.is_optional {
                format!("{}.map_or(0, |value| value.tag_number())", getter)
            } else {
                format!("{}.tag_number()", getter)
            };
            format!("out.extend_from_slice(&{}.to_le_bytes());", tag_number)
        }
        _ if field.is_optional => {
            let ValueLayout::Fixed { size, .. } = layout.value else {
                unreachable!("a fixed-size field is in the fixed block")
            };
            format!(
                "match {} {{ Some(value) => {}::encode_to_with_depth(&value, out, 0), None => {}::write_zeros(out, {}), }}",
                getter,
                generated_path(field.tag, "View"),
                IO,
                size
            )
        }
        _ => format!(
            "{}::encode_to_with_depth(&{}, out, 0);",
            generated_path(field.tag, "View"),
            getter
        ),
    }
}

/// Generates the statement appending the byte of the bit group at the `offset`.
fn gen_bits(fields: &[Field<'_>], layout: &StructLayout, offset: usize) -> String {
    let bit = |value: &str, bit: u8| -> String {
        if bit == 0 {
            format!("u8::from({})", value)
        } else {
            format!("(u8::from({}) << {})", value, bit)
        }
    };
    let mut bits: Vec<String> = Vec::default();
    for (field, field_layout) in fields.iter().zip(&layout.fields) {
        let getter: String = format!("self.{}()", ident(&field.name));
        if let Some(presence) = field_layout.presence
            && presence.offset == offset
        {
            bits.push(bit(&format!("{}.is_some()", getter), presence.bit));
        }
        if let ValueLayout::Bit(value) = field_layout.value
            && value.offset == offset
        {
            let getter: String = if field.is_optional {
                format!("{}.unwrap_or_default()", getter)
            } else {
                getter
            };
            bits.push(bit(&getter, value.bit));
        }
    }
    format!("out.push({});", bits.join(" | "))
}

/// Generates the encoding of a message from the getters of its `View` trait.
///
/// # Note
/// - The `fields` are in tag number order, which is the order they are written in.
/// - A view has no unknown fields, so none are written.
pub(crate) fn gen_message_view_encoding(
    typing: Typing<'_>,
    fields: &[TaggedField<'_>],
) -> ViewEncoding {
    let mut encoded_len: Vec<String> = Vec::default();
    let mut encode: Vec<String> = Vec::default();
    if fields.is_empty() {
        encoded_len.push(format!("{}::message_index_len(0)", IO));
        encode.push(format!("{}::MessageWriter::new(out, 0, &[]).finish();", IO));
    } else {
        encoded_len.push("let mut count: usize = 0;".to_string());
        encoded_len.push("let mut values: usize = 0;".to_string());
        let mut known: Vec<String> = Vec::default();
        let mut writes: Vec<String> = Vec::default();
        for tagged in fields {
            let getter: String = format!("self.{}()", ident(&tagged.field.name));
            encoded_len.push(match tagged.size {
                Some(size) => format!(
                    "if {}.is_some() {{ count += 1; values += {}; }}",
                    getter, size
                ),
                None => format!(
                    "if let Some(value) = {} {{ count += 1; values += {}; }}",
                    getter,
                    gen_len(typing, tagged.field.tag, Item::value_of("value"))
                ),
            });
            known.push(format!("usize::from({}.is_some())", getter));
            writes.push(format!(
                "if let Some(value) = {} {{ {} writer.end_field(); }}",
                getter,
                gen_write(
                    typing,
                    tagged.field.tag,
                    Item::value_of("value"),
                    &format!("writer.field({})", tagged.tag),
                    "depth - 1"
                )
            ));
        }
        encoded_len.push(format!("{}::message_index_len(count) + values", IO));
        encode.push(format!("let known: usize = {};", known.join(" + ")));
        encode.push(format!(
            "let mut writer: {}::MessageWriter = {}::MessageWriter::new(out, known, &[]);",
            IO, IO
        ));
        encode.extend(writes);
        encode.push("writer.finish();".to_string());
    }
    ViewEncoding {
        kind: "message",
        is_level: true,
        uses_out: true,
        encoded_len,
        encode,
    }
}

/// Generates the encoding of a variant from the getters of its `View` trait.
///
/// # Note
/// - A view has the value of each case of its trait, so it can't write any other case.
pub(crate) fn gen_variant_view_encoding(typing: Typing<'_>, cases: &[Case<'_>]) -> ViewEncoding {
    let mut encoded_len: Vec<String> = Vec::default();
    let mut encode: Vec<String> = Vec::default();
    for case in cases {
        let getter: String = format!("self.{}()", case.getter_name());
        encoded_len.push(match case.size {
            Some(size) => format!("if {}.is_some() {{ return {}; }}", getter, 2 + size),
            None => format!(
                "if let Some(value) = {} {{ return 2 + {}; }}",
                getter,
                gen_len(typing, case.tag, Item::value_of("value"))
            ),
        });
        encode.push(format!(
            "if let Some(value) = {} {{ out.extend_from_slice(&{}_u16.to_le_bytes()); {} return; }}",
            getter,
            case.number,
            gen_write(typing, case.tag, Item::value_of("value"), "out", "depth - 1")
        ));
    }
    encoded_len.push(UNKNOWN_CASE.to_string());
    encode.push(format!("{};", UNKNOWN_CASE));
    ViewEncoding {
        kind: "variant",
        is_level: true,
        uses_out: !cases.is_empty(),
        encoded_len,
        encode,
    }
}

/// Gets the signature of the function getting the length of the encoding.
fn encoded_len_signature() -> Signature {
    Signature::from("encoded_len")
        .with_receiver(Receiver::Borrowed)
        .with_result("usize")
}

/// Gets the signature of the function appending the encoding, with the `out` & `depth` names.
fn encode_to_with_depth_signature(out: &str, depth: &str) -> Signature {
    Signature::from("encode_to_with_depth")
        .with_receiver(Receiver::Borrowed)
        .with_param(Var::from((out, "&mut Vec<u8>")))
        .with_param(Var::from((depth, "usize")))
}

/// Generates the provided functions of a `View` trait that encode its type from its getters.
pub(crate) fn gen_view_functions(encoding: &ViewEncoding) -> Vec<Function> {
    let kind: &str = encoding.kind;
    let mut encoded_len: Function = Function::from(encoded_len_signature())
        .with_comment(" Gets the length of the encoding, in bytes.");
    for statement in &encoding.encoded_len {
        encoded_len.add_literal(statement);
    }

    let encode: Function = Function::from(
        Signature::from("encode")
            .with_receiver(Receiver::Borrowed)
            .with_result("Vec<u8>"),
    )
    .with_comment(format!(" Encodes the {}.", kind))
    .with_comment("")
    .with_comment(" # Note")
    .with_comment(" - Any type with the getters is written, with no native value in between.")
    .with_semi("let mut out: Vec<u8> = Vec::with_capacity(self.encoded_len())")
    .with_semi("self.encode_to(&mut out)")
    .with_literal("out");

    let encode_to: Function = Function::from(
        Signature::from("encode_to")
            .with_receiver(Receiver::Borrowed)
            .with_param(Var::from(("out", "&mut Vec<u8>"))),
    )
    .with_comment(format!(
        " Appends the encoding of the {} to the `out` bytes.",
        kind
    ))
    .with_semi(format!("self.encode_to_with_depth(out, {}::MAX_DEPTH)", IO));

    let out: &str = if encoding.uses_out { "out" } else { "_out" };
    let depth: &str = if encoding.is_level { "depth" } else { "_depth" };
    let mut encode_to_with_depth: Function = Function::from(encode_to_with_depth_signature(
        out, depth,
    ))
    .with_comment(format!(
        " Appends the encoding of the {} to the `out` bytes, which can nest the `depth` of levels.",
        kind
    ))
    .with_comment("")
    .with_comment(" # Panics")
    .with_comment(format!(" Panics if the {} can't be written.", kind));
    if encoding.is_level {
        encode_to_with_depth.add_semi(format!("{}::assert_depth(depth)", IO));
    }
    for statement in &encoding.encode {
        encode_to_with_depth.add_literal(statement);
    }
    vec![encoded_len, encode, encode_to, encode_to_with_depth]
}

/// An impl block that takes several functions at once.
pub(crate) trait WithViewFunctions: Sized {
    /// Adds the `functions`.
    fn with_functions(self, functions: Vec<Function>) -> Self;
}

impl WithViewFunctions for ImplBlock {
    fn with_functions(self, functions: Vec<Function>) -> Self {
        functions
            .into_iter()
            .fold(self, |block, function| block.with_function(function))
    }
}

/// Generates the functions of the `form` that override the provided functions of its `View` trait.
pub(crate) fn gen_view_overrides(form: ViewForm<'_>, is_level: bool) -> Vec<Function> {
    let depth: &str = match form {
        ViewForm::Native(_) | ViewForm::Ref if !is_level => "_depth",
        _ => "depth",
    };
    let (encoded_len, encode): (String, String) = match form {
        ViewForm::Reference => (
            "(**self).encoded_len()".to_string(),
            "(**self).encode_to_with_depth(out, depth);".to_string(),
        ),
        ViewForm::Native(type_name) => (
            format!("super::{}::encoded_len(self)", type_name),
            if is_level {
                format!(
                    "super::{}::encode_to_with_depth(self, out, depth);",
                    type_name
                )
            } else {
                format!("super::{}::encode_to(self, out);", type_name)
            },
        ),
        ViewForm::Ref => (
            "self.as_bytes().len()".to_string(),
            if is_level {
                format!(
                    "if depth < {io}::MAX_DEPTH && Self::new_with_depth(self.as_bytes(), depth).is_err() {{ {io}::assert_depth(0); }} out.extend_from_slice(self.as_bytes());",
                    io = IO
                )
            } else {
                "out.extend_from_slice(self.as_bytes());".to_string()
            },
        ),
        ViewForm::Encoded(view_name) => (
            "self.as_bytes().len()".to_string(),
            format!(
                "{}::encode_to_with_depth(&self.as_ref(), out, depth);",
                view_name
            ),
        ),
    };
    vec![
        Function::from(encoded_len_signature()).with_literal(encoded_len),
        Function::from(encode_to_with_depth_signature("out", depth)).with_literal(encode),
    ]
}
