use parse_int::parse;

use ross_protocol::protocol::Protocol;
use ross_protocol::interface::serial::Serial;
use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::general_event::*;
use ross_protocol::event::bootloader_event::*;
use ross_protocol::event::programmer_event::*;
use ross_protocol::event::bcm_event::*;

use crate::ross_configurator::*;
use crate::event_type::EventType;
use crate::event_type::EventType::*;

pub fn send_event(protocol: &mut Protocol<Serial>, event: EventType, data: Vec<&str>) -> Result<(), ConfiguratorError>  {
    let packet = match event {
        Ack => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let transmitter_address = parse_u16(data[1], "transmitter_address")?;

            AckEvent {
                receiver_address,
                transmitter_address,
            }.to_packet()
        },
        Data => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let transmitter_address = parse_u16(data[1], "transmitter_address")?;
            let data_len = parse_u16(data[2], "data_len")?;

            if data.len() != data_len as usize + 3 {
                eprintln!("Wrong amount of bytes provided.");
                return Err(ConfiguratorError::BadUsage);
            }

            let mut bytes = vec!();

            for byte_string in data[3..data.len()].iter() {
                let byte = parse_u8(byte_string, "byte")?;
                bytes.push(byte);
            }

            DataEvent {
                receiver_address,
                transmitter_address,
                data_len,
                data: bytes,
            }.to_packet()
        },
        BootloaderHello => {
            let programmer_address = parse_u16(data[0], "programmer_address")?;
            let bootloader_address = parse_u16(data[1], "bootloader_address")?;
            let firmware_version = parse_u32(data[2], "firmware_version")?;

            BootloaderHelloEvent {
                programmer_address,
                bootloader_address,
                firmware_version,
            }.to_packet()
        },
        ProgrammerHello => {
            let programmer_address = parse_u16(data[0], "programmer_address")?;
            let firmware_version = parse_u32(data[1], "firmware_version")?;

            ProgrammerHelloEvent {
                programmer_address,
                firmware_version,
            }.to_packet()
        },
        ProgrammerStartUpload => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let programmer_address = parse_u16(data[0], "programmer_address")?;
            let new_firmware_version = parse_u32(data[1], "new_firmware_version")?;
            let firmware_size = parse_u32(data[1], "firmware_size")?;

            ProgrammerStartUploadEvent {
                receiver_address,
                programmer_address,
                new_firmware_version,
                firmware_size,
            }.to_packet()
        },

        BcmChangeBrightness => {
            let bcm_address = parse_u16(data[0], "bcm_address")?;
            let channel = parse_u8(data[1], "channel")?;
            let brightness = parse_u8(data[2], "brightness")?;

            BcmChangeBrightnessEvent {
                bcm_address,
                channel,
                brightness,
            }.to_packet()
        },
    };

    protocol.add_packet_handler(Box::new(|packet, _can| {
        println!("Received packet ({:?})", packet);
    }), false).unwrap();

    match protocol.send_packet(&packet) {
        Ok(()) => {
            println!("Sent packet ({:?}).", packet);
            Ok(())
        },
        Err(err) => Err(ConfiguratorError::ProtocolError(err)),
    }
}

fn parse_u8(string: &str, name: &str) -> Result<u8, ConfiguratorError> {
    match parse::<u8>(string) {
        Ok(value) => Ok(value),
        Err(_) => {
            eprintln!("{} is not a number.", name);
            Err(ConfiguratorError::BadUsage)
        }
    }
}

fn parse_u16(string: &str, name: &str) -> Result<u16, ConfiguratorError> {
    match parse::<u16>(string) {
        Ok(value) => Ok(value),
        Err(_) => {
            eprintln!("{} is not a number.", name);
            Err(ConfiguratorError::BadUsage)
        }
    }
}

fn parse_u32(string: &str, name: &str) -> Result<u32, ConfiguratorError> {
    match parse::<u32>(string) {
        Ok(value) => Ok(value),
        Err(_) => {
            eprintln!("{} is not a number.", name);
            Err(ConfiguratorError::BadUsage)
        }
    }
}
