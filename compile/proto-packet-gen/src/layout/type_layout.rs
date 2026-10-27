use crate::layout::StructLayout;
use proto_packet_tree::{
    PrimitiveType, Project, QualifiedName, SpecialType, TimeType, TypeDec, TypeTag,
};

/// The layout of a type's values, in a struct's head or a list. (see `FORMAT.md`)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TypeLayout {
    /// A fixed-size value, of the `size`, in the `alignment` group of a head.
    Fixed { size: usize, alignment: usize },

    /// A `bool` bit.
    Bit,

    /// A variable-size value. (a length in a head, with the value in the data)
    Variable,
}

impl TypeLayout {
    //! Construction

    /// Gets the layout of the values of the `tag` of the linked `project`.
    ///
    /// Returns `None` if the `tag` has no encoding yet. (ex: a message)
    pub fn new(project: &Project, tag: &TypeTag) -> Option<Self> {
        Self::new_visiting(project, tag, &mut Vec::default())
    }

    /// Gets the layout of the values of the `tag`, with the `visiting` structs as variable-size.
    ///
    /// A struct being visited contains itself through a loop of types, which has no fixed size.
    pub(crate) fn new_visiting(
        project: &Project,
        tag: &TypeTag,
        visiting: &mut Vec<QualifiedName>,
    ) -> Option<Self> {
        let fixed = |size: usize| Self::Fixed {
            size,
            alignment: size.min(8),
        };
        match tag {
            TypeTag::Primitive(PrimitiveType::Boolean) => Some(Self::Bit),
            TypeTag::Primitive(primitive) => Some(fixed(primitive_size(*primitive))),
            TypeTag::Special(SpecialType::Uuid) => Some(fixed(16)),
            TypeTag::Special(SpecialType::String) => Some(Self::Variable),
            TypeTag::Time(TimeType::Timestamp) => Some(fixed(8)),
            TypeTag::Time(TimeType::Date) => Some(fixed(4)),
            TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref())? {
                TypeDec::Enum(_) => Some(fixed(2)),
                TypeDec::Struct(_) if visiting.contains(name) => Some(Self::Variable),
                TypeDec::Struct(structure) => {
                    visiting.push(name.clone());
                    let layout: Option<StructLayout> =
                        StructLayout::new_visiting(project, structure, visiting);
                    visiting.pop();
                    let layout: StructLayout = layout?;
                    if layout.is_fixed() {
                        Some(Self::Fixed {
                            size: layout.head_size(),
                            alignment: layout.alignment(),
                        })
                    } else {
                        Some(Self::Variable)
                    }
                }
                _ => None,
            },
            TypeTag::Slice { base } => {
                Self::new_visiting(project, base, visiting)?;
                Some(Self::Variable)
            }
            TypeTag::Map { .. } => None,
        }
    }
}

/// Gets the size of the `primitive`, in bytes. (a `bool` is a bit, not a byte)
fn primitive_size(primitive: PrimitiveType) -> usize {
    match primitive {
        PrimitiveType::Boolean | PrimitiveType::UnsignedInt8 | PrimitiveType::SignedInt8 => 1,
        PrimitiveType::UnsignedInt16 | PrimitiveType::SignedInt16 => 2,
        PrimitiveType::UnsignedInt32 | PrimitiveType::SignedInt32 | PrimitiveType::Float32 => 4,
        PrimitiveType::UnsignedInt64 | PrimitiveType::SignedInt64 | PrimitiveType::Float64 => 8,
        PrimitiveType::UnsignedInt128 | PrimitiveType::SignedInt128 => 16,
    }
}
