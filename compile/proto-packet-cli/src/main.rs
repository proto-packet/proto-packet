use clap::{Parser, Subcommand};
use clerr::Report;
use proto_packet_compile::Language;
use std::path::PathBuf;
use std::process::ExitCode;

/// The ProtoPacket compiler.
#[derive(Parser)]
#[command(name = "proto-packet", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compiles the project in the source folder to code in the target folder.
    ///
    /// The target folder is replaced once the code is generated.
    Compile {
        /// The target language. (rust)
        #[arg(value_parser = parse_language)]
        language: Language,

        /// The project's source folder.
        source: PathBuf,

        /// The target folder.
        target: PathBuf,
    },
}

fn main() -> ExitCode {
    let result: Result<(), Report> = match Cli::parse().command {
        Command::Compile {
            language,
            source,
            target,
        } => proto_packet_compile::compile(language, &source, &target),
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
