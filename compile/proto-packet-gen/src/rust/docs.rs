use code_gen::rust::WithComments;

/// A schema member line of a pps doc block, with its comments. (ex: `name: string = 1;`)
pub(crate) struct PpsLine<'a> {
    pub(crate) comments: &'a [String],
    pub(crate) line: String,
}

/// Adds the pps doc block of a type declaration to the `result`.
///
/// The block re-prints the schema declaration, since the generated fields are private & their
/// comments, tag numbers & optionality would not show in the docs otherwise.
pub(crate) fn add_pps_docs<R>(
    result: &mut R,
    keyword: &str,
    type_name: &str,
    comments: &[String],
    members: &[PpsLine<'_>],
) where
    R: WithComments,
{
    result.add_comment(" ```pps");
    for comment in comments {
        result.add_comment(format!(" //{}", comment));
    }
    result.add_comment(format!(" {} {} {{", keyword, type_name));
    for member in members {
        result.add_comment("");
        for comment in member.comments {
            result.add_comment(format!("     //{}", comment));
        }
        result.add_comment(format!("     {}", member.line));
    }
    result.add_comment(" }");
    result.add_comment(" ```");
}
