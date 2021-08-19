use std::time::SystemTime;

use ross_protocol::ross_interface::ross_serial::RossSerial;
use ross_protocol::ross_interface::RossInterface;
use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_event::ross_programmer_event::*;
use ross_protocol::ross_event::ross_bootloader_event::*;

use crate::ross_configurator::*;
use crate::get_programmer::get_programmer;

pub fn get_devices(serial: &mut RossSerial) -> Result<Vec<RossBootloaderHelloEvent>, RossConfiguratorError>  {
    let programmer_hello_event = get_programmer(serial)?;

    let devices = send_programmer_hello_event(serial, &programmer_hello_event)?;

    for bootloader_hello_event in devices.iter() {
        println!("Found device (address: {:#06x}, firmware_version: {:#010x})", bootloader_hello_event.bootloader_address, bootloader_hello_event.firmware_version);
    }

    Ok(devices)
}

fn send_programmer_hello_event(serial: &mut RossSerial, programmer_hello_event: &RossProgrammerHelloEvent) -> Result<Vec<RossBootloaderHelloEvent>, RossConfiguratorError> {   
    loop {
        let packet = programmer_hello_event.to_packet();

        if let Err(err) = serial.try_send_packet(&packet) {
            return Err(RossConfiguratorError::InterfaceError(err));
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
        
        if now.elapsed().unwrap().as_millis() > TRANSACTION_TIMEOUT_MS {
            return Err(RossConfiguratorError::TransactionTimedOut);
        }    
    }
}
