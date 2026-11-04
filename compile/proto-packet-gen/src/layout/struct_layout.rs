use crate::layout::{
    BitLayout, EntryLayout, FieldLayout, TypeLayout, UnitLayout, ValueLayout, alignment,
    is_encodable,
};
use proto_packet::io::align_up;
use proto_packet_tree::{
    Project, QualifiedName, SpecialType, Struct, StructField, TypeDec, TypeTag, WithTypeTag,
};
use std::cmp::Reverse;

/// The layout of a struct's encoding. (see `FORMAT.md`)
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructLayout {
    /// The alignment.
    pub alignment: usize,

    /// The size of the fixed block, with its padding. (the size of a fixed struct)
    pub fixed_size: usize,

    /// The offset of the end table in the fixed block. (0 without entries)
    pub table_offset: usize,

    /// The fields, in declaration order.
    pub fields: Vec<FieldLayout>,

    /// The units of the fixed block, in order.
    pub units: Vec<UnitLayout>,

    /// The entries of the tail, in order.
    pub entries: Vec<EntryLayout>,
}

impl StructLayout {
    //! Construction

    /// Gets the layout of the `structure` of the linked `project`.
    ///
    /// Returns `None` if the `structure` contains a type with no encoding. (ex: a service)
    pub fn new(project: &Project, structure: &Struct) -> Option<Self> {
        let mut visited: Vec<QualifiedName> = Vec::default();
        let is_encodable: bool = structure
            .fields()
            .iter()
            .all(|field| is_encodable(project, field.type_tag(), &mut visited));
        if !is_encodable {
            return None;
        }
        let alignment: usize = alignment(project, structure)?;
        let fields: &[StructField] = structure.fields();
        let types: Vec<TypeLayout> = fields
            .iter()
            .map(|field| TypeLayout::new(project, field.type_tag()))
            .collect::<Option<_>>()?;
        let entries: Vec<EntryLayout> = entries(fields, &types);

        let mut units: Vec<UnitLayout> = Vec::default();
        let mut offsets: Vec<usize> = vec![0; fields.len()];
        let mut offset: usize = 0;
        let mut table_offset: usize = 0;
        for group in [8, 4, 2, 1] {
            for (index, layout) in types.iter().enumerate() {
                if let TypeLayout::Fixed { size, alignment } = *layout
                    && alignment == group
                {
                    units.push(UnitLayout::Field { index });
                    offsets[index] = offset;
                    offset += size;
                }
            }
            if group == 4 && !entries.is_empty() {
                units.push(UnitLayout::EndTable);
                table_offset = offset;
                offset += entries.len() * 4;
            }
        }

        let bits_offset: usize = offset;
        let mut bit_count: usize = 0;
        let mut next_bit = || {
            let bit: BitLayout = BitLayout {
                offset: bits_offset + bit_count / 8,
                bit: (bit_count % 8) as u8,
            };
            bit_count += 1;
            bit
        };
        let fields: Vec<FieldLayout> = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let has_presence: bool = field.is_optional() && !is_enum(project, field.type_tag());
                let presence: Option<BitLayout> = has_presence.then(&mut next_bit);
                let value: ValueLayout = match types[index] {
                    TypeLayout::Bit => ValueLayout::Bit(next_bit()),
                    TypeLayout::Fixed { size, .. } => ValueLayout::Fixed {
                        offset: offsets[index],
                        size,
                    },
                    TypeLayout::Variable { .. } => ValueLayout::Entry {
                        index: entries
                            .iter()
                            .position(|entry| entry.field == index)
                            .expect("each variable-size field has an entry"),
                    },
                };
                FieldLayout { value, presence }
            })
            .collect();
        if bit_count > 0 {
            let size: usize = bit_count.div_ceil(8);
            units.push(UnitLayout::Bits {
                offset: bits_offset,
                size,
            });
            offset += size;
        }

        let padded_alignment: usize = entries.first().map_or(alignment, |entry| entry.alignment);
        let fixed_size: usize = align_up(offset, padded_alignment);
        if fixed_size != offset {
            units.push(UnitLayout::Padding {
                size: fixed_size - offset,
            });
        }

        Some(Self {
            alignment,
            fixed_size,
            table_offset,
            fields,
            units,
            entries,
        })
    }
}

impl StructLayout {
    //! Properties

    /// Checks if the struct is fixed.
    pub fn is_fixed(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Gets the entries of the `fields` with the `types`: the variable structs & messages by descending
/// alignment, followed by the strings.
fn entries(fields: &[StructField], types: &[TypeLayout]) -> Vec<EntryLayout> {
    let mut variable: Vec<(usize, usize, bool)> = fields
        .iter()
        .zip(types)
        .enumerate()
        .filter_map(|(index, (field, layout))| match layout {
            TypeLayout::Variable { alignment } => Some((index, *alignment, is_string(field))),
            _ => None,
        })
        .collect();
    variable.sort_by_key(|(_, alignment, is_string)| (*is_string, Reverse(*alignment)));
    (0..variable.len())
        .map(|index| EntryLayout {
            field: variable[index].0,
            alignment: variable[index].1,
            padded_alignment: variable.get(index + 1).map_or(1, |next| next.1),
        })
        .collect()
}

/// Checks if the `field` is a string.
fn is_string(field: &StructField) -> bool {
    *field.type_tag() == TypeTag::Special(SpecialType::String)
}

/// Checks if the `tag` is an enum of the `project`.
fn is_enum(project: &Project, tag: &TypeTag) -> bool {
    match tag {
        TypeTag::Named(name) => matches!(
            project.type_dec_with_name(name.to_ref()),
            Some(TypeDec::Enum(_))
        ),
        _ => false,
    }
}
