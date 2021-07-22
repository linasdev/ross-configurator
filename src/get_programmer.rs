use std::time::SystemTime;

use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_event::ross_programmer_event::*;
use ross_protocol::ross_event::ross_configurator_event::*;

use crate::ross_configurator::*;
use crate::ross_serial::RossSerial;

pub fn get_programmer(mut serial: RossSerial) -> Result<(), RossConfiguratorError>  {
    let programmer_hello_event = send_configurator_hello_event(&mut serial)?;

    println!("Connected to programmer (address: {:#04x}, firmware_version: {:#04x})", programmer_hello_event.programmer_address, programmer_hello_event.programmer_address);

    Ok(())
}


fn send_configurator_hello_event(serial: &mut RossSerial) -> Result<RossProgrammerHelloEvent, RossConfiguratorError> {   
    loop {
        let event = RossConfiguratorHelloEvent {};
        let packet = event.to_packet();

        if let Err(err) = serial.try_send_packet(&packet) {
            return Err(RossConfiguratorError::SerialError(err));
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
    }
}
