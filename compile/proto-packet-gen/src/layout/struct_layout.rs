use crate::layout::{
    BitLayout, EntryLayout, FieldLayout, LayoutCache, TypeLayout, UnitLayout, ValueLayout,
};
use proto_packet::io::align_up;
use proto_packet_tree::{Project, Struct, StructField, WithTypeTag};
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
    pub fn new(project: &Project, structure: &Struct) -> Self {
        Self::new_cached(&mut LayoutCache::new(project), structure)
    }

    /// Gets the layout of the `structure` with the `cache`.
    pub(crate) fn new_cached(cache: &mut LayoutCache<'_>, structure: &Struct) -> Self {
        let alignment: usize = cache.alignment(structure);
        let fields: &[StructField] = structure.fields();
        let types: Vec<TypeLayout> = fields
            .iter()
            .map(|field| cache.type_layout(field.type_tag()))
            .collect();
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
                let has_presence: bool = field.is_optional() && !cache.is_enum(field.type_tag());
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

        Self {
            alignment,
            fixed_size,
            table_offset,
            fields,
            units,
            entries,
        }
    }
}

impl StructLayout {
    //! Properties

    /// Checks if the struct is fixed.
    pub fn is_fixed(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Gets the entries of the `fields` with the `types`: the variable structs, messages & variants by
/// descending alignment, followed by the strings.
fn entries(fields: &[StructField], types: &[TypeLayout]) -> Vec<EntryLayout> {
    let mut variable: Vec<(usize, usize, bool)> = fields
        .iter()
        .zip(types)
        .enumerate()
        .filter_map(|(index, (field, layout))| match layout {
            TypeLayout::Variable { alignment } => {
                Some((index, *alignment, field.type_tag().is_string()))
            }
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
