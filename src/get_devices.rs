use std::thread::sleep;
use std::time::Duration;

use ross_protocol::ross_protocol::RossProtocol;
use ross_protocol::ross_interface::ross_serial::RossSerial;
use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_event::ross_bootloader_event::*;

use crate::ross_configurator::*;
use crate::get_programmer::get_programmer;

pub fn get_devices(protocol: &mut RossProtocol<RossSerial>) -> Result<Vec<RossBootloaderHelloEvent>, RossConfiguratorError>  {
    let programmer_hello_event = get_programmer(protocol)?;

    let devices: Vec<RossBootloaderHelloEvent> = match protocol.exchange_packets(programmer_hello_event.to_packet(), false, TRANSACTION_RETRY_COUNT as u32, || {
        sleep(Duration::from_millis(PACKET_TIMEOUT_MS))
    }) {
        Ok(event) => event,
        Err(err) => return Err(RossConfiguratorError::ProtocolError(err)),
    };

    for bootloader_hello_event in devices.iter() {
        println!("Found device (address: {:#06x}, firmware_version: {:#010x})", bootloader_hello_event.bootloader_address, bootloader_hello_event.firmware_version);
    }

    Ok(devices)
}
