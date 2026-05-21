use clap::Parser;
use clerr::{Code, Report};
use file_storage::{FolderPath, StoragePath};
use proto_packet_compile::{Compiler, Language};

#[derive(Parser)]
pub struct CompileArgs {
    /// The target language. (rust)
    pub language: String,

    /// The schema source folder.
    pub source: String,

    /// The target output folder.
    pub target: String,
}

pub fn compile(args: CompileArgs) -> Result<(), Report> {
    let language: Language = parse_language(args.language.as_str())?;
    let source: FolderPath = parse_folder(args.source.as_str())?;
    let target: FolderPath = parse_folder(args.target.as_str())?;
    let compiler: Compiler = Compiler::from(language);
    compiler.compile(&source, &target).map_err(Report::from)
}

fn parse_language(s: &str) -> Result<Language, Report> {
    match s {
        "rust" => Ok(Language::Rust),
        other => Err(Report::from(Code::error(
            "CLI_UNKNOWN_LANGUAGE",
            format!("unknown language '{}'", other),
        ))),
    }
}

fn parse_folder(s: &str) -> Result<FolderPath, Report> {
    let with_trailing: String = if s.ends_with('/') || s.ends_with('\\') {
        s.to_string()
    } else {
        format!("{}/", s)
    };
    let path: StoragePath = StoragePath::parse(with_trailing).map_err(|e| {
        Report::from(Code::error("CLI_INVALID_PATH", "invalid folder path"))
            .with_entry(vec![e.to_string().into()])
    })?;
    path.to_folder().map_err(|e| {
        Report::from(Code::error("CLI_INVALID_FOLDER", "path is not a folder"))
            .with_entry(vec![e.to_string().into()])
    })
}
