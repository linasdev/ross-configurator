use std::thread::sleep;
use std::time::Duration;

use ross_protocol::protocol::Protocol;
use ross_protocol::interface::serial::Serial;
use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::programmer::*;
use ross_protocol::event::general::*;
use ross_protocol::event::bootloader::*;

use crate::ross_configurator::*;

pub fn set_device_address(protocol: &mut Protocol<Serial>, programmer: &ProgrammerHelloEvent, devices: &Vec<BootloaderHelloEvent>, new_address: u16, address: u16) -> Result<(), ConfiguratorError>  {
    for device in devices.iter() {
        if device.bootloader_address == address {
            println!("Updating device's address (address: {:#06x}, new_address: {:#06x}).", address, new_address);

            let programmer_set_device_address_event = ProgrammerSetDeviceAddressEvent {
                programmer_address: programmer.programmer_address,
                receiver_address: device.bootloader_address,
                new_address
            };

            let _: AckEvent = match protocol.exchange_packet(programmer_set_device_address_event.to_packet(), false, TRANSACTION_RETRY_COUNT as u32, || {
                sleep(Duration::from_millis(PACKET_TIMEOUT_MS))
            }) {
                Ok(event) => event,
                Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
            };

            return Ok(())
        }
    }

    Err(ConfiguratorError::DeviceNotFound)
}
