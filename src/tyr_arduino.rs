use std::path::{Path, PathBuf};
use anyhow::Error;

use crate::tyr_utils;
use crate::tyr_config;

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

pub fn compile(
    device_id: &str,
    deveui: &str,
    appeui: &str,
    appkey: &str,
) -> Result<(), Error> {

    let mut image_path = PathBuf::from(&tyr_config::get_arduino_device_path().unwrap());

    image_path.push(device_id);

    std::fs::create_dir_all(image_path.clone()).expect("Failed to create config directory");

    println!("Image will be stored in {:?}", image_path);

    let mut cpp_extra_flags = String::from("\"compiler.cpp.extra_flags=-DZAL_APPEUI_BIG_ENDIAN=");
    cpp_extra_flags.push_str(appeui);


    cpp_extra_flags.push_str(" -DZAL_DEVEUI_BIG_ENDIAN=");
    cpp_extra_flags.push_str(deveui);


    cpp_extra_flags.push_str(" -DZAL_APPKEY_BIG_ENDIAN=");
    //
    cpp_extra_flags.push_str(appkey);
    cpp_extra_flags.push_str(" \"");

    tyr_utils::process_command(&["arduino-cli", "compile", "-e", "-b", &tyr_config::get_arduino_board_type().unwrap(),
        "--build-property", &cpp_extra_flags, "--output-dir", &image_path.as_path().display().to_string(), &tyr_config::get_arduino_sketch_path().unwrap()],
                               "Failed to compile image");


    Ok(())
}