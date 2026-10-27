use crate::{LinkError, Location, Member};
use proto_packet_tree::{
    Project, QualifiedName, TypeDec, TypeTag, WithCaseName, WithFieldName, WithTypeName,
    WithTypeTag,
};
use std::collections::{BTreeMap, BTreeSet};

/// Checks the linked `project` has no loops of types without a `box`.
///
/// Fields & cases of named types are the links of the loops. Slices, maps & boxed links break them.
pub(crate) fn check_unboxed_loops(project: &Project) -> Result<(), LinkError> {
    let links: BTreeMap<QualifiedName, Vec<Link>> = links(project);
    let mut done: BTreeSet<&QualifiedName> = BTreeSet::default();
    for name in links.keys() {
        visit(name, &links, &mut done, &mut Vec::default())?;
    }
    Ok(())
}

/// An unboxed link from a type to the `to` type, at the `location`.
struct Link {
    to: QualifiedName,
    location: Location,
}

/// Gets the unboxed links of each data type in the `project`.
fn links(project: &Project) -> BTreeMap<QualifiedName, Vec<Link>> {
    let mut links: BTreeMap<QualifiedName, Vec<Link>> = BTreeMap::default();
    for (mod_path, schema_file) in project.schema_files() {
        for type_dec in schema_file.type_decs() {
            let type_name = type_dec.type_name();
            let mut type_links: Vec<Link> = Vec::default();
            let mut add = |tag: &TypeTag, is_boxed: bool, member: Member| {
                if let (TypeTag::Named(to), false) = (tag, is_boxed) {
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
                    add(field.type_tag(), field.is_boxed(), member)
                }),
                TypeDec::Message(message) => message.fields().iter().for_each(|field| {
                    let member: Member = Member::Field(field.field_name().into_owned());
                    add(field.type_tag(), field.is_boxed(), member)
                }),
                TypeDec::Variant(variant) => variant.cases().iter().for_each(|case| {
                    let member: Member = Member::Case(case.case_name().into_owned());
                    add(case.type_tag(), case.is_boxed(), member)
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
            return Err(LinkError::UnboxedLoop {
                location: link.location.clone(),
                names,
            });
        }
        visit(&link.to, links, done, path)?;
    }
    path.pop();
    done.insert(name);
    Ok(())
}
