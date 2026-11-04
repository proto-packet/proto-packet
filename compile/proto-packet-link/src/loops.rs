use crate::{LinkError, Location, Member};
use proto_packet_tree::{
    Project, QualifiedName, TypeDec, TypeTag, WithCaseName, WithFieldName, WithTypeName,
    WithTypeTag,
};
use std::collections::{BTreeMap, BTreeSet};

/// Checks the linked `project` has no loops of types without a `box`, & no structs that need
/// themselves.
pub(crate) fn check_loops(project: &Project) -> Result<(), LinkError> {
    check_loops_of(project, LoopKind::Unboxed)?;
    check_loops_of(project, LoopKind::Required)
}

/// A kind of loop of types.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum LoopKind {
    /// Fields & cases of named types, without a `box`, since a value would have an infinite size.
    Unboxed,

    /// Required fields of structs, boxed or not, since a value would never end. An optional field
    /// ends a value, & so can another case of a variant.
    Required,
}

/// Checks the `project` has no loops of the `kind`.
fn check_loops_of(project: &Project, kind: LoopKind) -> Result<(), LinkError> {
    let links: BTreeMap<QualifiedName, Vec<Link>> = links(project, kind);
    let mut done: BTreeSet<&QualifiedName> = BTreeSet::default();
    for name in links.keys() {
        visit(name, &links, kind, &mut done, &mut Vec::default())?;
    }
    Ok(())
}

/// A link from a type to the `to` type, at the `location`.
struct Link {
    to: QualifiedName,
    location: Location,
}

/// Checks if the `tag` is a struct of the `project`.
fn is_struct(project: &Project, tag: &TypeTag) -> bool {
    match tag {
        TypeTag::Named(name) => matches!(
            project.type_dec_with_name(name.to_ref()),
            Some(TypeDec::Struct(_))
        ),
        _ => false,
    }
}

/// Gets the links of the `kind` of each data type in the `project`.
fn links(project: &Project, kind: LoopKind) -> BTreeMap<QualifiedName, Vec<Link>> {
    let mut links: BTreeMap<QualifiedName, Vec<Link>> = BTreeMap::default();
    for (mod_path, schema_file) in project.schema_files() {
        for type_dec in schema_file.type_decs() {
            let type_name = type_dec.type_name();
            let mut type_links: Vec<Link> = Vec::default();
            let mut add = |tag: &TypeTag, is_link: bool, member: Member| {
                if let (TypeTag::Named(to), true) = (tag, is_link) {
                    let location: Location = Location {
                        mod_path: mod_path.clone(),
                        type_name: type_name.into_owned(),
                        member,
                    };
                    type_links.push(Link {
                        to: to.clone(),
                        location,
                    });
                }
            };
            match type_dec {
                TypeDec::Struct(structure) => structure.fields().iter().for_each(|field| {
                    let member: Member = Member::Field(field.field_name().into_owned());
                    let is_link: bool = match kind {
                        LoopKind::Unboxed => !field.is_boxed(),
                        LoopKind::Required => {
                            !field.is_optional() && is_struct(project, field.type_tag())
                        }
                    };
                    add(field.type_tag(), is_link, member)
                }),
                TypeDec::Message(message) => message.fields().iter().for_each(|field| {
                    let member: Member = Member::Field(field.field_name().into_owned());
                    let is_link: bool = kind == LoopKind::Unboxed && !field.is_boxed();
                    add(field.type_tag(), is_link, member)
                }),
                TypeDec::Variant(variant) => variant.cases().iter().for_each(|case| {
                    let member: Member = Member::Case(case.case_name().into_owned());
                    let is_link: bool = kind == LoopKind::Unboxed && !case.is_boxed();
                    add(case.type_tag(), is_link, member)
                }),
                TypeDec::Enum(_) | TypeDec::Service(_) => {}
            }
            links.insert(
                QualifiedName::from_parts(mod_path.to_ref(), type_name),
                type_links,
            );
        }
    }
    links
}

/// Visits the `name` depth-first, failing on a link back into the `path` of visiting types.
fn visit<'a>(
    name: &'a QualifiedName,
    links: &'a BTreeMap<QualifiedName, Vec<Link>>,
    kind: LoopKind,
    done: &mut BTreeSet<&'a QualifiedName>,
    path: &mut Vec<&'a QualifiedName>,
) -> Result<(), LinkError> {
    if done.contains(name) {
        return Ok(());
    }
    path.push(name);
    for link in links.get(name).into_iter().flatten() {
        if let Some(start) = path.iter().position(|visiting| **visiting == link.to) {
            let mut names: Vec<QualifiedName> =
                path[start..].iter().map(|name| (*name).clone()).collect();
            names.push(link.to.clone());
            let location: Location = link.location.clone();
            return Err(match kind {
                LoopKind::Unboxed => LinkError::UnboxedLoop { location, names },
                LoopKind::Required => LinkError::RequiredLoop { location, names },
            });
        }
        visit(&link.to, links, kind, done, path)?;
    }
    path.pop();
    done.insert(name);
    Ok(())
}
