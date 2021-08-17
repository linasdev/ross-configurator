use std::time::SystemTime;
use std::fs::File;
use std::io::Read;

use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_event::ross_programmer_event::*;
use ross_protocol::ross_event::ross_general_event::*;

use crate::ross_configurator::*;
use crate::ross_serial::RossSerial;
use crate::get_devices::get_devices;

pub fn update_firmware(serial: &mut RossSerial, firmware: &str, version: u32, address: u16) -> Result<(), RossConfiguratorError>  {
    let devices = get_devices(serial)?;

    for device in devices.iter() {
        if device.device_address == address {
            let mut file = match File::open(firmware) {
                Ok(file) => file,
                Err(err) => {
                    return Err(RossConfiguratorError::IOError(err));
                }
            };

            let mut buf = vec!();

            if let Err(err) = file.read_to_end(&mut buf) {
                return Err(RossConfiguratorError::IOError(err));
            }

            println!("Updating device's firmware (address: {:#06x}, old_firmware_version: {:#010x}, new_firmware_version: {:#010x}, firmware_size: {:#010x}).", address, device.firmware_version, version, buf.len());

            send_programmer_start_upload_event(serial, device.programmer_address, address, version, buf.len() as u32)?;

            let packet_count = (buf.len() - 1) / DATA_PACKET_SIZE + 1;

            for i in 0..packet_count {
                let slice_start = i * DATA_PACKET_SIZE;
                let slice_offset = if i == packet_count - 1 {
                    if buf.len() % DATA_PACKET_SIZE == 0 {
                        DATA_PACKET_SIZE
                    } else {
                        buf.len() % DATA_PACKET_SIZE
                    }
                } else {
                    DATA_PACKET_SIZE
                };

                println!("Sending bytes {} - {}", slice_start, slice_start + slice_offset);

                let data = &buf[slice_start..slice_start + slice_offset];

                send_data_event(serial, device.programmer_address, device.device_address, data)?;
            }

            return Ok(())
        }
    }

    Err(RossConfiguratorError::DeviceNotFound)
}

fn send_programmer_start_upload_event(serial: &mut RossSerial, programmer_address: u16, device_address: u16, new_firmware_version: u32, firmware_size: u32) -> Result<(), RossConfiguratorError> {   
    loop {
        let programmer_start_upload_event = RossProgrammerStartUploadEvent {
            programmer_address,
            device_address,
            new_firmware_version,
            firmware_size,
        };

        let packet = programmer_start_upload_event.to_packet();

        if let Err(err) = serial.try_send_packet(&packet) {
            return Err(RossConfiguratorError::SerialError(err));
        }

        let now = SystemTime::now();

        loop {
            if let Ok(packet) = serial.try_get_packet() {
                match RossAckEvent::try_from_packet(&packet) {
                    Ok(event) => {
                        if event.transmitter_address == programmer_address {
                            return Ok(());
                        }
                    },
                    Err(err) => {
                        println!("Failed to parse `ack_event` ({:?}).", err);
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

fn send_data_event(serial: &mut RossSerial, programmer_address: u16, device_address: u16, data: &[u8]) -> Result<(), RossConfiguratorError> {
    loop {
        let data_event = RossDataEvent {
            transmitter_address: programmer_address,
            receiver_address: device_address,
            data_len: data.len() as u16,
            data: data.to_vec(),
        };

        let packet = data_event.to_packet();

        if let Err(err) = serial.try_send_packet(&packet) {
            return Err(RossConfiguratorError::SerialError(err));
        }

        let now = SystemTime::now();

        loop {
            if let Ok(packet) = serial.try_get_packet() {
                match RossAckEvent::try_from_packet(&packet) {
                    Ok(event) => {
                        if event.transmitter_address == programmer_address {
                            return Ok(());
                        }
                    },
                    Err(err) => {
                        println!("Failed to parse `ack_event` ({:?}).", err);
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
