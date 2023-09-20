use anyhow::Error;
use config_file::FromConfigFile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, EnumString, Debug, Clone)]
pub enum TyrFamilies {
    #[strum(serialize = "arduino")]
    Arduino,
    // #[strum(serialize = "particle")]
    // Particle,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TyrConfig {
    family: TyrFamilies,
    arduino: TyrArduinoConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TyrArduinoConfig {
    cli_path: String,
    #[serde(default)]
    sketch_path: String,
    #[serde(default)]
    board_type: String,
    #[serde(default)]
    devices_path: String,
}

pub fn get_default_config_path() -> PathBuf {
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

pub fn write_config(config_path: PathBuf, tyr_config: TyrConfig) -> Result<(), Error> {
    debug!("Writing config to: {:?}", config_path);
    std::fs::write(config_path, toml::to_string(&tyr_config)?)?;
    Ok(())
}

pub fn maybe_read_config(config_path: PathBuf) -> Result<TyrConfig, Error> {
    let mut tyr_config = TyrConfig {
        family: TyrFamilies::Arduino,
        arduino: TyrArduinoConfig {
            cli_path: String::from("arduino-cli"),
            board_type: String::from("adafruit:samd:adafruit_feather_m0"),
            sketch_path: String::from("C:\\Users\\siddg\\Documents\\Arduino\\libraries\\zeppylin-arduino-lorawan\\sketches\\chirpstack-otaa-us915a"),
            devices_path: String::from("C:\\Users\\siddg\\Documents\\Arduino\\libraries\\zeppylin-arduino-lorawan\\devices"),
        },
    };

    debug!("Config path: {:?}", config_path);

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

    assert!(config.arduino.cli_path.as_str().contains("arduino-cli"));
}

pub fn set_config(
    family: TyrFamilies,
    arduino_board_type: Option<String>,
    arduino_sketch_path: Option<String>,
    arduino_devices_path: Option<String>,
) -> Result<(), Error> {
    let mut config_path = get_default_config_path();
    let mut config = maybe_read_config(config_path)?;

    config.family = family;
    if let Some(value) = arduino_board_type {
        println!("Setting arduino board type to {:?}", value);
        config.arduino.board_type = value;
    }
    if let Some(value) = arduino_sketch_path {
        println!("Setting arduino sketch path to {:?}", value);
        config.arduino.sketch_path = value;
    }
    if let Some(value) = arduino_devices_path {
        println!("Setting arduino devices path to {:?}", value);
        config.arduino.devices_path = value;
    }
    write_config(get_default_config_path(), config)?;

    Ok(())
}

#[test]
fn test_set_config() {
    let config = maybe_read_config(get_default_config_path()).unwrap();

    assert!(config.arduino.cli_path.as_str().contains("arduino-cli"));

    set_config(
        TyrFamilies::Arduino,
        Some(String::from("adafruit:samd:adafruit_feather_m0")),
        Some(String::from("C:\\Users\\siddg\\Documents\\Arduino\\libraries\\zeppylin-arduino-lorawan\\sketches\\chirpstack-otaa-us915a")),
        Some(String::from("C:\\Users\\siddg\\Documents\\Arduino\\libraries\\zeppylin-arduino-lorawan\\devices")),
    )
    .unwrap();

    let config = maybe_read_config(get_default_config_path()).unwrap();

    assert!(config
        .arduino
        .board_type
        .as_str()
        .contains("adafruit:samd:adafruit_feather_m0"));
}

pub fn get_config() -> Result<TyrConfig, Error> {
    let config_path = get_default_config_path();

    let config = maybe_read_config(config_path)?;

    Ok(config)
}

pub fn get_arduino_device_path() -> Result<String, Error> {
    let config = get_config()?;

    Ok(config.arduino.devices_path)
}

pub fn get_arduino_sketch_path() -> Result<String, Error> {
    let config = get_config()?;

    Ok(config.arduino.sketch_path)
}

pub fn get_arduino_board_type() -> Result<String, Error> {
    let config = get_config()?;

    Ok(config.arduino.board_type)
}
