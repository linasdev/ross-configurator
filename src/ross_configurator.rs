use std::io::Error as IOError;

use crate::ross_serial::RossSerialError;

pub const PACKET_TIMEOUT_MS: u128 = 500;
pub const DEFAULT_BAUDRATE: u32 = 115_200;
pub const DATA_PACKET_SIZE: usize = 128;

#[derive(Debug)]
pub enum RossConfiguratorError {
    BadUsage,
    DeviceNotFound,
    IOError(IOError),
    FailedToOpenDevice(serialport::Error),
    SerialError(RossSerialError),
}
