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
use toml::value::Array;

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
        /// Device ID
        device_id: String,
        /// Semantic Version String
        #[arg(short = 'v', long)]
        fimware_image_version: Option<String>,
        /// DevEUI
        #[arg(short = 'd', long)]
        deveui: Option<String>,
        /// AppEUI
        #[arg(short = 'a', long)]
        appeui: Option<String>,
        /// AppKey
        #[arg(short = 'k', long)]
        appkey: Option<String>,
    },
    /// Upload Image to S3
    UploadImage {
        /// Device Service Tag
        device_service_tag: String,
        /// Firmware Image Version
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
    arduino: TyrArduinoConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct TyrArduinoConfig {
    cli_path: String,
    #[serde(default)]
    sketch_path: String,
    #[serde(default)]
    board_type: String,
    #[serde(default)]
    devices_path: String,
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

fn write_config(config_path: PathBuf, tyr_config: TyrConfig) -> Result<()> {
    println!("Writing config to: {:?}", config_path);
    std::fs::write(config_path, toml::to_string(&tyr_config)?)?;
    Ok(())
}

fn maybe_read_config(config_path: PathBuf) -> Result<TyrConfig> {
    let mut tyr_config = TyrConfig {
        family: TyrFamilies::Arduino,
            arduino: TyrArduinoConfig {
                cli_path: String::from("arduino-cli"),
                board_type: String::from("adafruit:samd:adafruit_feather_m0"),
                sketch_path: String::from("C:\\Users\\siddg\\Documents\\Arduino\\libraries\\zeppylin-arduino-lorawan\\sketches\\chirpstack-otaa-us915a"),
                devices_path: String::from("C:\\Users\\siddg\\Documents\\Arduino\\libraries\\zeppylin-arduino-lorawan\\devices"),
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
                    "arduino-cli not found, please download and install it from https://arduino.github.io/arduino-cli/0.33/installation/")
}

#[test]
fn test_check_arduino_cli_install() {
    let result = check_arduino_cli_install();
    assert!(result.is_ok());
}

#[test]
fn test_maybe_read_config() {
    let mut config_path = get_default_config_path();

    let config = maybe_read_config(config_path).unwrap();

    assert!(config
        .arduino
        .cli_path
        .as_str()
        .contains("arduino-cli"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut config = maybe_read_config(get_default_config_path())?;

    match args.command {
        TyrCommands::GetConfig => {
            println!("Config: {:?}", config);
        },
        TyrCommands::SetConfig { family, arduino_board_type, arduino_sketch_path, arduino_devices_path } => {
            println!("Setting family to {:?}", family);
            config.family = family;
            if arduino_board_type.is_some() {
                println!("Setting arduino board type to {:?}", arduino_board_type);
                config.arduino.board_type = arduino_board_type.unwrap();
            }
            if arduino_sketch_path.is_some() {
                println!("Setting arduino sketch path to {:?}", arduino_sketch_path);
                config.arduino.sketch_path = arduino_sketch_path.unwrap();
            }
            if arduino_devices_path.is_some() {
                println!("Setting arduino devices path to {:?}", arduino_devices_path);
                config.arduino.devices_path = arduino_devices_path.unwrap();
            }
            write_config(get_default_config_path(), config)?;
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

                },
                TyrManufactureCommands::CreateImage { device_id, fimware_image_version,
                    deveui, appeui, appkey} => {
                    println!("Creating image for device {:?} with deveui {:?}, appeui {:?} and appkey {:?}", device_id, deveui, appeui, appkey);

                    if !(deveui.is_some() && appeui.is_some() && appkey.is_some()) {
                        println!("deveui, appeui and appkey must be specified");
                        std::process::exit(1);
                    }

                    if deveui.clone().expect("deveui should non NONE").len() != 16 {
                        println!("Invalid deveui {:?}, length must be 16", deveui);
                        std::process::exit(1);
                    }
                    if appeui.clone().expect("appeui should be non NONE").len() != 16 {
                        println!("Invalid appeui {:?}, length must be 16", appeui);
                        std::process::exit(1);
                    }
                    if appkey.clone().expect("appkey should be non NONE").len() != 32 {
                        println!("Invalid appkey {:?}, length must be 32", appkey);
                        std::process::exit(1);
                    }

                    let mut image_path = PathBuf::from(&config.arduino.devices_path);

                    image_path.push(device_id);

                    std::fs::create_dir_all(image_path.clone()).expect("Failed to create config directory");

                    println!("Image will be stored in {:?}", image_path);

                    let mut cpp_extra_flags = String::from("\"compiler.cpp.extra_flags=-DZAL_APPEUI_BIG_ENDIAN=");
                    cpp_extra_flags.push_str(String::from(&appeui.unwrap()).as_str());
                    cpp_extra_flags.push_str(" -DZAL_DEVEUI_BIG_ENDIAN=");
                    cpp_extra_flags.push_str(String::from(&deveui.unwrap()).as_str());
                    cpp_extra_flags.push_str(" -DZAL_APPKEY_BIG_ENDIAN=");
                    cpp_extra_flags.push_str(String::from(&appkey.unwrap()).as_str());
                    cpp_extra_flags.push_str(" \"");

                    process_command(&["arduino-cli", "compile", "-e", "-b", &config.arduino.board_type,
                        "--build-property", &cpp_extra_flags, "--output-dir", &image_path.as_path().display().to_string(), &config.arduino.sketch_path],
                                    "Failed to compile image");
                },
                TyrManufactureCommands::ListDevices => {
                    println!("Listing devices");
                    process_command(&["arduino-cli", "board", "list"],
                                    "No devices found, please connect a device and try again");
                },
                TyrManufactureCommands::FlashDevice { device_service_tag, fimware_image_version } => {
                    println!("Flashing device {:?} with version {:?}", device_service_tag, fimware_image_version);
                },
                TyrManufactureCommands::UploadImage { device_service_tag, fimware_image_version } => {
                    println!("Uploading image {:?} with version {:?}", device_service_tag, fimware_image_version);
                },
            }
        },
    }

    Ok(())
}
