use crate::layout::{
    BitLayout, EntryLayout, FieldLayout, LayoutCache, TypeLayout, UnitLayout, ValueLayout,
};
use proto_packet::io::align_up;
use proto_packet_tree::{Project, Struct, StructField, WithTypeTag};

/// The layout of a struct's encoding. (see `format/binary.md`)
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructLayout {
    /// The alignment.
    pub alignment: usize,

    /// The size of the fixed block. (the size of a fixed struct)
    pub fixed_size: usize,

    /// The offset of the end table in the fixed block. (0 without an end table)
    pub table_offset: usize,

    /// The fields, in declaration order.
    pub fields: Vec<FieldLayout>,

    /// The units of the fixed block, in order.
    pub units: Vec<UnitLayout>,

    /// The entries of the tail, in declaration order.
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
        let entries: Vec<EntryLayout> = entries(&types);
        let ends: usize = entries.len().saturating_sub(1);

        let is_struct: Vec<bool> = fields
            .iter()
            .map(|field| cache.is_with_structs(field.type_tag()))
            .collect();

        let mut units: Vec<UnitLayout> = Vec::default();
        let mut offsets: Vec<usize> = vec![0; fields.len()];
        let mut offset: usize = 0;
        let mut table_offset: usize = 0;
        for group in [8, 4, 2, 1] {
            for structs in [false, true] {
                if structs && group == 4 && ends > 0 {
                    offset = pad(&mut units, offset, 4);
                    units.push(UnitLayout::EndTable);
                    table_offset = offset;
                    offset += ends * 4;
                }
                for (index, layout) in types.iter().enumerate() {
                    if let TypeLayout::Fixed { size, alignment } = *layout
                        && alignment == group
                        && is_struct[index] == structs
                    {
                        offset = pad(&mut units, offset, alignment);
                        units.push(UnitLayout::Field { index });
                        offsets[index] = offset;
                        offset += size;
                    }
                }
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
                count: bit_count,
            });
            offset += size;
        }

        Self {
            alignment,
            fixed_size: offset,
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

    /// Gets the count of the ends in the end table.
    ///
    /// # Note
    /// - The last entry has none, since it ends where the struct ends.
    pub fn ends(&self) -> usize {
        self.entries.len().saturating_sub(1)
    }
}

/// Adds the padding that aligns the `offset` to the `units`, & gets the aligned offset.
///
/// # Note
/// - Only a struct field can end off a multiple of the next unit's alignment.
fn pad(units: &mut Vec<UnitLayout>, offset: usize, alignment: usize) -> usize {
    let aligned: usize = align_up(offset, alignment);
    if aligned != offset {
        units.push(UnitLayout::Padding {
            offset,
            size: aligned - offset,
        });
    }
    aligned
}

/// Gets the entries of the fields with the `types`, in declaration order.
fn entries(types: &[TypeLayout]) -> Vec<EntryLayout> {
    types
        .iter()
        .enumerate()
        .filter_map(|(field, layout)| match layout {
            TypeLayout::Variable { alignment } => Some(EntryLayout {
                field,
                alignment: *alignment,
            }),
            _ => None,
        })
        .collect()
}
