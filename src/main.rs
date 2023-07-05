#![allow(unused)]

use std::io::{BufRead, BufReader};
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use arduino_cli_client::commands::arduino_core_client::ArduinoCoreClient;
use arduino_cli_client::commands::{BoardListReq, InitReq};


/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        TyrCommands::Bootstrap { family, command } => {
            println!("Bootstrapping {} subcommand {:?}", family, command);

            let mut client = ArduinoCoreClient::connect("http://localhost:50051").await?;

            // Start a new instance of the Arduino Core Service
            let mut init_stream = client
                .init(InitReq {
                    library_manager_only: false,
                })
                .await?
                .into_inner();

            let resp_instance = init_stream.message().await?.expect("Failed to init");

            // List the boards currently connected to the computer.
            let resp_boards = client
                .board_list(BoardListReq {
                    instance: resp_instance.instance,
                })
                .await?
                .into_inner();

            print!("Boards: {:?}", resp_boards.ports);

        }
    }

    Ok(())
}
