use serde::Deserialize;

/**
 * This binary runs the server
 **/

#[derive(Debug, Deserialize)]
struct Config {
    port: u16,
    storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StorageConfig {
    Local { directory: String },
}

#[tokio::main]
async fn main() {
    // Load some env config
    let config_path = std::env::var("CONFIG_PATH").expect("CONFIG_PATH not defined");
    let f = std::fs::File::open(config_path).expect("error opening file");
    let config: Config = serde_yaml::from_reader(f).expect("error deserializing config");

    println!("{:?}", config);

    // TODO: Set up blob server
    // TODO: Set up index server

    // TODO: Run the server
}
