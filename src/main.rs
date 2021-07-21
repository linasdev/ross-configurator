use clap::clap_app;

use crate::ross_configurator::{RossConfiguratorError, DEFAULT_BAUDRATE};
use crate::get_info::get_info;

mod ross_configurator;
mod get_info;

fn main() -> Result<(), RossConfiguratorError> {
    let matches = clap_app!(ross_configurator =>
        (@setting SubcommandRequiredElseHelp)
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg DEVICE: -d --device +required +takes_value "Path of device to use")
        (@arg BAUDRATE: -b --baudrate +takes_value "Baudrate to use")
        (@subcommand get_info => 
            (about: "Gets connected programmer information")
        )
    ).get_matches();

    let device = matches.value_of("DEVICE").unwrap();
    let baudrate = match matches.value_of("BAUDRATE") {
        Some(baudrate_str) => {
            match baudrate_str.parse::<u32>() {
                Ok(baudrate) => baudrate,
                Err(_) => {
                    eprintln!("BAUDRATE is not a number.");
                    return Err(RossConfiguratorError::BadUsage);
                }
            }
        },
        None => DEFAULT_BAUDRATE,
    };

    match matches.subcommand() {
        ("get_info", _) => {
            get_info(device, baudrate)
        },
        (_, _) => {
            Ok(())
        },
    }
}
