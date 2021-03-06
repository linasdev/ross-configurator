use std::collections::BTreeSet;
use std::iter::IntoIterator;
use std::thread::sleep;
use std::time::Duration;

use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::bootloader::*;
use ross_protocol::event::programmer::*;
use ross_protocol::interface::serial::Serial;
use ross_protocol::protocol::Protocol;

use crate::ross_configurator::*;

pub fn get_devices(
    protocol: &mut Protocol<Serial>,
    programmer: &ProgrammerHelloEvent,
) -> Result<BTreeSet<BootloaderHelloEvent>, ConfiguratorError> {
    let devices: Vec<BootloaderHelloEvent> = match protocol.exchange_packets(
        programmer.to_packet(),
        false,
        || sleep(Duration::from_millis(PACKET_TIMEOUT_MS)),
    ) {
        Ok(event) => event,
        Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
    };

    let devices: BTreeSet<BootloaderHelloEvent> = devices.into_iter().collect();

    for bootloader_hello_event in devices.iter() {
        println!(
            "Found device (address: {:#06x})",
            bootloader_hello_event.bootloader_address
        );
    }

    Ok(devices)
}
