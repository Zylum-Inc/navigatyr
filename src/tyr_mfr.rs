use anyhow::Error;
use clap::{Parser, Subcommand, ValueEnum};

use crate::tyr_arduino;
use crate::tyr_config::TyrConfig;
use crate::tyr_utils::process_command;

#[derive(Subcommand, Debug)]
pub enum TyrManufactureCommands {
    /// Show available device images
    ListImages,
    /// Create a new firmware image
    CreateImage {
        /// Device ID
        device_id: String,
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

pub fn handle_manufacture_commands(command: TyrManufactureCommands, config: TyrConfig) -> Result<(), Error> {
    match command {
        TyrManufactureCommands::ListImages => Ok(()),
        TyrManufactureCommands::CreateImage { device_id} => {

            match config.family {
                crate::tyr_config::TyrFamilies::Arduino => {
                    println!("Creating image for device {:?} ", device_id);

                    tyr_arduino::compile(&device_id)?;
                }
                _ => {
                    println!("Family {:?} not supported", config.family);
                }
            }

            Ok(())
        }
        TyrManufactureCommands::ListDevices => {
            println!("Listing devices");
            process_command(
                &["arduino-cli", "board", "list"],
                "No devices found, please connect a device and try again",
            );
            Ok(())
        }
        TyrManufactureCommands::FlashDevice {
            device_service_tag,
            fimware_image_version,
        } => {
            println!(
                "Flashing device {:?} with version {:?}",
                device_service_tag, fimware_image_version
            );
            Ok(())
        }
        TyrManufactureCommands::UploadImage {
            device_service_tag,
            fimware_image_version,
        } => {
            println!(
                "Uploading image {:?} with version {:?}",
                device_service_tag, fimware_image_version
            );
            Ok(())
        }
    }
}
