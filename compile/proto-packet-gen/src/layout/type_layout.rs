use crate::layout::StructLayout;
use proto_packet_tree::{
    PrimitiveType, Project, QualifiedName, SpecialType, Struct, TypeDec, TypeTag, WithTypeTag,
};
use std::collections::BTreeMap;

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

/// Gets the declaration of the type with the `name` in the linked `project`.
fn linked_type<'a>(project: &'a Project, name: &QualifiedName) -> &'a TypeDec {
    project
        .type_dec_with_name(name.to_ref())
        .expect("a linked type is declared")
}

/// Gets the size of the `primitive`, in bytes. (a `bool` in a struct is a bit instead)
pub(crate) const fn primitive_size(primitive: PrimitiveType) -> usize {
    match primitive {
        PrimitiveType::Boolean | PrimitiveType::UnsignedInt8 | PrimitiveType::SignedInt8 => 1,
        PrimitiveType::UnsignedInt16 | PrimitiveType::SignedInt16 => 2,
        PrimitiveType::UnsignedInt32 | PrimitiveType::SignedInt32 => 4,
        PrimitiveType::UnsignedInt64 | PrimitiveType::SignedInt64 => 8,
        PrimitiveType::UnsignedInt128 | PrimitiveType::SignedInt128 => 16,
    }
}

/// The largest fixed-size value tracked exactly, so sizes can't overflow. It's far above the largest
/// valid fixed block.
const MAX_TRACKED_SIZE: usize = 1 << 40;

/// The layouts found so far, since a struct can be reached through many paths.
#[derive(Clone, Debug)]
pub(crate) struct LayoutCache<'a> {
    project: &'a Project,
    is_fixed: BTreeMap<QualifiedName, bool>,
    fixed_layouts: BTreeMap<QualifiedName, TypeLayout>,
}

impl<'a> LayoutCache<'a> {
    //! Construction

    /// Creates a new cache of the layouts of the linked `project`.
    pub(crate) const fn new(project: &'a Project) -> Self {
        Self {
            project,
            is_fixed: BTreeMap::new(),
            fixed_layouts: BTreeMap::new(),
        }
    }
}

impl LayoutCache<'_> {
    //! Layouts

    /// Gets the layout of the `tag`.
    pub(crate) fn type_layout(&mut self, tag: &TypeTag) -> TypeLayout {
        let fixed = |size: usize| TypeLayout::Fixed {
            size,
            alignment: size.min(8),
        };
        let project: &Project = self.project;
        match tag {
            TypeTag::Primitive(PrimitiveType::Boolean) => TypeLayout::Bit,
            TypeTag::Primitive(primitive) => fixed(primitive_size(*primitive)),
            TypeTag::Special(SpecialType::Uuid) => fixed(16),
            TypeTag::Special(SpecialType::String) => TypeLayout::Variable { alignment: 1 },
            TypeTag::Named(name) => match linked_type(project, name) {
                TypeDec::Enum(_) => fixed(2),
                TypeDec::Message(_) | TypeDec::Variant(_) => TypeLayout::Variable { alignment: 1 },
                TypeDec::Struct(structure) if self.is_fixed(tag, &mut Vec::default()) => {
                    if let Some(layout) = self.fixed_layouts.get(name) {
                        return *layout;
                    }
                    let layout: StructLayout = StructLayout::new_cached(self, structure);
                    let layout: TypeLayout = TypeLayout::Fixed {
                        size: layout.fixed_size.min(MAX_TRACKED_SIZE),
                        alignment: layout.alignment,
                    };
                    self.fixed_layouts.insert(name.clone(), layout);
                    layout
                }
                TypeDec::Struct(structure) => TypeLayout::Variable {
                    alignment: self.alignment(structure),
                },
                TypeDec::Service(_) => unreachable!("the linker rejects services as types"),
            },
        }
    }

    /// Checks if the `tag` is an enum.
    pub(crate) fn is_enum(&self, tag: &TypeTag) -> bool {
        match tag {
            TypeTag::Named(name) => matches!(linked_type(self.project, name), TypeDec::Enum(_)),
            _ => false,
        }
    }

    /// Checks if the `tag` is fixed-size, with the `visiting` structs as variable-size.
    ///
    /// A struct being visited contains itself through a loop of types, which has no fixed size. So
    /// does any struct on the way to it, so the answer holds however a struct is reached.
    fn is_fixed(&mut self, tag: &TypeTag, visiting: &mut Vec<QualifiedName>) -> bool {
        let project: &Project = self.project;
        match tag {
            TypeTag::Primitive(_) | TypeTag::Special(SpecialType::Uuid) => true,
            TypeTag::Special(SpecialType::String) => false,
            TypeTag::Named(name) => match linked_type(project, name) {
                TypeDec::Enum(_) => true,
                TypeDec::Struct(structure) => {
                    if let Some(is_fixed) = self.is_fixed.get(name) {
                        return *is_fixed;
                    }
                    if visiting.contains(name) {
                        return false;
                    }
                    visiting.push(name.clone());
                    let is_fixed: bool = structure
                        .fields()
                        .iter()
                        .all(|field| self.is_fixed(field.type_tag(), visiting));
                    visiting.pop();
                    self.is_fixed.insert(name.clone(), is_fixed);
                    is_fixed
                }
                _ => false,
            },
        }
    }

    /// Gets the alignment of the `structure`: the largest of it & every struct it contains.
    ///
    /// A message or variant has alignment 1, whatever it contains, so its fields & cases can
    /// change. A struct with two or more variable-size fields has an end table, of alignment 4.
    pub(crate) fn alignment(&mut self, structure: &Struct) -> usize {
        let project: &Project = self.project;
        let mut structs: Vec<&Struct> = vec![structure];
        let mut visited: Vec<&QualifiedName> = Vec::default();
        let mut result: usize = 1;
        while let Some(structure) = structs.pop() {
            let mut variable: usize = 0;
            for field in structure.fields() {
                let tag: &TypeTag = field.type_tag();
                if !self.is_fixed(tag, &mut Vec::default()) {
                    variable += 1;
                }
                match tag {
                    TypeTag::Named(name) => match linked_type(project, name) {
                        TypeDec::Enum(_) => result = result.max(2),
                        TypeDec::Struct(child) if !visited.contains(&name) => {
                            visited.push(name);
                            structs.push(child);
                        }
                        _ => {}
                    },
                    _ => {
                        if let TypeLayout::Fixed { alignment, .. } = self.type_layout(tag) {
                            result = result.max(alignment);
                        }
                    }
                }
            }
            if variable >= 2 {
                result = result.max(4);
            }
        }
        result
    }
}
