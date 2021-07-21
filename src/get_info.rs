use std::time::SystemTime;
use serialport::SerialPort;

use ross_protocol::ross_frame::RossFrame;
use ross_protocol::ross_convert_packet::RossConvertPacket;
use ross_protocol::ross_packet::{RossPacket, RossPacketBuilder};
use ross_protocol::ross_event::ross_programmer_event::*;
use ross_protocol::ross_event::ross_configurator_event::*;

use crate::ross_configurator::*;

pub fn get_info(device: &str, baudrate: u32) -> Result<(), RossConfiguratorError>  {
    let mut port = match serialport::new(device, baudrate).open() {
        Ok(port) => port,
        Err(err) => {
            eprintln!("Failed to open device.");
            return Err(RossConfiguratorError::FailedToOpenDevice(err));
        }
    };

    let programmer_hello_event = send_configurator_hello_event(&mut port);

    println!("Connected to programmer (address: {:#04x}, firmware_version: {:#04x})", programmer_hello_event.programmer_address, programmer_hello_event.programmer_address);

    Ok(())
}


fn send_configurator_hello_event(port: &mut Box<dyn SerialPort>) -> RossProgrammerHelloEvent {   
    loop {
        let event = RossConfiguratorHelloEvent {};
        let packet = event.to_packet();

        send_packet(&packet, port);

        let now = SystemTime::now();

        let mut builder = None;

        loop {
            if let Ok(packet) = try_get_packet(port, &mut builder) {
                match RossProgrammerHelloEvent::try_from_packet(&packet) {
                    Ok(event) => {
                        return event;
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

fn try_get_packet(port: &mut Box<dyn SerialPort>, builder: &mut Option<RossPacketBuilder>) -> Result<RossPacket, ()> {
    loop {
        let mut buf = [0x00; 1];

        match port.read_exact(&mut buf[..]) {
            Ok(_) => {
                if buf[0] == 0x00 {
                    let expected_length = match port.read_exact(&mut buf[..]) {
                        Ok(_) => buf[0],
                        Err(_) => return Err(()),
                    };

                    let mut frame = vec![0x00; expected_length as usize];

                    if let Err(_) = port.read_exact(&mut frame[..]) {
                        return Err(());
                    }

                    let ross_frame = match RossFrame::from_usart_frame(frame) {
                        Ok(frame) => frame,
                        Err(_) => {
                            return Err(());
                        },
                    };

                    if let Some(ref mut packet_builder) = builder {
                        if let Err(_) = packet_builder.add_frame(ross_frame) {
                            return Err(());
                        }
                    } else {
                        builder.replace(match RossPacketBuilder::new(ross_frame) {
                            Ok(builder) => builder,
                            Err(_) => return Err(()),
                        });
                    }

                    if let Some(ref mut packet_builder) = builder {
                        if packet_builder.frames_left() == 0 {
                            let packet = match packet_builder.build() {
                                Ok(packet) => packet,
                                Err(_) => return Err(()),
                            };

                            builder.take();

                            return Ok(packet);
                        }
                    }
                }
            },
            Err(_) => break,
        }
    }

    Err(())
}

fn send_packet(packet: &RossPacket, port: &mut Box<dyn SerialPort>) {
    for frame in packet.to_frames().iter() {
        let frame_buf = frame.to_usart_frame();

        let buf = [0x00; 1];
        port.write(&buf).unwrap();

        let buf = [frame_buf.len() as u8; 1];
        port.write(&buf).unwrap();

        port.write(&frame_buf).unwrap();
    };

    port.flush().unwrap();
}
