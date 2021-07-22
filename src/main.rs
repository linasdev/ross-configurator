use std::time::Duration;
use clap::clap_app;
use parse_int::parse;

use crate::ross_configurator::{RossConfiguratorError, DEFAULT_BAUDRATE};
use crate::ross_serial::RossSerial;
use crate::get_programmer::get_programmer;
use crate::get_devices::get_devices;
use crate::update_firmware::update_firmware;

mod ross_configurator;
mod ross_serial;
mod get_programmer;
mod get_devices;
mod update_firmware;

fn main() -> Result<(), RossConfiguratorError> {
    let matches = clap_app!(ross_configurator =>
        (@setting SubcommandRequiredElseHelp)
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg DEVICE: -d --device +required +takes_value "Path of device to use")
        (@arg BAUDRATE: -b --baudrate +takes_value "Baudrate to use")
        (@subcommand get_programmer => 
            (about: "Gets connected programmer's information")
        )
        (@subcommand get_devices =>
            (about: "Gets connected devices' information")
        )
        (@subcommand update_firmware =>
            (about: "Updates a specific device's firmware")
            (@arg FIRMWARE: -f --firmware +required +takes_value "Path of the firmware to use")
            (@arg VERSION: -v --version +required +takes_value "New firmware's version")
            (@arg ADDRESS: -a --address +required +takes_value "Recipient device address")
        )
    ).get_matches();

    let device = matches.value_of("DEVICE").unwrap();
    let baudrate = match matches.value_of("BAUDRATE") {
        Some(baudrate_str) => {
            match parse::<u32>(baudrate_str) {
                Ok(baudrate) => baudrate,
                Err(_) => {
                    eprintln!("BAUDRATE is not a number.");
                    return Err(RossConfiguratorError::BadUsage);
                }
            }
        },
        None => DEFAULT_BAUDRATE,
    };    
    
    let mut serial = {        
        let port = match serialport::new(device, baudrate)
            .timeout(Duration::from_secs(2))
            .open() {
            Ok(port) => port,
            Err(err) => {
                eprintln!("Failed to open device.");
                return Err(RossConfiguratorError::FailedToOpenDevice(err));
            }
        };

        RossSerial::new(port)
    };

    match matches.subcommand() {
        ("get_programmer", _) => {
            get_programmer(&mut serial)?;
            Ok(())
        },
        ("get_devices", _) => {
            get_devices(&mut serial)?;
            Ok(())
        },
        ("update_firmware", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let firmware = sub_matches.value_of("FIRMWARE").unwrap();
            let version = match parse::<u32>(sub_matches.value_of("VERSION").unwrap()) {
                Ok(version) => version,
                Err(_) => {
                    eprintln!("VERSION is not a number.");
                    return Err(RossConfiguratorError::BadUsage);
                }
            };
            let address = match parse::<u16>(sub_matches.value_of("ADDRESS").unwrap()) {
                Ok(address) => address,
                Err(_) => {
                    eprintln!("ADDRESS is not a number.");
                    return Err(RossConfiguratorError::BadUsage);
                }
            };

            update_firmware(&mut serial, firmware, version, address)?;

            Ok(())
        },
        (_, _) => {
            Ok(())
        },
    }
}
