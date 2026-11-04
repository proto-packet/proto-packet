use crate::SchemaSource;
use clerr::{Code, Report};
use lex::lexer::Span;
use proto_packet_link::{LinkError, Location, Member};
use proto_packet_parse::{SchemaFileTree, TypeDecTree, TypeTagTree, join_idents, span_report};
use proto_packet_tree::{ModPathRef, Project, QualifiedName};

/// Links the `project` of the schema `sources`.
///
/// Link errors are located in their schema files by reparsing them, since the tree has no spans.
pub fn link(project: &Project, sources: &[SchemaSource]) -> Result<Project, Report> {
    proto_packet_link::link_project(project).map_err(|error| link_report(&error, sources))
}

/// Creates the report for the link `error`, located by reparsing its schema file in the `sources`.
fn link_report(error: &LinkError, sources: &[SchemaSource]) -> Report {
    let message: String = error.to_string();
    let unlocated: Code = Code::error(code(error), message.as_str());
    let (mod_path, locate): (ModPathRef<'_>, Locate) = match error {
        LinkError::UnresolvedType { location, name }
        | LinkError::ServiceAsType { location, name } => {
            (location.mod_path.to_ref(), Locate::TypeTag(location, name))
        }
        LinkError::UnboxedLoop { location, names }
        | LinkError::RequiredLoop { location, names } => {
            let Some(name) = names.last() else {
                return Report::from(unlocated);
            };
            (location.mod_path.to_ref(), Locate::TypeTag(location, name))
        }
        LinkError::UnresolvedImport { mod_path, name }
        | LinkError::ImportedService { mod_path, name } => {
            (mod_path.to_ref(), Locate::Import(name.value()))
        }
    };
    let Some(source) = sources.iter().find(|source| source.mod_path == mod_path) else {
        return Report::from(unlocated);
    };
    let Ok(tree) = proto_packet_parse::parse(&source.source) else {
        return Report::from(unlocated);
    };
    match locate.span(&tree, &source.source) {
        Some(span) => span_report(
            code(error),
            &message,
            span,
            &source.path.display().to_string(),
            &source.source,
        ),
        None => Report::from(unlocated),
    }
}

/// Gets the error code of the link `error`.
fn code(error: &LinkError) -> &'static str {
    match error {
        LinkError::UnresolvedType { .. } => "L_UNRESOLVED_TYPE",
        LinkError::ServiceAsType { .. } => "L_SERVICE_AS_TYPE",
        LinkError::UnresolvedImport { .. } => "L_UNRESOLVED_IMPORT",
        LinkError::ImportedService { .. } => "L_IMPORTED_SERVICE",
        LinkError::UnboxedLoop { .. } => "L_UNBOXED_LOOP",
        LinkError::RequiredLoop { .. } => "L_REQUIRED_LOOP",
    }
}

/// What to locate in a schema file.
enum Locate<'a> {
    /// The named type with the qualified name, in the type tag at the location.
    TypeTag(&'a Location, &'a QualifiedName),

    /// The import with the qualified name.
    Import(&'a str),
}

impl Locate<'_> {
    /// Gets the span of what to locate in the schema file `tree` of the `source`.
    fn span(&self, tree: &SchemaFileTree, source: &str) -> Option<Span> {
        match self {
            Self::TypeTag(location, name) => {
                let type_name: &str = location.type_name.value();
                let type_dec: &TypeDecTree = tree
                    .type_decs
                    .iter()
                    .find(|type_dec| type_dec.type_name().text(source) == type_name)?;
                let tag: &TypeTagTree = member_type_tag(type_dec, &location.member, source)?;
                name_span(tag, name.to_ref().type_name().value(), source)
            }
            Self::Import(name) => tree
                .imports
                .iter()
                .find(|import| join_idents(&import.name.idents, source) == *name)
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

/// Gets the span of the first name of the `type_name` in the type `tag` of the `source`.
///
/// The name may have a mod path. (ex: `a.Foo` for `Foo`)
fn name_span(tag: &TypeTagTree, type_name: &str, source: &str) -> Option<Span> {
    match tag {
        TypeTagTree::Name(name) => name
            .idents
            .last()
            .is_some_and(|ident| ident.text(source) == type_name)
            .then(|| name.span()),
    }
}
