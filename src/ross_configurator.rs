use std::io::Error as IOError;
use ross_protocol::protocol::ProtocolError;
use ross_dsl::parser::ParserError;
use ross_config::config::ConfigSerializerError;

pub const PACKET_TIMEOUT_MS: u64 = 100;
pub const TRANSACTION_RETRY_COUNT: u64 = 5;
pub const DEFAULT_BAUDRATE: u64 = 115_200;
pub const DATA_PACKET_SIZE: usize = 128;

#[derive(Debug)]
pub enum ConfiguratorError {
    BadUsage,
    DeviceNotFound,
    IOError(IOError),
    FailedToOpenDevice(serialport::Error),
    ProtocolError(ProtocolError),
    ParserError(ParserError),
    ConfigSerializerError(ConfigSerializerError),
}
