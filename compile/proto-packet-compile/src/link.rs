use crate::SchemaSource;
use clerr::{Code, Report, Severity, TokenInfo};
use lex::lexer::Span;
use proto_packet_link::{LinkError, Location, Member};
use proto_packet_parse::{SchemaFileTree, TypeDecTree, TypeTagTree};
use proto_packet_tree::{ModPathRef, Project};

/// Links the `project` of the schema `sources`.
///
/// Link errors are located in their schema files by reparsing them, since the tree has no spans.
pub fn link(project: &Project, sources: &[SchemaSource]) -> Result<Project, Report> {
    proto_packet_link::link_project(project).map_err(|error| link_report(&error, sources))
}

/// Creates the report for the link `error`, located by reparsing its schema file in the `sources`.
fn link_report(error: &LinkError, sources: &[SchemaSource]) -> Report {
    let message: String = error.to_string();
    let code: Code = Code::error(code(error), message.as_str());
    let (mod_path, locate): (ModPathRef<'_>, Locate) = match error {
        LinkError::UnresolvedType { location, .. }
        | LinkError::ServiceAsType { location, .. }
        | LinkError::UnboxedLoop { location, .. } => {
            (location.mod_path.to_ref(), Locate::TypeTag(location))
        }
        LinkError::UnresolvedImport { mod_path, name } => {
            (mod_path.to_ref(), Locate::Import(name.value()))
        }
    };
    let Some(source) = sources.iter().find(|source| source.mod_path == mod_path) else {
        return Report::from(code);
    };
    let Ok(tree) = proto_packet_parse::parse(&source.source) else {
        return Report::from(code);
    };
    match locate.span(&tree, &source.source) {
        Some(span) => {
            let file_name: String = source.path.display().to_string();
            let range = span.offset() as usize..span.end() as usize;
            let info: TokenInfo = TokenInfo::from_source(
                Severity::Error,
                &file_name,
                &source.source,
                range,
                &message,
            );
            Report::from(code).with_entry(info)
        }
        None => Report::from(code),
    }
}

/// Gets the error code of the link `error`.
fn code(error: &LinkError) -> &'static str {
    match error {
        LinkError::UnresolvedType { .. } => "L_UNRESOLVED_TYPE",
        LinkError::ServiceAsType { .. } => "L_SERVICE_AS_TYPE",
        LinkError::UnresolvedImport { .. } => "L_UNRESOLVED_IMPORT",
        LinkError::UnboxedLoop { .. } => "L_UNBOXED_LOOP",
    }
}

/// What to locate in a schema file.
enum Locate<'a> {
    /// The named type of the type tag at the location.
    TypeTag(&'a Location),

    /// The import with the qualified name.
    Import(&'a str),
}

impl Locate<'_> {
    /// Gets the span of what to locate in the schema file `tree` of the `source`.
    fn span(&self, tree: &SchemaFileTree, source: &str) -> Option<Span> {
        match self {
            Self::TypeTag(location) => {
                let type_name: &str = location.type_name.value();
                let type_dec: &TypeDecTree = tree
                    .type_decs
                    .iter()
                    .find(|type_dec| type_dec.type_name().text(source) == type_name)?;
                member_type_tag(type_dec, &location.member, source).map(name_span)
            }
            Self::Import(name) => tree
                .imports
                .iter()
                .find(|import| join(&import.name.idents, source) == *name)
                .map(|import| import.name.span()),
        }
    }
}

/// Gets the type tag of the `member` of the `type_dec`.
fn member_type_tag<'a>(
    type_dec: &'a TypeDecTree,
    member: &Member,
    source: &str,
) -> Option<&'a TypeTagTree> {
    let is = |span: Span, name: &str| span.text(source) == name;
    match (type_dec, member) {
        (TypeDecTree::Struct(structure), Member::Field(name)) => structure
            .fields
            .iter()
            .find(|field| is(field.field_name, name))
            .map(|field| &field.type_tag),
        (TypeDecTree::Message(message), Member::Field(name)) => message
            .fields
            .iter()
            .find(|field| is(field.field_name, name))
            .map(|field| &field.type_tag),
        (TypeDecTree::Variant(variant), Member::Case(name)) => variant
            .cases
            .iter()
            .find(|case| is(case.case_name, name))
            .map(|case| &case.type_tag),
        (TypeDecTree::Service(service), Member::Request(name)) => service
            .calls
            .iter()
            .find(|call| is(call.call_name, name))
            .map(|call| &call.request),
        (TypeDecTree::Service(service), Member::Response(name)) => service
            .calls
            .iter()
            .find(|call| is(call.call_name, name))
            .map(|call| &call.response),
        _ => None,
    }
}

/// Gets the span of the name in the type `tag`. (the base name of slices)
fn name_span(tag: &TypeTagTree) -> Span {
    match tag {
        TypeTagTree::Name(name) => name.span(),
        TypeTagTree::Slice { base } => name_span(base),
    }
}

/// Joins the identifier `spans` of the `source` with dots. (ex: `a.b.Foo`)
fn join(spans: &[Span], source: &str) -> String {
    spans
        .iter()
        .map(|span| span.text(source))
        .collect::<Vec<&str>>()
        .join(".")
}
