use std::thread::sleep;
use std::time::Duration;

use ross_protocol::protocol::Protocol;
use ross_protocol::interface::serial::Serial;
use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::programmer_event::*;
use ross_protocol::event::configurator_event::*;

use crate::ross_configurator::*;

pub fn get_programmer(protocol: &mut Protocol<Serial>) -> Result<ProgrammerHelloEvent, ConfiguratorError>  {
    let configurator_hello_event = ConfiguratorHelloEvent {};

    let programmer_hello_event: ProgrammerHelloEvent = match protocol.exchange_packet(configurator_hello_event.to_packet(), false, TRANSACTION_RETRY_COUNT as u32, || {
        sleep(Duration::from_millis(PACKET_TIMEOUT_MS))
    }) {
        Ok(event) => event,
        Err(err) => return Err(ConfiguratorError::ProtocolError(err)),
    };

    println!("Found programmer (address: {:#06x}, firmware_version: {:#010x})", programmer_hello_event.programmer_address, programmer_hello_event.firmware_version);

    Ok(programmer_hello_event)
}
