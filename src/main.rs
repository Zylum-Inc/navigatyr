#![allow(unused)]

use anyhow::{Context, Error, Result};
use arduino_cli_client::commands::arduino_core_client::ArduinoCoreClient;
use arduino_cli_client::commands::{BoardListReq, InitReq};
use clap::builder::Str;
use clap::{Parser, Subcommand, ValueEnum};
use config_file::FromConfigFile;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use strum_macros::{Display, EnumString};

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
    GetFamily,
    /// Set the family of devices to work with
    SetFamily {
        #[arg(value_enum)]
        family: TyrFamilies,
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

#[derive(Serialize, Deserialize, EnumString, Debug, Clone)]
enum TyrFamilies {
    #[strum(serialize = "arduino")]
    Arduino,
    // #[strum(serialize = "particle")]
    // Particle,
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

#[derive(Subcommand, Debug)]
enum TyrManufactureCommands {
    /// Show available device images
    ListImages,
    /// Create a new firmware image
    CreateImage {
        /// Device Service Tag
        device_service_tag: String,
        /// Semantic Version String
        fimware_image_version: String,
    },
    /// Show connected devices
    ListDevices,
    /// Flash a device
    FlashDevice {
        /// Device Service Tag
        device_service_tag: String,
        /// Firmware Image Version
        fimware_image_version: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct TyrConfig {
    family: TyrFamilies,
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

    config_path.push(".tyr");

    std::fs::create_dir_all(config_path.clone()).expect("Failed to create config directory");

    config_path.push("config.toml");

    config_path
}

#[test]
fn test_get_default_config_path() {
    let mut config_path = get_default_config_path();
    println!("Config path: {:?}", config_path);
    config_path.pop();
    assert!(
        config_path.exists(),
        "Config path: {:?} does not exist",
        config_path
    );
}

fn maybe_read_config(config_path: PathBuf) -> Result<TyrConfig> {
    let mut tyr_config = TyrConfig {
        family: TyrFamilies::Arduino,
        bootstrap: TyrBootstrapConfig {
            arduino: TyrArduinoConfig {
                cli_path: String::from("arduino-cli"),
            },
        },
    };

    println!("Config path: {:?}", config_path);

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

fn process_command(command: &[&str], error_msg: &str) -> Result<()> {

    println!("Running command: {:?}", command);

    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .arg("/C")
            .args(command)
            .output()
            .expect("failed to execute process")
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command.join(" "))
            .output()
            .expect("failed to execute process")
    };

    if !output.status.success() {
        println!("{}", error_msg);
        std::process::exit(1);
    } else {
        println!("Command output: {}", String::from_utf8_lossy(&output.stdout));
    }



    Ok(())
}


fn check_arduino_cli_install() -> Result<()> {

    // Check for arduino-cli
    // If it doesn't exist, throw an error
    process_command(&["arduino-cli", "version"],
                    "arduino-cli not found, please download and install it from https://arduino.github.io/arduino-cli/0.33/installation/");

    Ok(())
}

#[test]
fn test_check_arduino_cli_install() {
    let result = check_arduino_cli_install();
    assert!(result.is_err())
}

#[test]
fn test_maybe_read_config() {
    let mut config_path = get_default_config_path();

    let config = maybe_read_config(config_path).unwrap();

    assert!(config
        .bootstrap
        .arduino
        .cli_path
        .as_str()
        .contains("arduino-cli"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let config = maybe_read_config(get_default_config_path())?;

    println!("Config: {:?}", config);

    match args.command {
        TyrCommands::GetFamily => {
            println!("Family: {:?}", config.family);
        },
        TyrCommands::SetFamily { family } => {
            println!("Setting family to {:?}", family);
        },
        TyrCommands::Bootstrap { command } => {
            println!("Bootstrapping {:?} subcommand {:?}", config.family, command);
        },
        TyrCommands::Provision { command } => {
            println!("Provisioning {:?} subcommand {:?}", config.family, command);
        },
        TyrCommands::Manufacture { command } => {
            println!("Manufacturing {:?} subcommand {:?}", config.family, command);
            check_arduino_cli_install()?;

            match command {
                TyrManufactureCommands::ListImages => {
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg("arduino-cli core list")
                        .output()
                        .expect("failed to execute process");

                    println!("Output: {:?}", output);
                },
                TyrManufactureCommands::CreateImage { device_service_tag, fimware_image_version } => {
                    println!("Creating image for device {:?} with version {:?}", device_service_tag, fimware_image_version);
                },
                TyrManufactureCommands::ListDevices => {
                    println!("Listing devices");
                    process_command(&["arduino-cli", "board", "list"],
                                    "No devices found, please connect a device and try again");
                },
                TyrManufactureCommands::FlashDevice { device_service_tag, fimware_image_version } => {
                    println!("Flashing device {:?} with version {:?}", device_service_tag, fimware_image_version);
                },
            }
        },
    }

    Ok(())
}
