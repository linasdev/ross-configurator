use std::time::SystemTime;

use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_event::ross_programmer_event::*;
use ross_protocol::ross_event::ross_bootloader_event::*;

use crate::ross_configurator::*;
use crate::ross_serial::RossSerial;
use crate::get_programmer::get_programmer;

pub fn get_devices(serial: &mut RossSerial) -> Result<(), RossConfiguratorError>  {
    let programmer_hello_event = get_programmer(serial)?;

    for bootloader_hello_event in send_programmer_hello_event(serial, &programmer_hello_event)?.iter() {
        println!("Connected to device (address: {:#04x}, firmware_version: {:#04x})", bootloader_hello_event.device_address, bootloader_hello_event.firmware_version);
    }

    Ok(())
}

fn send_programmer_hello_event(serial: &mut RossSerial, programmer_hello_event: &RossProgrammerHelloEvent) -> Result<Vec<RossBootloaderHelloEvent>, RossConfiguratorError> {   
    loop {
        let packet = programmer_hello_event.to_packet();

        if let Err(err) = serial.try_send_packet(&packet) {
            return Err(RossConfiguratorError::SerialError(err));
        }

        let now = SystemTime::now();

        loop {
            if let Ok(packet) = serial.try_get_packet() {
                match RossBootloaderHelloEvent::try_from_packet(&packet) {
                    Ok(event) => {
                        return Ok(vec!(event));
                    },
                    Err(err) => {
                        println!("Failed to parse `bootloader_hello_event` ({:?}).", err);
                    }
                }
            }

            if now.elapsed().unwrap().as_millis() > PACKET_TIMEOUT_MS {
                break;
            }
        }       
    }
}
