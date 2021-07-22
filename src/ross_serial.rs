use serialport::SerialPort;

use ross_protocol::ross_frame::*;
use ross_protocol::ross_packet::*;

#[derive(Debug, PartialEq)]
pub enum RossSerialError {
    ReadError,
    WriteError,
    BuilderError(RossPacketBuilderError),
    FrameError(RossFrameError),
}

pub struct RossSerial {
    port: Box<dyn SerialPort>,
    packet_builder: Option<RossPacketBuilder>,
}

impl RossSerial {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        RossSerial {
            port,
            packet_builder: None,
        }
    }

    pub fn try_get_packet(&mut self) -> Result<RossPacket, RossSerialError> {
        loop {
            let mut buf = [0x00; 1];
    
            match self.port.read_exact(&mut buf[..]) {
                Ok(_) => {
                    if buf[0] == 0x00 {
                        let expected_length = match self.port.read_exact(&mut buf[..]) {
                            Ok(_) => buf[0],
                            Err(_) => return Err(RossSerialError::ReadError),
                        };
    
                        let mut frame = vec![0x00; expected_length as usize];
    
                        if let Err(_) = self.port.read_exact(&mut frame[..]) {
                            return Err(RossSerialError::ReadError);
                        }
    
                        let ross_frame = match RossFrame::from_usart_frame(frame) {
                            Ok(frame) => frame,
                            Err(err) => {
                                return Err(RossSerialError::FrameError(err));
                            },
                        };
    
                        if let Some(ref mut packet_builder) = self.packet_builder {
                            if let Err(err) = packet_builder.add_frame(ross_frame) {
                                return Err(RossSerialError::BuilderError(err));
                            }
                        } else {
                            self.packet_builder = match RossPacketBuilder::new(ross_frame) {
                                Ok(builder) => Some(builder),
                                Err(err) => return Err(RossSerialError::BuilderError(err)),
                            };
                        }
    
                        if let Some(ref mut packet_builder) = self.packet_builder {
                            if packet_builder.frames_left() == 0 {
                                let packet = match packet_builder.build() {
                                    Ok(packet) => packet,
                                    Err(err) => return Err(RossSerialError::BuilderError(err)),
                                };
    
                                self.packet_builder = None;
    
                                return Ok(packet);
                            }
                        }
                    }
                },
                Err(_) => return Err(RossSerialError::ReadError),
            }
        }
    }

    pub fn try_send_packet(&mut self, packet: &RossPacket) -> Result<(), RossSerialError> {
        for frame in packet.to_frames().iter() {
            let frame_buf = frame.to_usart_frame();
    
            let buf = [0x00; 1];
            if let Err(_) = self.port.write(&buf) {
                return Err(RossSerialError::WriteError);
            }
    
            let buf = [frame_buf.len() as u8; 1];
            if let Err(_) = self.port.write(&buf) {
                return Err(RossSerialError::WriteError);
            }
    
            if let Err(_) = self.port.write(&frame_buf) {
                return Err(RossSerialError::WriteError);
            }
        };
    
        if let Err(_) = self.port.flush() {
            Err(RossSerialError::WriteError)
        } else {
            Ok(())
        }
    }
}
