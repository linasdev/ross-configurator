use clap::clap_app;
use parse_int::parse;

use crate::ross_configurator::{RossConfiguratorError, DEFAULT_BAUDRATE};
use crate::ross_serial::RossSerial;
use crate::get_programmer::get_programmer;
use crate::get_devices::get_devices;

mod ross_configurator;
mod ross_serial;
mod get_programmer;
mod get_devices;

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
        let port = match serialport::new(device, baudrate).open() {
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
        }
        (_, _) => {
            Ok(())
        },
    }
}
