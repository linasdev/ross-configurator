use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::bootloader::*;
use ross_protocol::event::general::*;
use ross_protocol::event::programmer::*;
use ross_protocol::interface::serial::Serial;
use ross_protocol::protocol::Protocol;

use crate::ross_configurator::*;

pub fn upgrade_firmware(
    protocol: &mut Protocol<Serial>,
    programmer: &ProgrammerHelloEvent,
    devices: &BTreeSet<BootloaderHelloEvent>,
    firmware: &str,
    address: u16,
) -> Result<(), ConfiguratorError> {
    for device in devices.iter() {
        if device.bootloader_address == address {
            let mut file = match File::open(firmware) {
                Ok(file) => file,
                Err(err) => {
                    return Err(ConfiguratorError::IOError(err));
                }
            };

            let mut buf = vec![];

            if let Err(err) = file.read_to_end(&mut buf) {
                return Err(ConfiguratorError::IOError(err));
            }

            println!(
                "Updating device's firmware (address: {:#06x}, firmware_size: {:#010x}).",
                address,
                buf.len()
            );

            let programmer_start_upload_event = ProgrammerStartFirmwareUpgradeEvent {
                programmer_address: programmer.programmer_address,
                receiver_address: device.bootloader_address,
                firmware_size: buf.len() as u32,
            };

            let _: AckEvent = match protocol.exchange_packet(
                programmer_start_upload_event.to_packet(),
                false,
                TRANSACTION_RETRY_COUNT as u32,
                || sleep(Duration::from_millis(PACKET_TIMEOUT_MS)),
            ) {
                Ok(event) => event,
                Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
            };

            let packet_count = (buf.len() - 1) / DATA_PACKET_SIZE + 1;

            for i in 0..packet_count {
                let slice_start = i * DATA_PACKET_SIZE;
                let slice_offset = if i == packet_count - 1 {
                    if buf.len() % DATA_PACKET_SIZE == 0 {
                        DATA_PACKET_SIZE
                    } else {
                        buf.len() % DATA_PACKET_SIZE
                    }
                } else {
                    DATA_PACKET_SIZE
                };

                println!(
                    "Sending bytes {} - {}",
                    slice_start,
                    slice_start + slice_offset
                );

                let data = &buf[slice_start..slice_start + slice_offset];

                let data_event = DataEvent {
                    transmitter_address: programmer.programmer_address,
                    receiver_address: device.bootloader_address,
                    data_len: data.len() as u16,
                    data: data.to_vec(),
                };

                let _: AckEvent = match protocol.exchange_packet(
                    data_event.to_packet(),
                    false,
                    TRANSACTION_RETRY_COUNT as u32,
                    || sleep(Duration::from_millis(PACKET_TIMEOUT_MS)),
                ) {
                    Ok(event) => event,
                    Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
                };
            }

            return Ok(());
        }
    }

    Err(ConfiguratorError::DeviceNotFound)
}
