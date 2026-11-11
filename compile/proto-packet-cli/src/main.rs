use clap::{Parser, Subcommand};
use clerr::Report;
use proto_packet_compile::{Language, path_report};
use proto_packet_lsp::LanguageServer;
use proto_packet_parse::SchemaFileTree;
use proto_packet_tree::SchemaFile;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The ProtoPacket compiler.
#[derive(Parser)]
#[command(name = "packet", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parses a schema file & prints its syntax tree.
    Parse {
        /// The schema file.
        file: PathBuf,
    },

    /// Validates a schema file & prints its semantic tree.
    Validate {
        /// The schema file.
        file: PathBuf,
    },

    /// Checks the project in the source folder for errors.
    ///
    /// Nothing is written. The code of each language is generated in memory, so every error that
    /// compiling would find is found.
    Check {
        /// The project's source folder.
        source: PathBuf,
    },

    /// Compiles the project in the source folder to code in the target folder.
    ///
    /// The contents of the target folder are replaced once the code is generated. The compile
    /// fails if the folder holds a file that was not generated.
    Compile {
        /// The target language. (rust)
        #[arg(value_parser = parse_language)]
        language: Language,

        /// The project's source folder.
        source: PathBuf,

        /// The target folder.
        target: PathBuf,
    },

    /// Runs the language server on the standard input & output.
    ///
    /// An editor starts it to show the errors of the schema files that are open.
    Lsp,
}

fn main() -> ExitCode {
    let result: Result<(), Report> = match Cli::parse().command {
        Command::Parse { file } => parse(&file),
        Command::Validate { file } => validate(&file),
        Command::Check { source } => proto_packet_compile::check(&source).map(|_| ()),
        Command::Compile {
            language,
            source,
            target,
        } => proto_packet_compile::compile(language, &source, &target),
        Command::Lsp => LanguageServer::serve_stdio(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(report) => {
            eprintln!("{}", report);
            ExitCode::FAILURE
        }
    }
}

/// Parses the language `name`.
fn parse_language(name: &str) -> Result<Language, String> {
    Language::from_name(name).ok_or_else(|| format!("unknown language `{}` (expected: rust)", name))
}

/// Parses the schema `file` & prints its syntax tree.
fn parse(file: &Path) -> Result<(), Report> {
    let source: String = read(file)?;
    println!("{:#?}", parse_tree(file, &source)?);
    Ok(())
}

/// Validates the schema `file` & prints its semantic tree.
fn validate(file: &Path) -> Result<(), Report> {
    let source: String = read(file)?;
    let tree: SchemaFileTree = parse_tree(file, &source)?;
    let schema_file: SchemaFile = proto_packet_validate::validate_schema_file(&tree, &source)
        .map_err(|error| error.report(&file.display().to_string(), &source))?;
    println!("{:#?}", schema_file);
    Ok(())
}

/// Parses the `source` of the schema `file`.
fn parse_tree(file: &Path, source: &str) -> Result<SchemaFileTree, Report> {
    proto_packet_parse::parse(source)
        .map_err(|error| error.report(&file.display().to_string(), source))
}

/// Reads the schema `file`.
fn read(file: &Path) -> Result<String, Report> {
    std::fs::read_to_string(file).map_err(|error| {
        path_report(
            "CLI_READ_FILE",
            "could not read the schema file",
            file,
            error,
        )
    })
}
