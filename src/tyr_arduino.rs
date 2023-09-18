use anyhow::Error;
use std::path::{Path, PathBuf};

extern crate yaml_rust;
use yaml_rust::{YamlEmitter, YamlLoader};

use crate::tyr_config;
use crate::tyr_utils;

pub fn check_arduino_cli_install() -> Result<(), Error> {
    // Check for arduino-cli
    // If it doesn't exist, throw an error
    tyr_utils::process_command(&["arduino-cli", "version"],
                               "arduino-cli not found, please download and install it from https://arduino.github.io/arduino-cli/0.33/installation/")
}

#[test]
fn test_check_arduino_cli_install() {
    let result = check_arduino_cli_install();
    assert!(result.is_ok());
}

pub fn get_cpp_extra_flags(device_id: &str, config_file: &Path) -> Result<String, Error> {
    if !config_file.exists() {
        return Err(Error::msg(format!(
            "Config file {:?} does not exist",
            config_file
        )));
    }

    let docs = YamlLoader::load_from_str(&std::fs::read_to_string(config_file)?)?;

    let doc = &docs[0];

    if device_id != doc["Device"]["id"].as_str().unwrap() {
        return Err(Error::msg(format!(
            "DeviceID in config file {:?} does not match device_id {:?}",
            config_file, device_id
        )));
    }

    let mut cpp_extra_flags = String::from(doc["CompileTimeConfigPrefix"].as_str().unwrap());

    let num_device_config_elements = doc["DeviceConfig"].as_vec().unwrap().len();

    println!(
        "num_device_config_elements: {:?}",
        num_device_config_elements
    );

    for i in 0..num_device_config_elements {
        println!(
            "i: {}, DeviceConfig[i]: {:?}, DeviceConfig[i][compile_time_prefix]: {:?}",
            i, doc["DeviceConfig"][i], doc["DeviceConfig"][i]["compile_time_prefix"]
        );
        cpp_extra_flags.push_str(
            doc["DeviceConfig"][i]["compile_time_prefix"]
                .as_str()
                .unwrap(),
        );
        cpp_extra_flags.push_str(doc["DeviceConfig"][i]["value"].as_str().unwrap());
        cpp_extra_flags.push_str(" ");
    }

    let num_network_config_elements = doc["Networks"][0]["config"].as_vec().unwrap().len();

    println!(
        "num_network_config_elements: {:?}",
        num_network_config_elements
    );

    for j in 0..num_network_config_elements {
        cpp_extra_flags.push_str(
            doc["Networks"][0]["config"][j]["compile_time_prefix"]
                .as_str()
                .unwrap(),
        );
        cpp_extra_flags.push_str(doc["Networks"][0]["config"][j]["value"].as_str().unwrap());
        cpp_extra_flags.push_str(" ");
    }

    println!("cpp_extra_flags: {:?}", cpp_extra_flags);

    return Ok(cpp_extra_flags);
}

#[test]
fn test_get_cpp_extra_flags() {
    let result = get_cpp_extra_flags(
        "216fa23d-8fda-4a17-8efa-93d45796dcf3",
        Path::new("test.yaml"),
    );

    if result.is_err() {
        println!("Error: {:?}", result.as_ref().err());
    }
    assert!(result.is_ok());
}

#[test]
fn test_get_cpp_extra_flags_device_id_mismatch() {
    let result = get_cpp_extra_flags(
        "216fa23d-8fda-4a17-8efa-93d45796dcf4",
        Path::new("test.yaml"),
    );

    if result.is_err() {
        println!("Error: {:?}", result.as_ref().err());
    }

    assert!(result.is_err());
}

pub fn compile(device_id: &str) -> Result<(), Error> {
    let mut image_path = PathBuf::from(&tyr_config::get_arduino_device_path().unwrap());

    image_path.push(device_id);

    std::fs::create_dir_all(image_path.clone()).expect("Failed to create config directory");

    println!("Image will be stored in {:?}", image_path);

    let mut device_config_file_path = image_path.clone();

    device_config_file_path.push("config.yaml");

    let cpp_extra_flags = get_cpp_extra_flags(device_id, device_config_file_path.as_path())?;

    tyr_utils::process_command(
        &[
            "arduino-cli",
            "compile",
            "-e",
            "-b",
            &tyr_config::get_arduino_board_type().unwrap(),
            "--build-property",
            &cpp_extra_flags,
            "--output-dir",
            &image_path.as_path().display().to_string(),
            &tyr_config::get_arduino_sketch_path().unwrap(),
        ],
        "Failed to compile image",
    );

    // tyr_utils::process_command(&["echo", cpp_extra_flags.as_str()], "Failed to compile image");

    Ok(())
}
