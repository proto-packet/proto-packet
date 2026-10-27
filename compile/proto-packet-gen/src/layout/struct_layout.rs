use crate::layout::{BitLayout, FieldLayout, TypeLayout, ValueLayout};
use proto_packet_tree::{
    Project, QualifiedName, Struct, StructField, TypeDec, TypeTag, WithTypeTag,
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
/// followed by the lengths. The bit group has the `bool` bits & the presence bits of the optional
/// fields, in declaration order.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructLayout {
    fields: Vec<FieldLayout>,
    head_size: usize,
    alignment: usize,
    is_fixed: bool,
}

impl StructLayout {
    //! Construction

    /// Computes the layout of the `structure` of the linked `project`.
    ///
    /// Returns `None` if a field has no encoding yet. (ex: a slice)
    pub fn new(project: &Project, structure: &Struct) -> Option<Self> {
        Self::new_visiting(project, structure, &mut Vec::default())
    }

    /// Computes the layout of the `structure`, with the `visiting` structs as variable-size.
    pub(crate) fn new_visiting(
        project: &Project,
        structure: &Struct,
        visiting: &mut Vec<QualifiedName>,
    ) -> Option<Self> {
        let items: Vec<TypeLayout> = structure
            .fields()
            .iter()
            .map(|field| TypeLayout::new_visiting(project, field.type_tag(), visiting))
            .collect::<Option<_>>()?;

        let mut values: Vec<Option<ValueLayout>> = vec![None; items.len()];
        let mut offset: usize = 0;
        for group in ALIGNMENT_GROUPS {
            for (index, item) in items.iter().enumerate() {
                if let TypeLayout::Fixed { size, alignment } = *item
                    && alignment == group
                {
                    values[index] = Some(ValueLayout::Fixed { offset, size });
                    offset += size;
                }
            }
            if group == LENGTH_ALIGNMENT {
                for (index, item) in items.iter().enumerate() {
                    if let TypeLayout::Variable = item {
                        values[index] = Some(ValueLayout::Length { offset });
                        offset += LENGTH_SIZE;
                    }
                }
            }
        }

        let mut bits: usize = 0;
        let mut next_bit = || {
            let bit: BitLayout = BitLayout {
                offset: offset + bits / 8,
                bit: (bits % 8) as u8,
            };
            bits += 1;
            bit
        };
        let mut fields: Vec<FieldLayout> = Vec::with_capacity(items.len());
        for ((field, item), value) in structure.fields().iter().zip(&items).zip(values) {
            let presence: Option<BitLayout> =
                has_presence_bit(project, field, *item).then(&mut next_bit);
            let value: ValueLayout = match (item, value) {
                (TypeLayout::Bit, _) => ValueLayout::Bit(next_bit()),
                (_, Some(value)) => value,
                (_, None) => unreachable!("a placed value"),
            };
            fields.push(FieldLayout { value, presence });
        }
        let head_size: usize = offset + bits.div_ceil(8);

        let largest: usize = items
            .iter()
            .map(|item| match item {
                TypeLayout::Fixed { alignment, .. } => *alignment,
                TypeLayout::Variable => LENGTH_ALIGNMENT,
                TypeLayout::Bit => 1,
            })
            .max()
            .unwrap_or(1);
        Some(Self {
            fields,
            head_size,
            alignment: if head_size.is_multiple_of(largest) {
                largest
            } else {
                1
            },
            is_fixed: !items
                .iter()
                .any(|item| matches!(item, TypeLayout::Variable)),
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

/// Checks if the `field` of the `project`, with the head `item`, has a presence bit.
///
/// An optional field has one unless its type has an absent value: the tag number 0 of an enum, or
/// the length `u32::MAX` of a variable-size value.
fn has_presence_bit(project: &Project, field: &StructField, item: TypeLayout) -> bool {
    let is_enum = || match field.type_tag() {
        TypeTag::Named(name) => matches!(
            project.type_dec_with_name(name.to_ref()),
            Some(TypeDec::Enum(_))
        ),
        _ => false,
    };
    field.is_optional() && !matches!(item, TypeLayout::Variable) && !is_enum()
}

#[cfg(test)]
mod tests {
    use crate::layout::{BitLayout, FieldLayout, StructLayout, ValueLayout};
    use proto_packet::io::TagNumber;
    use proto_packet_tree::{
        CaseName, Enum, EnumCase, FieldName, Message, ModPath, PrimitiveType, Project,
        QualifiedName, SchemaFile, SpecialType, Struct, StructField, TypeDec, TypeName, TypeTag,
    };

    /// Gets the layout of a required fixed-size field, of the `size`, at the `offset`.
    fn fixed(offset: usize, size: usize) -> FieldLayout {
        FieldLayout {
            value: ValueLayout::Fixed { offset, size },
            presence: None,
        }
    }

    /// Gets the layout of a required variable-size field, with its length at the `offset`.
    fn length(offset: usize) -> FieldLayout {
        FieldLayout {
            value: ValueLayout::Length { offset },
            presence: None,
        }
    }

    /// Gets the layout of a required `bool` field, at the `bit` of the byte at the `offset`.
    fn bit(offset: usize, bit: u8) -> FieldLayout {
        FieldLayout {
            value: ValueLayout::Bit(BitLayout { offset, bit }),
            presence: None,
        }
    }

    /// Gets the `layout` with the presence bit at the `bit` of the byte at the `offset`.
    fn present(layout: FieldLayout, offset: usize, bit: u8) -> FieldLayout {
        FieldLayout {
            presence: Some(BitLayout { offset, bit }),
            ..layout
        }
    }

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
        assert_eq!(result.fields(), [length(20), fixed(16, 4), fixed(0, 16),]);
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
                length(28),
                fixed(24, 4),
                fixed(0, 16),
                length(32),
                bit(36, 0),
                fixed(16, 8),
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
            [fixed(16, 9), fixed(8, 8), fixed(0, 8), bit(25, 0),]
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
        assert_eq!(result.fields()[0], bit(1, 0));
        assert_eq!(result.fields()[7], bit(1, 7));
        assert_eq!(result.fields()[8], bit(2, 0));
        assert_eq!(result.fields()[9], fixed(0, 1));
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
            [fixed(20, 2), fixed(0, 8), fixed(8, 8), length(16),]
        );
        assert_eq!(result.head_size(), 22);
        assert!(!result.is_fixed());
    }

    #[test]
    fn fields_without_encodings() {
        let message: Message = Message::from(TypeName::try_from("Label").unwrap());
        let test_cases: Vec<(&str, StructField)> = vec![
            ("list of messages", field("x", named("Label").into_slice())),
            ("map", field("x", TypeTag::map(string(), string()))),
            ("message", field("x", named("Label"))),
        ];
        for (name, field) in test_cases {
            let type_decs: Vec<TypeDec> = vec![
                message.clone().into(),
                structure("Loop", vec![field]).into(),
            ];
            assert_eq!(layout(type_decs, "Loop"), None, "field: {}", name);
        }
    }

    #[test]
    fn format_example_opt() {
        let color: Enum = Enum::from(TypeName::try_from("Color").unwrap())
            .with_case(EnumCase::new(
                CaseName::try_from("Red").unwrap(),
                TagNumber::new(1).unwrap(),
            ))
            .unwrap();
        let opt: Struct = structure(
            "Opt",
            vec![
                field("a", primitive(PrimitiveType::UnsignedInt32)).with_optional(true),
                field("b", primitive(PrimitiveType::Boolean)),
                field("c", string()).with_optional(true),
                field("d", primitive(PrimitiveType::Boolean)).with_optional(true),
                field("e", named("Color")).with_optional(true),
            ],
        );
        let result: StructLayout = layout(vec![color.into(), opt.into()], "Opt").unwrap();
        assert_eq!(
            result.fields(),
            [
                present(fixed(0, 4), 10, 0),
                bit(10, 1),
                length(4),
                present(bit(10, 3), 10, 2),
                fixed(8, 2),
            ]
        );
        assert_eq!(result.head_size(), 11);
        assert!(!result.is_fixed());
    }

    #[test]
    fn optional_fixed_structs_are_fixed() {
        let optional: Struct = structure(
            "Optional",
            vec![field("point", named("Point")).with_optional(true)],
        );
        let result: StructLayout = layout(vec![point(), optional.into()], "Optional").unwrap();
        assert_eq!(result.fields(), [present(fixed(0, 8), 8, 0)]);
        assert_eq!(result.head_size(), 9);
        assert!(result.is_fixed());
    }

    #[test]
    fn structs_containing_themselves_are_variable_size() {
        let list_node: Struct = structure(
            "ListNode",
            vec![
                field("value", primitive(PrimitiveType::UnsignedInt32)),
                field("next", named("ListNode"))
                    .with_optional(true)
                    .with_boxed(true),
            ],
        );
        let result: StructLayout = layout(vec![list_node.into()], "ListNode").unwrap();
        assert_eq!(result.fields(), [fixed(0, 4), length(4)]);
        assert_eq!(result.head_size(), 8);
        assert!(!result.is_fixed());
    }

    #[test]
    fn lists_are_variable_size() {
        let lists: Struct = structure(
            "Lists",
            vec![
                field("bytes", primitive(PrimitiveType::UnsignedInt8).into_slice()),
                field("points", named("Point").into_slice()),
                field("nested", string().into_slice().into_slice()),
            ],
        );
        let result: StructLayout = layout(vec![point(), lists.into()], "Lists").unwrap();
        assert_eq!(result.fields(), [length(0), length(4), length(8)]);
        assert!(!result.is_fixed());
    }
}
