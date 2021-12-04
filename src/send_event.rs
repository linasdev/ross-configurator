use parse_int::parse;

use ross_protocol::convert_packet::ConvertPacket;
use ross_protocol::event::bcm::*;
use ross_protocol::event::bootloader::*;
use ross_protocol::event::button::*;
use ross_protocol::event::configurator::*;
use ross_protocol::event::general::*;
use ross_protocol::event::internal::*;
use ross_protocol::event::programmer::*;
use ross_protocol::interface::serial::Serial;
use ross_protocol::protocol::Protocol;

use crate::event_type::EventType;
use crate::event_type::EventType::*;
use crate::ross_configurator::*;

pub fn send_event(
    protocol: &mut Protocol<Serial>,
    event: EventType,
    data: Vec<&str>,
) -> Result<(), ConfiguratorError> {
    let packet = match event {
        Ack => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let transmitter_address = parse_u16(data[1], "transmitter_address")?;

            AckEvent {
                receiver_address,
                transmitter_address,
            }
            .to_packet()
        }
        Data => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let transmitter_address = parse_u16(data[1], "transmitter_address")?;
            let data_len = parse_u16(data[2], "data_len")?;

            if data.len() != data_len as usize + 3 {
                eprintln!("Wrong amount of bytes provided.");
                return Err(ConfiguratorError::BadUsage);
            }

            let mut bytes = vec![];

            for byte_string in data[3..data.len()].iter() {
                let byte = parse_u8(byte_string, "byte")?;
                bytes.push(byte);
            }

            DataEvent {
                receiver_address,
                transmitter_address,
                data_len,
                data: bytes,
            }
            .to_packet()
        }
        ConfiguratorHello => ConfiguratorHelloEvent {}.to_packet(),
        BootloaderHello => {
            let programmer_address = parse_u16(data[0], "programmer_address")?;
            let bootloader_address = parse_u16(data[1], "bootloader_address")?;

            BootloaderHelloEvent {
                programmer_address,
                bootloader_address,
            }
            .to_packet()
        }

        ProgrammerHello => {
            let programmer_address = parse_u16(data[0], "programmer_address")?;

            ProgrammerHelloEvent { programmer_address }.to_packet()
        }
        ProgrammerStartFirmwareUpgrade => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let programmer_address = parse_u16(data[1], "programmer_address")?;
            let firmware_size = parse_u32(data[2], "firmware_size")?;

            ProgrammerStartFirmwareUpgradeEvent {
                receiver_address,
                programmer_address,
                firmware_size,
            }
            .to_packet()
        }
        ProgrammerStartConfigUpgrade => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let programmer_address = parse_u16(data[0], "programmer_address")?;
            let config_size = parse_u32(data[1], "config_size")?;

            ProgrammerStartConfigUpgradeEvent {
                receiver_address,
                programmer_address,
                config_size,
            }
            .to_packet()
        }
        ButtonPressed => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let button_address = parse_u16(data[1], "button_address")?;
            let index = parse_u8(data[2], "channel")?;

            ButtonPressedEvent {
                receiver_address,
                button_address,
                index,
            }
            .to_packet()
        }
        ButtonReleased => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;
            let button_address = parse_u16(data[1], "button_address")?;
            let index = parse_u8(data[2], "channel")?;

            ButtonReleasedEvent {
                receiver_address,
                button_address,
                index,
            }
            .to_packet()
        }

        SystemTick => {
            let receiver_address = parse_u16(data[0], "receiver_address")?;

            SystemTickEvent { receiver_address }.to_packet()
        }
    };

    protocol
        .add_packet_handler(
            Box::new(|packet, _can| {
                println!("Received packet ({:?})", packet);
            }),
            false,
        )
        .unwrap();

    match protocol.send_packet(&packet) {
        Ok(()) => {
            println!("Sent packet ({:?}).", packet);
            Ok(())
        }
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
