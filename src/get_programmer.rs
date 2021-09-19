use std::thread::sleep;
use std::time::Duration;

use ross_protocol::ross_protocol::RossProtocol;
use ross_protocol::ross_interface::ross_serial::RossSerial;
use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_event::ross_programmer_event::*;
use ross_protocol::ross_event::ross_configurator_event::*;

use crate::ross_configurator::*;

pub fn get_programmer(protocol: &mut RossProtocol<RossSerial>) -> Result<RossProgrammerHelloEvent, RossConfiguratorError>  {
    let configurator_hello_event = RossConfiguratorHelloEvent {};

    let programmer_hello_event: RossProgrammerHelloEvent = match protocol.exchange_packet(configurator_hello_event.to_packet(), false, TRANSACTION_RETRY_COUNT as u32, || {
        sleep(Duration::from_millis(PACKET_TIMEOUT_MS))
    }) {
        Ok(event) => event,
        Err(err) => return Err(RossConfiguratorError::ProtocolError(err)),
    };

    println!("Found programmer (address: {:#06x}, firmware_version: {:#010x})", programmer_hello_event.programmer_address, programmer_hello_event.firmware_version);

    Ok(programmer_hello_event)
}
