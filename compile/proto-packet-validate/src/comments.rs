use lex::lexer::Span;
use proto_packet_tree::WithComments;

/// Adds the comment `spans` of the `source` to the `element`.
pub(crate) fn add_comments<E>(element: &mut E, spans: &[Span], source: &str)
where
    E: WithComments,
{
    for span in spans {
        element.add_comment(span.text(source));
    }
}
