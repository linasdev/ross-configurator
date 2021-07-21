pub const PACKET_TIMEOUT_MS: u128 = 500;
pub const DEFAULT_BAUDRATE: u32 = 9_600;

#[derive(Debug)]
pub enum RossConfiguratorError {
    BadUsage,
    FailedToOpenDevice(serialport::Error),
}
