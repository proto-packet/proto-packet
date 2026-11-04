use crate::layout::StructLayout;
use proto_packet_tree::{
    PrimitiveType, Project, QualifiedName, SpecialType, Struct, TypeDec, TypeTag, WithTypeTag,
};

/// The layout of a type's values in a struct.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TypeLayout {
    /// A `bool` bit.
    Bit,

    /// A fixed-size value.
    Fixed { size: usize, alignment: usize },

    /// A variable-size value.
    Variable { alignment: usize },
}

impl TypeLayout {
    //! Construction

    /// Gets the layout of the `tag` of the linked `project`.
    ///
    /// Returns `None` if the `tag` has no encoding yet. (ex: a message)
    pub fn new(project: &Project, tag: &TypeTag) -> Option<Self> {
        let fixed = |size: usize| Self::Fixed {
            size,
            alignment: size.min(8),
        };
        match tag {
            TypeTag::Primitive(PrimitiveType::Boolean) => Some(Self::Bit),
            TypeTag::Primitive(primitive) => Some(fixed(primitive_size(*primitive))),
            TypeTag::Special(SpecialType::Uuid) => Some(fixed(16)),
            TypeTag::Special(SpecialType::String) => Some(Self::Variable { alignment: 1 }),
            TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref())? {
                TypeDec::Enum(_) => Some(fixed(2)),
                TypeDec::Struct(structure) if is_fixed(project, structure) => {
                    let layout: StructLayout = StructLayout::new(project, structure)?;
                    Some(Self::Fixed {
                        size: layout.fixed_size,
                        alignment: layout.alignment,
                    })
                }
                TypeDec::Struct(structure) => Some(Self::Variable {
                    alignment: alignment(project, structure)?,
                }),
                _ => None,
            },
        }
    }
}

/// Gets the size of the non-`bool` `primitive`.
const fn primitive_size(primitive: PrimitiveType) -> usize {
    match primitive {
        PrimitiveType::Boolean | PrimitiveType::UnsignedInt8 => 1,
        PrimitiveType::UnsignedInt16 => 2,
        PrimitiveType::UnsignedInt32 => 4,
        PrimitiveType::UnsignedInt64 => 8,
        PrimitiveType::UnsignedInt128 => 16,
    }
}

/// Checks if the `structure` is fixed: all its fields are fixed-size, & it contains no loop.
fn is_fixed(project: &Project, structure: &Struct) -> bool {
    is_fixed_visiting(project, structure, &mut Vec::default())
}

/// Checks if the `structure` is fixed, with the `visiting` structs as variable-size.
///
/// A struct being visited contains itself through a loop of types, which has no fixed size.
fn is_fixed_visiting(
    project: &Project,
    structure: &Struct,
    visiting: &mut Vec<QualifiedName>,
) -> bool {
    structure
        .fields()
        .iter()
        .all(|field| is_fixed_tag(project, field.type_tag(), visiting))
}

/// Checks if the `tag` is fixed-size, with the `visiting` structs as variable-size.
fn is_fixed_tag(project: &Project, tag: &TypeTag, visiting: &mut Vec<QualifiedName>) -> bool {
    match tag {
        TypeTag::Primitive(_) | TypeTag::Special(SpecialType::Uuid) => true,
        TypeTag::Special(SpecialType::String) => false,
        TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref()) {
            Some(TypeDec::Enum(_)) => true,
            Some(TypeDec::Struct(structure)) if !visiting.contains(name) => {
                visiting.push(name.clone());
                let is_fixed: bool = is_fixed_visiting(project, structure, visiting);
                visiting.pop();
                is_fixed
            }
            _ => false,
        },
    }
}

/// Gets the alignment of the `structure`: the largest of it & every struct it contains.
///
/// Returns `None` if the `structure` contains a type with no encoding yet. (ex: a message)
pub(crate) fn alignment(project: &Project, structure: &Struct) -> Option<usize> {
    let mut structs: Vec<&Struct> = vec![structure];
    let mut visited: Vec<&QualifiedName> = Vec::default();
    let mut result: usize = 1;
    while let Some(structure) = structs.pop() {
        for field in structure.fields() {
            let tag: &TypeTag = field.type_tag();
            if !is_fixed_tag(project, tag, &mut Vec::default()) {
                result = result.max(4);
            }
            match tag {
                TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref())? {
                    TypeDec::Enum(_) => result = result.max(2),
                    TypeDec::Struct(child) => {
                        if !visited.contains(&name) {
                            visited.push(name);
                            structs.push(child);
                        }
                    }
                    _ => return None,
                },
                _ => {
                    if let Some(TypeLayout::Fixed { alignment, .. }) = TypeLayout::new(project, tag)
                    {
                        result = result.max(alignment);
                    }
                }
            }
        }
    }
    Some(result)
}
