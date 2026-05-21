use crate::util;
use clap::Parser;
use clerr::Report;
use proto_packet_parse::SchemaFileTree;
use proto_packet_tree::SchemaFile;

#[derive(Parser)]
pub struct ValidateArgs {
    pub file: String,
}

pub fn validate(args: ValidateArgs) -> Result<(), Report> {
    let file_name: &str = util::file_name(args.file.as_str());
    let source: String = util::read_file(args.file.as_str())?;
    let tree: SchemaFileTree = proto_packet_parse::parse(file_name, source.clone())?;
    let schema: SchemaFile = proto_packet_validate::validate(file_name, &tree, &source)?;
    println!("{:#?}", schema);
    Ok(())
}
