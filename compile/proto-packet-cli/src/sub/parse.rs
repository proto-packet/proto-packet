use crate::util;
use clap::Parser;
use clerr::Report;
use proto_packet_parse::SchemaFileTree;

#[derive(Parser)]
pub struct ParseArgs {
    pub file: String,
}

pub fn parse(args: ParseArgs) -> Result<(), Report> {
    let file_name: &str = util::file_name(&args.file);
    let source: String = util::read_file(args.file.as_str())?;
    let tree: SchemaFileTree = proto_packet_parse::parse(file_name, source)?;
    println!("{:#?}", tree);
    Ok(())
}
