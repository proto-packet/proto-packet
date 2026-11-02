use proto_packet_tree::{CallName, CaseName, FieldName, ModPath, TypeName};
use std::fmt::{Display, Formatter};

/// The location of a type tag in a project. (ex: the `name` field of `a.b.Person`)
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Location {
    pub mod_path: ModPath,
    pub type_name: TypeName,
    pub member: Member,
}

/// The member of a type declaration with a type tag.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Member {
    /// A struct or message field.
    Field(FieldName),

    /// A variant case.
    Case(CaseName),

    /// The request of a service call.
    Request(CallName),

    /// The response of a service call.
    Response(CallName),
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (member, name): (&str, &str) = match &self.member {
            Member::Field(field_name) => ("field", field_name),
            Member::Case(case_name) => ("case", case_name),
            Member::Request(call_name) => ("call request", call_name),
            Member::Response(call_name) => ("call response", call_name),
        };
        write!(
            f,
            "the `{}` {} of `{}.{}`",
            name, member, self.mod_path, self.type_name
        )
    }
}
