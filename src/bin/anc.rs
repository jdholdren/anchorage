use anchorage::blobserver::client::Client;
use anchorage::blobserver::server;
use anchorage::{Node, NodeType};
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
                .subcommand_required(true)
                .subcommand(Command::new("blob").arg(arg!([blob_location]).required(false))),
        )
        .subcommand(
            Command::new("get-blob")
                .about("gets a blob from the server")
                .arg(arg!([hash]).required(true)),
        )
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Take in some env config for where the server is
    let client = Client::default();

    match cli().get_matches().subcommand() {
        Some(("put", submatches)) => {
            match submatches.subcommand() {
                Some(("blob", submatches)) => {
                    let mut is_file = false;
                    // Read from either std in or read in the file
                    let mut reader: Box<dyn Read> = if let Some(blob_location) =
                        submatches.get_one::<String>("blob_location")
                    {
                        is_file = true;
                        Box::new(File::open(blob_location)?)
                    } else {
                        // Read a max of 2MB from std in
                        Box::new(stdin())
                    };

                    let files = anchorage::chunk::create_chunks(&mut reader)?;
                    let mut blobs = vec![];
                    for (_, mut file) in files {
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf)?;

                        let resp = client.put_blob(&buf).await?;
                        println!("{:?}", resp.created);

                        blobs.push(resp.created);
                    }

                    // If it's a file, the command should also create a node referring to the
                    // chunks created.
                    if is_file {
                        let resp = client
                            .create_node(server::CreateNodeRequest {
                                node_type: NodeType::File,
                                blobs,
                            })
                            .await?;
                        println!("{:?}", resp);
                    }
                }
                _ => unreachable!(),
            }
        }
        Some(("get-blob", submatches)) => {
            let hash = submatches.get_one::<String>("hash").unwrap();
            let resp = client.get_blob(hash).await?;
            print!("{:?}", resp)
        }
        _ => unreachable!(),
    };

    Ok(())
}
