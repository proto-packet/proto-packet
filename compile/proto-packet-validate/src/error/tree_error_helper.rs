use clerr::{Code, Report, Severity, TokenInfo};
use colored::ColoredString;
use lex::lexer::Span;
use proto_packet_parse::{
    EnumCaseTree, MessageFieldTree, SchemaFileTree, ServiceCallTree, StructFieldTree, TypeDecTree,
    VariantCaseTree,
};
use proto_packet_tree::TreeError;

/// Responsible for creating [Report]s from [TreeError]s.
#[derive(Copy, Clone, Debug)]
pub struct TreeErrorHelper<'a> {
    file_name: &'a str,
    error: &'a TreeError,
    tree: &'a SchemaFileTree,
    source: &'a str,
}

impl<'a> TreeErrorHelper<'a> {
    //! Construction

    /// Creates a new [TreeErrorHelper].
    pub const fn new(
        file_name: &'a str,
        error: &'a TreeError,
        tree: &'a SchemaFileTree,
        source: &'a str,
    ) -> Self {
        Self {
            file_name,
            error,
            tree,
            source,
        }
    }
}

impl TreeErrorHelper<'_> {
    //! Report

    /// Creates the report.
    pub fn report(&self) -> Report {
        let mut report: Report = Report::from(Code::error(self.code(), self.message()));
        self.entries()
            .drain(..)
            .for_each(|entry| report.add_entry(entry));
        report
    }
}

impl TreeErrorHelper<'_> {
    //! Report: Code

    /// The tree error code prefix.
    const TREE_ERROR_CODE_PREFIX: &'static str = "T_";

    /// Gets the report code.
    fn code(&self) -> String {
        let s: &str = match self.error {
            TreeError::DuplicateFieldName { .. } => "DUP_FIELD_NAME",
            TreeError::DuplicateCaseName { .. } => "DUP_CASE_NAME",
            TreeError::DuplicateCallName { .. } => "DUP_CALL_NAME",
            TreeError::DuplicateTagNumber { .. } => "DUP_TAG_NUMBER",
            TreeError::DuplicateTypeDecName { .. } => "DUP_TYPE_DEC_NAME",
            TreeError::DuplicateImportName { .. } => "DUP_IMPORT_NAME",
            TreeError::ImportNameConflictsWithTypeDec { .. } => "IMPORT_NAME_CONFLICT",
            TreeError::DuplicateModPathName { .. } => "DUP_MOD_PATH",
        };
        format!("{}{}", Self::TREE_ERROR_CODE_PREFIX, s)
    }
}

impl TreeErrorHelper<'_> {
    //! Report: Message

    /// Gets the report message.
    fn message(&self) -> String {
        match self.error {
            TreeError::DuplicateFieldName {
                field_name,
                type_name,
            } => format!("duplicate field name `{}` in `{}`", field_name, type_name),
            TreeError::DuplicateCaseName {
                case_name,
                type_name,
            } => format!("duplicate case name `{}` in `{}`", case_name, type_name),
            TreeError::DuplicateCallName {
                call_name,
                type_name,
            } => format!("duplicate call name `{}` in `{}`", call_name, type_name),
            TreeError::DuplicateTagNumber {
                type_name,
                tag_number,
            } => format!("duplicate tag number `{}` in `{}`", tag_number, type_name),
            TreeError::DuplicateTypeDecName { type_name } => {
                format!("duplicate type declaration `{}`", type_name)
            }
            TreeError::DuplicateImportName { type_name } => {
                format!("duplicate import `{}`", type_name)
            }
            TreeError::ImportNameConflictsWithTypeDec { type_name } => {
                format!("import `{}` conflicts with a type declaration", type_name)
            }
            TreeError::DuplicateModPathName { mod_path } => {
                format!("duplicate mod path `{}`", mod_path)
            }
        }
    }
}

impl<'a> TreeErrorHelper<'a> {
    //! Report: Entries

    /// Gets the report entries.
    fn entries(&self) -> Vec<Vec<ColoredString>> {
        match self.error {
            TreeError::DuplicateFieldName {
                field_name,
                type_name,
            } => {
                let Some(type_dec) = self
                    .tree
                    .type_decs_with_name(type_name, self.source)
                    .first()
                    .copied()
                else {
                    return Vec::default();
                };
                match type_dec {
                    TypeDecTree::Struct(s) => {
                        let fields: Vec<&StructFieldTree> =
                            s.fields_with_name(field_name, self.source);
                        Self::spans_to_entries(fields.iter().map(|f| f.field_name), self)
                    }
                    TypeDecTree::Message(m) => {
                        let fields: Vec<&MessageFieldTree> =
                            m.fields_with_name(field_name, self.source);
                        Self::spans_to_entries(fields.iter().map(|f| f.field_name), self)
                    }
                    TypeDecTree::Variant(_) | TypeDecTree::Enum(_) | TypeDecTree::Service(_) => {
                        Vec::default()
                    }
                }
            }
            TreeError::DuplicateCaseName {
                case_name,
                type_name,
            } => {
                let Some(type_dec) = self
                    .tree
                    .type_decs_with_name(type_name, self.source)
                    .first()
                    .copied()
                else {
                    return Vec::default();
                };
                match type_dec {
                    TypeDecTree::Variant(v) => {
                        let cases: Vec<&VariantCaseTree> =
                            v.cases_with_name(case_name, self.source);
                        Self::spans_to_entries(cases.iter().map(|c| c.case_name), self)
                    }
                    TypeDecTree::Enum(e) => {
                        let cases: Vec<&EnumCaseTree> = e.cases_with_name(case_name, self.source);
                        Self::spans_to_entries(cases.iter().map(|c| c.case_name), self)
                    }
                    TypeDecTree::Struct(_) | TypeDecTree::Message(_) | TypeDecTree::Service(_) => {
                        Vec::default()
                    }
                }
            }
            TreeError::DuplicateCallName {
                call_name,
                type_name,
            } => {
                let Some(type_dec) = self
                    .tree
                    .type_decs_with_name(type_name, self.source)
                    .first()
                    .copied()
                else {
                    return Vec::default();
                };
                match type_dec {
                    TypeDecTree::Service(s) => {
                        let calls: Vec<&ServiceCallTree> =
                            s.calls_with_name(call_name, self.source);
                        Self::spans_to_entries(calls.iter().map(|c| c.call_name), self)
                    }
                    TypeDecTree::Struct(_)
                    | TypeDecTree::Message(_)
                    | TypeDecTree::Variant(_)
                    | TypeDecTree::Enum(_) => Vec::default(),
                }
            }
            TreeError::DuplicateTagNumber {
                type_name,
                tag_number,
            } => {
                let Some(type_dec) = self
                    .tree
                    .type_decs_with_name(type_name, self.source)
                    .first()
                    .copied()
                else {
                    return Vec::default();
                };
                let tag_str: String = tag_number.to_string();
                match type_dec {
                    TypeDecTree::Message(m) => {
                        let spans: Vec<Span> = m
                            .fields
                            .iter()
                            .map(|f| f.tag_number)
                            .filter(|span| span.text(self.source) == tag_str.as_str())
                            .collect();
                        Self::spans_to_entries(spans, self)
                    }
                    TypeDecTree::Variant(v) => {
                        let spans: Vec<Span> = v
                            .cases
                            .iter()
                            .map(|c| c.tag_number)
                            .filter(|span| span.text(self.source) == tag_str.as_str())
                            .collect();
                        Self::spans_to_entries(spans, self)
                    }
                    TypeDecTree::Enum(e) => {
                        let spans: Vec<Span> = e
                            .cases
                            .iter()
                            .map(|c| c.tag_number)
                            .filter(|span| span.text(self.source) == tag_str.as_str())
                            .collect();
                        Self::spans_to_entries(spans, self)
                    }
                    TypeDecTree::Struct(_) | TypeDecTree::Service(_) => Vec::default(),
                }
            }
            TreeError::DuplicateTypeDecName { type_name } => {
                let type_decs: Vec<&TypeDecTree> =
                    self.tree.type_decs_with_name(type_name, self.source);
                Self::spans_to_entries(type_decs.iter().map(|d| d.type_name()), self)
            }
            TreeError::DuplicateImportName { type_name } => {
                let name_str: &str = type_name.as_ref();
                let spans: Vec<Span> = self
                    .tree
                    .imports
                    .iter()
                    .filter_map(|i| i.idents.last().copied())
                    .filter(|span| span.text(self.source) == name_str)
                    .collect();
                Self::spans_to_entries(spans, self)
            }
            TreeError::ImportNameConflictsWithTypeDec { type_name } => {
                let name_str: &str = type_name.as_ref();
                let mut spans: Vec<Span> = self
                    .tree
                    .imports
                    .iter()
                    .filter_map(|i| i.idents.last().copied())
                    .filter(|span| span.text(self.source) == name_str)
                    .collect();
                spans.extend(
                    self.tree
                        .type_decs_with_name(type_name, self.source)
                        .into_iter()
                        .map(|d| d.type_name()),
                );
                Self::spans_to_entries(spans, self)
            }
            TreeError::DuplicateModPathName { .. } => Vec::default(),
        }
    }

    /// Maps spans to report entries.
    fn spans_to_entries<I>(spans: I, helper: &Self) -> Vec<Vec<ColoredString>>
    where
        I: IntoIterator<Item = Span>,
    {
        spans
            .into_iter()
            .map(|span| helper.token_info(span).into())
            .collect()
    }

    fn token_info(&self, span: Span) -> TokenInfo<'a> {
        let (line, column): (usize, usize) = span.line_column(self.source);
        let line_text: &str = self.source.lines().nth(line).unwrap_or("");
        TokenInfo {
            severity: Severity::Error,
            file_name: self.file_name,
            line: line + 1,
            position: column + 1,
            line_text,
            token_len: span.len() as usize,
            message: "declared here",
        }
    }
}
