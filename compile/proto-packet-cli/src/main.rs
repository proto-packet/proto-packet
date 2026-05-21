use clap::{Parser, Subcommand};
use clerr::Report;
use std::process::ExitCode;

mod sub;
mod util;

/// The ProtoPacket compiler CLI.
#[derive(Parser)]
#[command(name = "proto-packet", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parses a schema file. Prints the syntax tree.
    Parse(sub::ParseArgs),

    /// Validates a schema file. Prints the semantic tree.
    Validate(sub::ValidateArgs),

    /// Compiles a project. Generates source code from schema files.
    Compile(sub::CompileArgs),
}

fn main() -> ExitCode {
    let cli: Cli = Cli::parse();
    let result: Result<(), Report> = match cli.command {
        Command::Parse(args) => sub::parse(args),
        Command::Validate(args) => sub::validate(args),
        Command::Compile(args) => sub::compile(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(report) => {
            eprintln!("{}", report);
            ExitCode::FAILURE
        }
    }
}
