use crate::rust::GenRust;
use code_gen::rust::WithComments as RustWithComments;
use proto_packet::io::TagNumber;
use proto_packet_tree::{WithComments, WithFieldName, WithTypeName, WithTypeTag};

impl GenRust<'_> {
    //! Comments

    /// A string with four spaces. (for comment indentation)
    const FOUR_SPACES: &'static str = "    ";

    pub(in crate::rust) fn gen_comments_type_dec<D, R, I>(
        self,
        type_label: &str,
        type_dec: &D,
        result: &mut R,
        inner: I,
    ) where
        D: WithComments + WithTypeName,
        R: RustWithComments,
        I: Fn(&D, &mut R),
    {
        result.add_comment(" ```pps");
        for comment in type_dec.comments() {
            result.add_comment(format!(" //{}", comment));
        }
        result.add_comment(format!(" {} {} {{", type_label, type_dec.type_name()));
        inner(type_dec, result);
        result.add_comment(" }");
        result.add_comment(" ```");
    }

    /// Emits comments for the `fields`. `tag_numbers` must have the same length as `fields`; each
    /// `Some(n)` appends `= n` to the field's comment line, and each `None` omits the suffix.
    pub(in crate::rust) fn gen_comments_fields<F, R>(
        self,
        fields: &[F],
        tag_numbers: &[Option<TagNumber>],
        optional_flags: &[bool],
        result: &mut R,
    ) where
        F: WithComments + WithFieldName + WithTypeTag,
        R: RustWithComments,
    {
        debug_assert_eq!(fields.len(), tag_numbers.len());
        debug_assert_eq!(fields.len(), optional_flags.len());
        for ((field, tag_number), is_optional) in fields
            .iter()
            .zip(tag_numbers.iter().copied())
            .zip(optional_flags.iter().copied())
        {
            result.add_comment(Self::FOUR_SPACES);
            self.gen_comments_field(field, tag_number, is_optional, result);
        }
    }

    pub(in crate::rust) fn gen_comments_field<F, R>(
        self,
        field: &F,
        tag_number: Option<TagNumber>,
        is_optional: bool,
        result: &mut R,
    ) where
        F: WithComments + WithFieldName + WithTypeTag,
        R: RustWithComments,
    {
        for comment in field.comments() {
            result.add_comment(format!(" {}//{}", Self::FOUR_SPACES, comment));
        }
        let prefix: &str = if is_optional { "optional " } else { "" };
        let suffix: String = tag_number.map(|n| format!(" = {n}")).unwrap_or_default();
        result.add_comment(format!(
            " {}{}{}: {}{};",
            Self::FOUR_SPACES,
            prefix,
            field.field_name(),
            field.type_tag(),
            suffix,
        ));
    }
}
