use crate::layout::FieldLayout;
use proto_packet_tree::{
    PrimitiveType, Project, QualifiedName, SpecialType, Struct, TimeType, TypeDec, TypeTag,
    WithTypeTag,
};

/// The alignment groups of a head, in order.
const ALIGNMENT_GROUPS: [usize; 4] = [8, 4, 2, 1];

/// The alignment group of the lengths.
const LENGTH_ALIGNMENT: usize = 4;

/// The size of a length.
const LENGTH_SIZE: usize = 4;

/// The layout of a struct's encoding: its head, followed by its variable-size data.
///
/// The head has an item for each field, placed in alignment groups of 8, 4, 2 & 1 bytes, followed
/// by the bit group. Within an alignment group, the fixed-size values are in declaration order,
/// followed by the lengths.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructLayout {
    fields: Vec<FieldLayout>,
    head_size: usize,
    alignment: usize,
    is_fixed: bool,
}

/// The kind of a field's item in the head.
#[derive(Copy, Clone, Debug)]
enum Item {
    /// A fixed-size value, of the `size`, in the `alignment` group.
    Fixed { size: usize, alignment: usize },

    /// The length of a variable-size value.
    Length,

    /// A `bool` bit.
    Bit,
}

impl StructLayout {
    //! Construction

    /// Computes the layout of the `structure` of the linked `project`.
    ///
    /// Returns `None` if a field has no encoding yet. (ex: a slice, or a struct containing itself)
    pub fn new(project: &Project, structure: &Struct) -> Option<Self> {
        Self::new_visiting(project, structure, &mut Vec::default())
    }

    /// Computes the layout of the `structure`, without revisiting the `visiting` structs.
    fn new_visiting(
        project: &Project,
        structure: &Struct,
        visiting: &mut Vec<QualifiedName>,
    ) -> Option<Self> {
        let items: Vec<Item> = structure
            .fields()
            .iter()
            .map(|field| {
                if field.is_optional() {
                    None
                } else {
                    item(project, field.type_tag(), visiting)
                }
            })
            .collect::<Option<_>>()?;

        let mut fields: Vec<Option<FieldLayout>> = vec![None; items.len()];
        let mut offset: usize = 0;
        for group in ALIGNMENT_GROUPS {
            for (index, item) in items.iter().enumerate() {
                if let Item::Fixed { size, alignment } = *item
                    && alignment == group
                {
                    fields[index] = Some(FieldLayout::Fixed { offset, size });
                    offset += size;
                }
            }
            if group == LENGTH_ALIGNMENT {
                for (index, item) in items.iter().enumerate() {
                    if let Item::Length = item {
                        fields[index] = Some(FieldLayout::Length { offset });
                        offset += LENGTH_SIZE;
                    }
                }
            }
        }
        let mut bits: usize = 0;
        for (index, item) in items.iter().enumerate() {
            if let Item::Bit = item {
                let bit: u8 = (bits % 8) as u8;
                fields[index] = Some(FieldLayout::Bit {
                    offset: offset + bits / 8,
                    bit,
                });
                bits += 1;
            }
        }
        let head_size: usize = offset + bits.div_ceil(8);

        let largest: usize = items
            .iter()
            .map(|item| match item {
                Item::Fixed { alignment, .. } => *alignment,
                Item::Length => LENGTH_ALIGNMENT,
                Item::Bit => 1,
            })
            .max()
            .unwrap_or(1);
        Some(Self {
            fields: fields.into_iter().flatten().collect(),
            head_size,
            alignment: if head_size.is_multiple_of(largest) {
                largest
            } else {
                1
            },
            is_fixed: !items.iter().any(|item| matches!(item, Item::Length)),
        })
    }
}

impl StructLayout {
    //! Properties

    /// Gets the layouts of the fields, in declaration order.
    pub fn fields(&self) -> &[FieldLayout] {
        &self.fields
    }

    /// Gets the size of the head.
    pub fn head_size(&self) -> usize {
        self.head_size
    }

    /// Gets the alignment group of the struct as a fixed-size value in another struct's head.
    ///
    /// This is the largest alignment of its head's items, if its head's size is a multiple of it,
    /// & 1 otherwise.
    pub fn alignment(&self) -> usize {
        self.alignment
    }

    /// Checks if the struct is fixed. (all its fields are fixed-size, so it has no data)
    pub fn is_fixed(&self) -> bool {
        self.is_fixed
    }
}

/// Gets the head item of the `tag`, without revisiting the `visiting` structs.
///
/// Returns `None` if the `tag` has no encoding yet.
fn item(project: &Project, tag: &TypeTag, visiting: &mut Vec<QualifiedName>) -> Option<Item> {
    let fixed = |size: usize| Item::Fixed {
        size,
        alignment: size.min(8),
    };
    match tag {
        TypeTag::Primitive(PrimitiveType::Boolean) => Some(Item::Bit),
        TypeTag::Primitive(primitive) => Some(fixed(primitive_size(*primitive))),
        TypeTag::Special(SpecialType::Uuid) => Some(fixed(16)),
        TypeTag::Special(SpecialType::String) => Some(Item::Length),
        TypeTag::Time(TimeType::Timestamp) => Some(fixed(8)),
        TypeTag::Time(TimeType::Date) => Some(fixed(4)),
        TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref())? {
            TypeDec::Enum(_) => Some(fixed(2)),
            TypeDec::Struct(structure) if !visiting.contains(name) => {
                visiting.push(name.clone());
                let layout: Option<StructLayout> =
                    StructLayout::new_visiting(project, structure, visiting);
                visiting.pop();
                let layout: StructLayout = layout?;
                if layout.is_fixed {
                    Some(Item::Fixed {
                        size: layout.head_size,
                        alignment: layout.alignment,
                    })
                } else {
                    Some(Item::Length)
                }
            }
            _ => None,
        },
        TypeTag::Slice { .. } | TypeTag::Map { .. } => None,
    }
}

/// Gets the size of the `primitive`, in bytes. (a `bool` field is a bit, not a byte)
fn primitive_size(primitive: PrimitiveType) -> usize {
    match primitive {
        PrimitiveType::Boolean | PrimitiveType::UnsignedInt8 | PrimitiveType::SignedInt8 => 1,
        PrimitiveType::UnsignedInt16 | PrimitiveType::SignedInt16 => 2,
        PrimitiveType::UnsignedInt32 | PrimitiveType::SignedInt32 | PrimitiveType::Float32 => 4,
        PrimitiveType::UnsignedInt64 | PrimitiveType::SignedInt64 | PrimitiveType::Float64 => 8,
        PrimitiveType::UnsignedInt128 | PrimitiveType::SignedInt128 => 16,
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::{FieldLayout, StructLayout};
    use proto_packet::io::TagNumber;
    use proto_packet_tree::{
        CaseName, Enum, EnumCase, FieldName, Message, ModPath, PrimitiveType, Project,
        QualifiedName, SchemaFile, SpecialType, Struct, StructField, TypeDec, TypeName, TypeTag,
    };

    /// Gets the type tag of the named type `name` in the mod `a`.
    fn named(name: &str) -> TypeTag {
        TypeTag::from(QualifiedName::try_from(format!("a.{}", name)).unwrap())
    }

    /// Creates the struct with the `name` & `fields`.
    fn structure(name: &str, fields: Vec<StructField>) -> Struct {
        fields.into_iter().fold(
            Struct::from(TypeName::try_from(name).unwrap()),
            |result, field| result.with_field(field).unwrap(),
        )
    }

    /// Creates the field with the `name` & `tag`.
    fn field(name: &str, tag: TypeTag) -> StructField {
        StructField::new(FieldName::try_from(name).unwrap(), tag)
    }

    /// Computes the layout of the struct `name` of the mod `a` with the `type_decs`.
    fn layout(type_decs: Vec<TypeDec>, name: &str) -> Option<StructLayout> {
        let schema_file: SchemaFile = type_decs
            .into_iter()
            .fold(SchemaFile::default(), |result, type_dec| {
                result.with_type_dec(type_dec).unwrap()
            });
        let project: Project = Project::default()
            .with_schema_file(ModPath::try_from("a").unwrap(), schema_file)
            .unwrap();
        let qualified: QualifiedName = QualifiedName::try_from(format!("a.{}", name)).unwrap();
        match project.type_dec_with_name(qualified.to_ref()) {
            Some(TypeDec::Struct(structure)) => StructLayout::new(&project, structure),
            _ => panic!("no struct `{}`", name),
        }
    }

    /// Gets the type tag of the `primitive`.
    fn primitive(primitive: PrimitiveType) -> TypeTag {
        TypeTag::from(primitive)
    }

    /// Gets the type tag of a `string`.
    fn string() -> TypeTag {
        TypeTag::from(SpecialType::String)
    }

    /// Gets the `Point` struct. (8 bytes, alignment 4)
    fn point() -> TypeDec {
        let x: StructField = field("x", primitive(PrimitiveType::SignedInt32));
        let y: StructField = field("y", primitive(PrimitiveType::SignedInt32));
        structure("Point", vec![x, y]).into()
    }

    #[test]
    fn format_example_my_struct() {
        let my_struct: Struct = structure(
            "MyStruct",
            vec![
                field("x", string()),
                field("y", primitive(PrimitiveType::UnsignedInt32)),
                field("z", primitive(PrimitiveType::UnsignedInt128)),
            ],
        );
        let result: StructLayout = layout(vec![my_struct.into()], "MyStruct").unwrap();
        assert_eq!(
            result.fields(),
            [
                FieldLayout::Length { offset: 20 },
                FieldLayout::Fixed {
                    offset: 16,
                    size: 4
                },
                FieldLayout::Fixed {
                    offset: 0,
                    size: 16
                },
            ]
        );
        assert_eq!(result.head_size(), 24);
        assert!(!result.is_fixed());
    }

    #[test]
    fn format_example_s() {
        let s: Struct = structure(
            "S",
            vec![
                field("a", string()),
                field("b", primitive(PrimitiveType::UnsignedInt32)),
                field("c", primitive(PrimitiveType::UnsignedInt128)),
                field("d", string()),
                field("e", primitive(PrimitiveType::Boolean)),
                field("f", primitive(PrimitiveType::UnsignedInt64)),
            ],
        );
        let result: StructLayout = layout(vec![s.into()], "S").unwrap();
        assert_eq!(
            result.fields(),
            [
                FieldLayout::Length { offset: 28 },
                FieldLayout::Fixed {
                    offset: 24,
                    size: 4
                },
                FieldLayout::Fixed {
                    offset: 0,
                    size: 16
                },
                FieldLayout::Length { offset: 32 },
                FieldLayout::Bit { offset: 36, bit: 0 },
                FieldLayout::Fixed {
                    offset: 16,
                    size: 8
                },
            ]
        );
        assert_eq!(result.head_size(), 37);
        assert!(!result.is_fixed());
    }

    #[test]
    fn format_example_t() {
        let odd: Struct = structure(
            "Odd",
            vec![
                field("a", primitive(PrimitiveType::UnsignedInt64)),
                field("b", primitive(PrimitiveType::UnsignedInt8)),
            ],
        );
        let t: Struct = structure(
            "T",
            vec![
                field("o", named("Odd")),
                field("p", named("Point")),
                field("n", primitive(PrimitiveType::UnsignedInt64)),
                field("g", primitive(PrimitiveType::Boolean)),
            ],
        );
        let type_decs = || vec![point(), odd.clone().into(), t.clone().into()];

        let result: StructLayout = layout(type_decs(), "Point").unwrap();
        assert_eq!((result.head_size(), result.alignment()), (8, 4));
        let result: StructLayout = layout(type_decs(), "Odd").unwrap();
        assert_eq!((result.head_size(), result.alignment()), (9, 1));

        let result: StructLayout = layout(type_decs(), "T").unwrap();
        assert_eq!(
            result.fields(),
            [
                FieldLayout::Fixed {
                    offset: 16,
                    size: 9
                },
                FieldLayout::Fixed { offset: 8, size: 8 },
                FieldLayout::Fixed { offset: 0, size: 8 },
                FieldLayout::Bit { offset: 25, bit: 0 },
            ]
        );
        assert_eq!((result.head_size(), result.alignment()), (26, 1));
        assert!(result.is_fixed());
    }

    #[test]
    fn bits_fill_bytes_in_declaration_order() {
        let mut fields: Vec<StructField> = (0..9)
            .map(|index| field(&format!("b{}", index), primitive(PrimitiveType::Boolean)))
            .collect();
        fields.push(field("n", primitive(PrimitiveType::UnsignedInt8)));
        let result: StructLayout = layout(vec![structure("Bits", fields).into()], "Bits").unwrap();
        assert_eq!(result.fields()[0], FieldLayout::Bit { offset: 1, bit: 0 });
        assert_eq!(result.fields()[7], FieldLayout::Bit { offset: 1, bit: 7 });
        assert_eq!(result.fields()[8], FieldLayout::Bit { offset: 2, bit: 0 });
        assert_eq!(
            result.fields()[9],
            FieldLayout::Fixed { offset: 0, size: 1 }
        );
        assert_eq!((result.head_size(), result.alignment()), (3, 1));
    }

    #[test]
    fn named_fields() {
        let color: Enum = Enum::from(TypeName::try_from("Color").unwrap())
            .with_case(EnumCase::new(
                CaseName::try_from("Red").unwrap(),
                TagNumber::new(1).unwrap(),
            ))
            .unwrap();
        let label: Struct = structure("Label", vec![field("text", string())]);
        let named_fields: Struct = structure(
            "NamedFields",
            vec![
                field("color", named("Color")),
                field("one", named("Point")),
                field("two", named("Point")).with_boxed(true),
                field("label", named("Label")),
            ],
        );
        let type_decs: Vec<TypeDec> =
            vec![color.into(), point(), label.into(), named_fields.into()];
        let result: StructLayout = layout(type_decs, "NamedFields").unwrap();
        assert_eq!(
            result.fields(),
            [
                FieldLayout::Fixed {
                    offset: 20,
                    size: 2
                },
                FieldLayout::Fixed { offset: 0, size: 8 },
                FieldLayout::Fixed { offset: 8, size: 8 },
                FieldLayout::Length { offset: 16 },
            ]
        );
        assert_eq!(result.head_size(), 22);
        assert!(!result.is_fixed());
    }

    #[test]
    fn fields_without_encodings() {
        let message: Message = Message::from(TypeName::try_from("Label").unwrap());
        let test_cases: Vec<(&str, StructField)> = vec![
            ("slice", field("x", string().into_slice())),
            ("map", field("x", TypeTag::map(string(), string()))),
            ("optional", field("x", string()).with_optional(true)),
            ("message", field("x", named("Label"))),
            ("loop", field("x", named("Loop")).with_boxed(true)),
        ];
        for (name, field) in test_cases {
            let type_decs: Vec<TypeDec> = vec![
                message.clone().into(),
                structure("Loop", vec![field]).into(),
            ];
            assert_eq!(layout(type_decs, "Loop"), None, "field: {}", name);
        }
    }
}
