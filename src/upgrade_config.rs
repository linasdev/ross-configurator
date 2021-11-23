use std::thread::sleep;
use std::time::Duration;
use std::fs::File;
use std::io::{Read, BufReader};

use ross_protocol::protocol::Protocol;
use ross_protocol::interface::serial::Serial;
use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::programmer::*;
use ross_protocol::event::general::*;
use ross_dsl::parser::Parser;
use ross_config::config::ConfigSerializer;

use crate::ross_configurator::*;
use crate::get_programmer::get_programmer;
use crate::get_devices::get_devices;

pub fn upgrade_config(protocol: &mut Protocol<Serial>, config: &str, address: u16) -> Result<(), ConfiguratorError>  {
    let programmer = get_programmer(protocol)?;
    let devices = get_devices(protocol)?;

    for device in devices.iter() {
        if device.bootloader_address == address {
            let file = match File::open(config) {
                Ok(file) => file,
                Err(err) => {
                    return Err(ConfiguratorError::IOError(err));
                }
            };

            let mut source_code = String::new();

            let mut reader = BufReader::new(file);
            reader.read_to_string(&mut source_code).map_err(|err| ConfiguratorError::IOError(err))?;

            let config = Parser::parse(&source_code).map_err(|err| ConfiguratorError::ParserError(err))?;
            let config_data = ConfigSerializer::serialize(&config).map_err(|err| ConfiguratorError::ConfigSerializerError(err))?;

            println!("{:?}", config_data);
            println!("Updating device's firmware (address: {:#06x}, firmware_size: {:#010x}).", address, buf.len());

            let programmer_start_config_upgrade_event = ProgrammerStartConfigUpgradeEvent {
                programmer_address: programmer.programmer_address,
                receiver_address: device.bootloader_address,
                config_size: config_data.len() as u32,
            };

            let _: AckEvent = match protocol.exchange_packet(programmer_start_config_upgrade_event.to_packet(), false, TRANSACTION_RETRY_COUNT as u32, || {
                sleep(Duration::from_millis(PACKET_TIMEOUT_MS))
            }) {
                Ok(event) => event,
                Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
            };

            let packet_count = (config_data.len() - 1) / DATA_PACKET_SIZE + 1;

            for i in 0..packet_count {
                let slice_start = i * DATA_PACKET_SIZE;
                let slice_offset = if i == packet_count - 1 {
                    if config_data.len() % DATA_PACKET_SIZE == 0 {
                        DATA_PACKET_SIZE
                    } else {
                        config_data.len() % DATA_PACKET_SIZE
                    }
                } else {
                    DATA_PACKET_SIZE
                };

                println!("Sending bytes {} - {}", slice_start, slice_start + slice_offset);

                let data = &config_data[slice_start..slice_start + slice_offset];

                let data_event = DataEvent {
                    transmitter_address: programmer.programmer_address,
                    receiver_address: device.bootloader_address,
                    data_len: data.len() as u16,
                    data: data.to_vec(),
                };

                let _: AckEvent = match protocol.exchange_packet(data_event.to_packet(), false, TRANSACTION_RETRY_COUNT as u32, || {
                    sleep(Duration::from_millis(PACKET_TIMEOUT_MS))
                }) {
                    Ok(event) => event,
                    Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
                };
            }

            return Ok(())
        }
    }

    Err(ConfiguratorError::DeviceNotFound)
}
