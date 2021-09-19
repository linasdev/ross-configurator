use std::time::Duration;
use clap::{clap_app, value_t};
use parse_int::parse;

use ross_protocol::protocol::{Protocol, BROADCAST_ADDRESS};
use ross_protocol::interface::serial::Serial;

use crate::ross_configurator::*;
use crate::get_programmer::get_programmer;
use crate::get_devices::get_devices;
use crate::update_firmware::update_firmware;
use crate::send_event::send_event;
use crate::event_type::EventType;

mod ross_configurator;
mod get_programmer;
mod get_devices;
mod update_firmware;
mod send_event;
mod event_type;

fn main() -> Result<(), ConfiguratorError> {
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
        (@subcommand send_event =>
            (about: "Sends a single event")
            (@arg EVENT: -e --event +required +takes_value "Type of the event")
            (@arg DATA: -d --data ... +required +takes_value "Data of the event")
        )
    ).get_matches();

    let device = matches.value_of("DEVICE").unwrap();
    let baudrate = match matches.value_of("BAUDRATE") {
        Some(baudrate_str) => {
            match parse::<u64>(baudrate_str) {
                Ok(baudrate) => baudrate,
                Err(_) => {
                    eprintln!("BAUDRATE is not a number.");
                    return Err(ConfiguratorError::BadUsage);
                }
            }
        },
        None => DEFAULT_BAUDRATE,
    };    
    
    let mut protocol = {        
        let port = match serialport::new(device, baudrate as u32)
            .timeout(Duration::from_millis(TRANSACTION_RETRY_COUNT * PACKET_TIMEOUT_MS))
            .open() {
            Ok(port) => port,
            Err(err) => {
                eprintln!("Failed to open device.");
                return Err(ConfiguratorError::FailedToOpenDevice(err));
            }
        };

        let serial = Serial::new(port);
        Protocol::new(BROADCAST_ADDRESS, serial)
    };

    match matches.subcommand() {
        ("get_programmer", _) => {
            get_programmer(&mut protocol)?;
            Ok(())
        },
        ("get_devices", _) => {
            get_devices(&mut protocol)?;
            Ok(())
        },
        ("update_firmware", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let firmware = sub_matches.value_of("FIRMWARE").unwrap();
            let version = match parse::<u32>(sub_matches.value_of("VERSION").unwrap()) {
                Ok(version) => version,
                Err(_) => {
                    eprintln!("VERSION is not a number.");
                    return Err(ConfiguratorError::BadUsage);
                }
            };
            let address = match parse::<u16>(sub_matches.value_of("ADDRESS").unwrap()) {
                Ok(address) => address,
                Err(_) => {
                    eprintln!("ADDRESS is not a number.");
                    return Err(ConfiguratorError::BadUsage);
                }
            };

            update_firmware(&mut protocol, firmware, version, address)?;

            Ok(())
        },
        ("send_event", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let event = value_t!(sub_matches, "EVENT", EventType).unwrap_or_else(|e| e.exit());
            let data = sub_matches.values_of("DATA").unwrap().collect();

            send_event(&mut protocol, event, data)?;

            Ok(())
        },
        (_, _) => {
            Ok(())
        },
    }
}
