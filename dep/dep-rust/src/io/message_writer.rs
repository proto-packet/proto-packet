use crate::io::{UnknownField, message_index_len, write_end};

/// Responsible for appending an encoded message.
///
/// The fields are written in tag order. The unknown fields are written between them.
#[derive(Debug)]
pub struct MessageWriter<'a> {
    out: &'a mut Vec<u8>,
    tags: usize,
    ends: usize,
    values: usize,
    unknown: &'a [UnknownField],
}

impl<'a> MessageWriter<'a> {
    //! Construction

    /// Creates a new writer appending to the `out` bytes, for the `known` count of fields & the
    /// sorted `unknown` fields.
    pub fn new(out: &'a mut Vec<u8>, known: usize, unknown: &'a [UnknownField]) -> Self {
        let count: usize = known + unknown.len();
        let start: usize = out.len();
        let values: usize = start + message_index_len(count);
        out.resize(values, 0);
        out[start..start + 2].copy_from_slice(&(count as u16).to_le_bytes());
        Self {
            tags: start + 2,
            ends: start + 2 + 2 * count,
            values,
            out,
            unknown,
        }
    }
}

impl MessageWriter<'_> {
    //! Writing

    /// Starts the field with the `tag`, & gets the bytes to append its value to.
    pub fn field(&mut self, tag: u16) -> &mut Vec<u8> {
        self.write_unknown_before(u32::from(tag));
        self.write_tag(tag);
        self.out
    }

    /// Ends the field being written.
    ///
    /// # Note
    /// - The last field has no end, since its value ends where the message ends.
    pub fn end_field(&mut self) {
        if self.ends < self.values {
            write_end(self.out, self.ends, self.values);
            self.ends += 4;
        }
    }

    /// Finishes the message.
    pub fn finish(mut self) {
        self.write_unknown_before(u32::MAX);
    }

    /// Writes the unknown fields with tags below the `tag`.
    fn write_unknown_before(&mut self, tag: u32) {
        while let Some((field, rest)) = self.unknown.split_first()
            && u32::from(field.tag_number().value()) < tag
        {
            self.write_tag(field.tag_number().value());
            self.out.extend_from_slice(field.bytes());
            self.end_field();
            self.unknown = rest;
        }
    }

    /// Writes the `tag` to the index.
    fn write_tag(&mut self, tag: u16) {
        self.out[self.tags..self.tags + 2].copy_from_slice(&tag.to_le_bytes());
        self.tags += 2;
    }
}

/// Gets the length of the encoded `fields`, without their index.
#[must_use]
pub fn unknown_fields_len(fields: &[UnknownField]) -> usize {
    fields.iter().map(|field| field.bytes().len()).sum()
}
