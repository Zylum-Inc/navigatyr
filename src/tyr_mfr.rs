use clap::{Parser, Subcommand, ValueEnum};
use anyhow::Error;

use crate::tyr_utils::{process_command};
use crate::tyr_arduino;

#[derive(Subcommand, Debug)]
pub enum TyrManufactureCommands {
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


pub fn handle_manufacture_commands(command: TyrManufactureCommands) -> Result<(), Error> {
    match command {
        TyrManufactureCommands::ListImages => {
            Ok(())
        },
        TyrManufactureCommands::CreateImage {
            device_id, fimware_image_version,
            deveui, appeui, appkey
        } => {
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

            tyr_arduino::compile(&device_id,
                                 &String::from(&deveui.unwrap()).as_str(),
                                 &String::from(&appeui.unwrap()).as_str(),
                                 &String::from(&appkey.unwrap()).as_str())?;

            Ok(())

        },
        TyrManufactureCommands::ListDevices => {
            println!("Listing devices");
            process_command(&["arduino-cli", "board", "list"],
                                       "No devices found, please connect a device and try again");
            Ok(())
        },
        TyrManufactureCommands::FlashDevice { device_service_tag, fimware_image_version } => {
            println!("Flashing device {:?} with version {:?}", device_service_tag, fimware_image_version);
            Ok(())
        },
        TyrManufactureCommands::UploadImage { device_service_tag, fimware_image_version } => {
            println!("Uploading image {:?} with version {:?}", device_service_tag, fimware_image_version);
            Ok(())
        },
    }
}
