#![allow(unused)]

use anyhow::{Context, Error, Result};
use arduino_cli_client::commands::arduino_core_client::ArduinoCoreClient;
use arduino_cli_client::commands::{BoardListReq, InitReq};
use clap::builder::Str;
use clap::{Parser, Subcommand};
use config_file::FromConfigFile;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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
    },
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
        bootstrap: TyrBootstrapConfig {
            arduino: TyrArduinoConfig {
                cli_path: String::from("arduino-cli"),
            },
        },
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


fn check_arduino_cli_install() -> Result<()> {

    // Check for arduino-cli
    // If it doesn't exist, install it
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "arduino-cli --version"])
            .output()
            .expect("failed to execute process")
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg("arduino-cli --version")
            .output()
            .expect("failed to execute process")
    };

    if !output.status.success() {
        anyhow::bail!("arduino-cli not found, please download and install it from https://arduino.github.io/arduino-cli/0.33/installation/");
    }

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
        TyrCommands::Bootstrap { family, command } => {
            println!("Bootstrapping {} subcommand {:?}", family, command);

            check_arduino_cli_install()?;

        }
    }

    Ok(())
}
