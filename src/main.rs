#![allow(unused)]

use home::home_dir;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use arduino_cli_client::commands::arduino_core_client::ArduinoCoreClient;
use arduino_cli_client::commands::{BoardListReq, InitReq};
use clap::builder::Str;
use config_file::FromConfigFile;
use serde::{Deserialize, Serialize};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: TyrCommands,
}

#[derive(Subcommand)]
enum TyrCommands {
    /// Sub commands related to the boostrap phase of the firmware lifecycle
    Bootstrap {
        family: String,
        #[command(subcommand)]
        command: TyrBootstrapCommands,
    }
}

#[derive(Subcommand, Debug)]
enum TyrBootstrapCommands {
    /// Show connected devices
    ListDevices,
}

#[derive(Serialize, Deserialize, Debug)]
struct TyrConfig {
    bootstrap: TyrBootstrapConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct TyrBootstrapConfig {
    arduino: TyrArduinoConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct TyrArduinoConfig {
    cli_path: String,
}

fn get_default_config_path() -> PathBuf {
    let mut config_path = home::home_dir().expect("home_dir() returned an invalid value");

    config_path.push(".tyr/config.toml");

    config_path
}

#[test]
fn test_get_default_config_path() {
    let mut config_path = get_default_config_path();
    println!("Config path: {:?}", config_path);
    config_path.pop();
    assert!(config_path.exists(), "Config path does not exist");
}

fn maybe_read_config(config_path: PathBuf) -> Result<TyrConfig> {

    let mut tyr_config = TyrConfig {
        bootstrap: TyrBootstrapConfig {
            arduino: TyrArduinoConfig {
                cli_path: String::from("arduino-cli")
            }
        }
    };

    if config_path.exists() {
        tyr_config = TyrConfig::from_config_file(config_path)?;
    } else {
        println!("Config file does not exist, creating it");
        let mut config_dir = config_path.clone();
        config_dir.pop();
        std::fs::create_dir_all(config_dir)?;
        std::fs::write(config_path, toml::to_string(&tyr_config)?)?;
    }

    Ok(tyr_config)
}

#[test]
fn test_maybe_read_config() {
    let mut config_path = get_default_config_path();

    let config = maybe_read_config(config_path).unwrap();

    assert!(config.bootstrap.arduino.cli_path.as_str().contains("arduino-cli"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let config = maybe_read_config(get_default_config_path())?;

    println!("Config: {:?}", config);

    match args.command {
        TyrCommands::Bootstrap { family, command } => {
            println!("Bootstrapping {} subcommand {:?}", family, command);


            // let mut client = ArduinoCoreClient::connect("http://localhost:50051").await?;
            //
            // // Start a new instance of the Arduino Core Service
            // let mut init_stream = client
            //     .init(InitReq {
            //         library_manager_only: false,
            //     })
            //     .await?
            //     .into_inner();
            //
            // let resp_instance = init_stream.message().await?.expect("Failed to init");
            //
            // // List the boards currently connected to the computer.
            // let resp_boards = client
            //     .board_list(BoardListReq {
            //         instance: resp_instance.instance,
            //     })
            //     .await?
            //     .into_inner();
            //
            // print!("Boards: {:?}", resp_boards.ports);

        }
    }

    Ok(())
}
