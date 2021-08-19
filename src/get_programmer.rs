use std::time::SystemTime;

use ross_protocol::ross_interface::ross_serial::RossSerial;
use ross_protocol::ross_interface::RossInterface;
use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_event::ross_programmer_event::*;
use ross_protocol::ross_event::ross_configurator_event::*;

use crate::ross_configurator::*;

pub fn get_programmer(serial: &mut RossSerial) -> Result<RossProgrammerHelloEvent, RossConfiguratorError>  {
    let programmer_hello_event = send_configurator_hello_event(serial)?;

    println!("Found programmer (address: {:#06x}, firmware_version: {:#010x})", programmer_hello_event.programmer_address, programmer_hello_event.firmware_version);

    Ok(programmer_hello_event)
}


fn send_configurator_hello_event(serial: &mut RossSerial) -> Result<RossProgrammerHelloEvent, RossConfiguratorError> {   
    loop {
        let event = RossConfiguratorHelloEvent {};
        let packet = event.to_packet();

        if let Err(err) = serial.try_send_packet(&packet) {
            return Err(RossConfiguratorError::InterfaceError(err));
        }

        let now = SystemTime::now();

        loop {
            if let Ok(packet) = serial.try_get_packet() {
                match RossProgrammerHelloEvent::try_from_packet(&packet) {
                    Ok(event) => {
                        return Ok(event);
                    },
                    Err(err) => {
                        println!("Failed to parse `programmer_hello_event` ({:?}).", err);
                    }
                }
            }

            if now.elapsed().unwrap().as_millis() > PACKET_TIMEOUT_MS {
                break;
            }
        }
        
        if now.elapsed().unwrap().as_millis() > TRANSACTION_TIMEOUT_MS {
            return Err(RossConfiguratorError::TransactionTimedOut);
        }
    }
}
