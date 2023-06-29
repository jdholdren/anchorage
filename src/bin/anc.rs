use anyhow::Result;
use clap::{arg, Command};

use std::fs::File;
use std::io::{stdin, Read};

fn cli() -> Command {
    Command::new("anc")
        .version("0.1.0")
        .author("James H. <jamesdholdren@gmail.com>")
        .about("interacts with a given anchorage server")
        .subcommand_required(true)
        .subcommand(
            Command::new("put")
                .about("puts a blob into the server")
                .arg(arg!([blob_location]).required(false)),
        )
}

fn main() -> Result<()> {
    // TODO: Take in some env config for where the server is

    match cli().get_matches().subcommand() {
        Some(("put", submatches)) => {
            // Read from either std in or read in the file
            let mut reader: Box<dyn Read> =
                if let Some(blob_location) = submatches.get_one::<String>("blob_location") {
                    Box::new(File::open(blob_location)?)
                } else {
                    // Read a max of 2MB from std in
                    Box::new(stdin())
                };

            let files = anchorage::chunk::create_chunks(&mut reader)?;
            for (path, _file) in files {
                println!("{:?}", path);
            }

            // TODO: Encrypt the file with a shared secret
            // TODO: Rehash file for content
            // TODO: Upload to storage
        }
        _ => unreachable!(),
    };

    Ok(())
}
