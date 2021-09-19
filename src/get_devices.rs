use std::thread::sleep;
use std::time::Duration;

use ross_protocol::protocol::Protocol;
use ross_protocol::interface::serial::Serial;
use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::bootloader_event::*;

use crate::ross_configurator::*;
use crate::get_programmer::get_programmer;

pub fn get_devices(protocol: &mut Protocol<Serial>) -> Result<Vec<BootloaderHelloEvent>, ConfiguratorError>  {
    let programmer_hello_event = get_programmer(protocol)?;

    let mut devices: Vec<BootloaderHelloEvent> = match protocol.exchange_packets(programmer_hello_event.to_packet(), false, TRANSACTION_RETRY_COUNT as u32, || {
        sleep(Duration::from_millis(PACKET_TIMEOUT_MS))
    }) {
        Ok(event) => event,
        Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
    };

    devices.dedup();

    for bootloader_hello_event in devices.iter() {
        println!("Found device (address: {:#06x}, firmware_version: {:#010x})", bootloader_hello_event.bootloader_address, bootloader_hello_event.firmware_version);
    }

    Ok(devices)
}
