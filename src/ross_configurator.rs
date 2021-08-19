use std::io::Error as IOError;
use ross_protocol::ross_interface::RossInterfaceError;

pub const TRANSACTION_TIMEOUT_MS: u128 = 500;
pub const PACKET_TIMEOUT_MS: u128 = 100;
pub const DEFAULT_BAUDRATE: u32 = 115_200;
pub const DATA_PACKET_SIZE: usize = 128;

#[derive(Debug)]
pub enum RossConfiguratorError {
    BadUsage,
    DeviceNotFound,
    TransactionTimedOut,
    IOError(IOError),
    FailedToOpenDevice(serialport::Error),
    InterfaceError(RossInterfaceError),
}
