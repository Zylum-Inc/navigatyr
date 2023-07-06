#![allow(unused)]

use home::home_dir;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use arduino_cli_client::commands::arduino_core_client::ArduinoCoreClient;
use arduino_cli_client::commands::{BoardListReq, InitReq};
use clap::builder::Str;
use config_file::FromConfigFile;
use serde::Deserialize;

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

#[derive(Deserialize, Debug)]
struct Config {
    bootstrap: BootstrapConfig,
}

#[derive(Deserialize, Debug)]
struct BootstrapConfig {
    arduino: ArduinoConfig,
}

#[derive(Deserialize, Debug)]
struct ArduinoConfig {
    cli_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut config_path = home::home_dir().expect("home_dir() returned an invalid value");

    config_path.push(".tyr/config.toml");

    if let Some(config_path) = args.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    println!("Using config file: {}", config_path.display());

    let config = Config::from_config_file(config_path)?;

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
