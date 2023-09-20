#![allow(unused)]

use anyhow::{Context, Error, Result};
use clap::builder::Str;
use clap::{Parser, Subcommand, ValueEnum};
use config_file::FromConfigFile;
use home::home_dir;
use log::{debug, error, info, log_enabled, Level};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use strum_macros::{Display, EnumString};
use toml::value::Array;

pub mod tyr_arduino;
pub mod tyr_config;
pub mod tyr_mfr;
pub mod tyr_utils;

use crate::tyr_config::{get_config, set_config};
use crate::tyr_config::{TyrArduinoConfig, TyrConfig, TyrFamilies};
use crate::tyr_mfr::TyrManufactureCommands;
use crate::tyr_utils::process_command;

#[macro_use]
extern crate log;

use env_logger::Env;

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
    /// Get the currently set family
    GetConfig,
    /// Set the family of devices to work with
    SetConfig {
        #[arg(short, long, value_name = "FAMILY")]
        family: TyrFamilies,
        #[arg(short = 'b', long)]
        arduino_board_type: Option<String>,
        #[arg(short = 's', long)]
        arduino_sketch_path: Option<String>,
        #[arg(short = 'd', long)]
        arduino_devices_path: Option<String>,
    },
    /// Sub commands related to the boostrap phase of the firmware lifecycle
    Bootstrap {
        #[command(subcommand)]
        command: TyrBootstrapCommands,
    },
    /// Sub commands related to the provisioning phase of the firmware lifecycle
    Provision {
        #[command(subcommand)]
        command: TyrProvisionCommands,
    },
    /// Sub commands related to the manufactring phase of the firmware lifecycle
    Manufacture {
        #[command(subcommand)]
        command: TyrManufactureCommands,
    },
}

#[derive(Subcommand, Debug)]
enum TyrBootstrapCommands {
    /// Show devices
    ListDevices,
    /// Create a new device
    CreateDevice,
}

#[derive(Subcommand, Debug)]
enum TyrProvisionCommands {
    /// Show devices
    ListDevices,
    /// Provision a new network
    AddNetwork {
        /// Device Service Tag
        device_service_tag: String,
        /// The network name
        network_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
        .filter_or("TYR_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    if log_enabled!(Level::Debug) {
        println!("Debug logging enabled");
    }

    let args = Cli::parse();

    let config = tyr_config::get_config()?;

    match args.command {
        TyrCommands::GetConfig => {
            println!("Config: {:?}", config);
        }
        TyrCommands::SetConfig {
            family,
            arduino_board_type,
            arduino_sketch_path,
            arduino_devices_path,
        } => {
            println!("Setting family to {:?}", family);
            tyr_config::set_config(
                family,
                arduino_board_type,
                arduino_sketch_path,
                arduino_devices_path,
            )?;
        }
        TyrCommands::Bootstrap { command } => {
            debug!("Bootstrapping subcommand {:?}", command);
        }
        TyrCommands::Provision { command } => {
            debug!("Provisioning subcommand {:?}", command);
        }
        TyrCommands::Manufacture { command } => {
            debug!("Manufacturing subcommand {:?}", command);
            tyr_arduino::check_arduino_cli_install()?;
            tyr_mfr::handle_manufacture_commands(command, config)?;
        }
    }

    Ok(())
}
