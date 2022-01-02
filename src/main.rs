use clap::{clap_app, value_t};
use parse_int::parse;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;

use ross_dsl::Parser;
use ross_protocol::interface::serial::Serial;
use ross_protocol::protocol::{Protocol, BROADCAST_ADDRESS};

use ross_configurator::event_type::EventType;
use ross_configurator::get_devices::get_devices;
use ross_configurator::get_programmer::get_programmer;
use ross_configurator::ross_configurator::*;
use ross_configurator::send_event::send_event;
use ross_configurator::set_device_address::set_device_address;
use ross_configurator::upgrade_config::upgrade_config;
use ross_configurator::upgrade_firmware::upgrade_firmware;

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
        (@subcommand upgrade_firmware =>
            (about: "Upgrades a specific device's firmware")
            (@arg FIRMWARE: -f --firmware +required +takes_value "Path of the firmware to use")
            (@arg ADDRESS: -a --address +required +takes_value "Recipient device address")
        )
        (@subcommand upgrade_config =>
            (about: "Upgrades a specific device's config")
            (@arg CONFIG: -c --config +required +takes_value "Path of the config to use")
            (@arg ADDRESS: -a --address +required +takes_value "Recipient device address")
        )
        (@subcommand set_device_address =>
            (about: "Sets a specific device's address")
            (@arg NEW_ADDRESS: -n --("new-address") +required +takes_value "New device address")
            (@arg ADDRESS: -a --address +required +takes_value "Recipient device address")
        )
        (@subcommand send_event =>
            (about: "Sends a single event")
            (@arg EVENT: -e --event +required +takes_value "Type of the event")
            (@arg DATA: -d --data ... +required +takes_value "Data of the event")
        )
    )
    .get_matches();

    let device = matches.value_of("DEVICE").unwrap();
    let baudrate = match matches.value_of("BAUDRATE") {
        Some(baudrate_str) => match parse::<u64>(baudrate_str) {
            Ok(baudrate) => baudrate,
            Err(_) => {
                eprintln!("BAUDRATE is not a number.");
                return Err(ConfiguratorError::BadUsage);
            }
        },
        None => DEFAULT_BAUDRATE,
    };

    let mut protocol = {
        let port = match serialport::new(device, baudrate as u32)
            .timeout(Duration::from_millis(PACKET_TIMEOUT_MS))
            .open()
        {
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
        }
        ("get_devices", _) => {
            let programmer = get_programmer(&mut protocol)?;
            get_devices(&mut protocol, &programmer)?;
            Ok(())
        }
        ("upgrade_firmware", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let firmware = sub_matches.value_of("FIRMWARE").unwrap();
            let address = match parse::<u16>(sub_matches.value_of("ADDRESS").unwrap()) {
                Ok(address) => address,
                Err(_) => {
                    eprintln!("ADDRESS is not a number.");
                    return Err(ConfiguratorError::BadUsage);
                }
            };

            let programmer = get_programmer(&mut protocol)?;
            let devices = get_devices(&mut protocol, &programmer)?;

            upgrade_firmware(&mut protocol, &programmer, &devices, firmware, address)?;

            Ok(())
        }
        ("upgrade_config", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let config = sub_matches.value_of("CONFIG").unwrap();
            let address = match parse::<u16>(sub_matches.value_of("ADDRESS").unwrap()) {
                Ok(address) => address,
                Err(_) => {
                    eprintln!("ADDRESS is not a number.");
                    return Err(ConfiguratorError::BadUsage);
                }
            };

            let file = match File::open(config) {
                Ok(file) => file,
                Err(err) => {
                    return Err(ConfiguratorError::IOError(err));
                }
            };

            let mut source_code = String::new();

            let mut reader = BufReader::new(file);
            reader
                .read_to_string(&mut source_code)
                .map_err(|err| ConfiguratorError::IOError(err))?;

            let config = Parser::parse(&source_code).map_err(|err| {
                eprintln!("Parsing failed with error:");
                eprintln!("{}", err);

                ConfiguratorError::ParserError(err)
            })?;

            let programmer = get_programmer(&mut protocol)?;
            let devices = get_devices(&mut protocol, &programmer)?;

            upgrade_config(&mut protocol, &programmer, &devices, &config, address)?;

            Ok(())
        }
        ("set_device_address", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let new_address = match parse::<u16>(sub_matches.value_of("NEW_ADDRESS").unwrap()) {
                Ok(new_address) => new_address,
                Err(_) => {
                    eprintln!("NEW_ADDRESS is not a number.");
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

            let programmer = get_programmer(&mut protocol)?;
            let devices = get_devices(&mut protocol, &programmer)?;

            set_device_address(&mut protocol, &programmer, &devices, new_address, address)?;

            Ok(())
        }
        ("send_event", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let event = value_t!(sub_matches, "EVENT", EventType).unwrap_or_else(|e| e.exit());
            let data = sub_matches.values_of("DATA").unwrap().collect();

            get_programmer(&mut protocol)?;
            send_event(&mut protocol, event, data)?;

            Ok(())
        }
        (_, _) => Ok(()),
    }
}
