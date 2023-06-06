mod adapter;
mod packet;
mod session;
mod utility;
pub mod wintun_raw;

pub use adapter::*;
pub use packet::*;
pub use session::*;

use get_last_error::Win32Error;
use std::net::{Ipv4Addr, Ipv6Addr};
use widestring::WideCStr;
use winapi::shared::{basetsd::DWORD64, ntdef::LPCWSTR, winerror};

use self::wintun_raw::{
  WintunDeleteDriver, WintunGetRunningDriverVersion, WintunSetLogger, DWORD, WINTUN_LOGGER_LEVEL,
};

/// Maximum adapter name length including zero terminator
pub const MAX_ADAPTER_NAME: usize = 128;
pub const MIN_RING_CAPACITY: u32 = 0x20000;
pub const MAX_RING_CAPACITY: u32 = 0x4000000;
pub const MAX_IP_PACKET_SIZE: u32 = 0xFFFF;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GetRunningDriverVersionError {
  WintunNotLoaded,
  Other(Win32Error),
}

impl GetRunningDriverVersionError {
  pub fn is_not_loaded(self) -> bool {
    matches!(self, Self::WintunNotLoaded)
  }
}

impl std::fmt::Debug for GetRunningDriverVersionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let wte: WintunError = (*self).into();
    f.write_fmt(format_args!("{wte:?}"))
  }
}

impl std::fmt::Display for GetRunningDriverVersionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{self:?}"))
  }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReceivePacketError {
  AdapterIsTerminating,
  WouldBlock,
  InvalidData,
  Other(Win32Error),
}

impl ReceivePacketError {
  pub fn is_adapter_terminating(self) -> bool {
    matches!(self, Self::AdapterIsTerminating)
  }
  pub fn is_would_block(self) -> bool {
    matches!(self, Self::WouldBlock)
  }
  pub fn is_invalid_data(self) -> bool {
    matches!(self, Self::InvalidData)
  }
}

impl std::fmt::Debug for ReceivePacketError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let wte: WintunError = (*self).into();
    f.write_fmt(format_args!("{wte:?}"))
  }
}

impl std::fmt::Display for ReceivePacketError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{self:?}"))
  }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AllocatePacketError {
  AdapterIsTerminating,
  WouldBlock,
  Other(Win32Error),
}

impl AllocatePacketError {
  pub fn is_adapter_terminating(self) -> bool {
    matches!(self, Self::AdapterIsTerminating)
  }
  pub fn is_would_block(self) -> bool {
    matches!(self, Self::WouldBlock)
  }
}

impl std::fmt::Debug for AllocatePacketError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let wte: WintunError = (*self).into();
    f.write_fmt(format_args!("{wte:?}"))
  }
}

impl std::fmt::Display for AllocatePacketError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{self:?}"))
  }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingCapacityError {
  InvalidInput,
}

impl std::fmt::Display for RingCapacityError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Invalid ring capacity")
  }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpPacketSizeError {
  InvalidInput,
}

impl std::fmt::Display for IpPacketSizeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Invalid ip packet size")
  }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpMaskPrefixError {
  InvalidInput,
}

impl std::fmt::Display for IpMaskPrefixError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Invalid ip mask prefix. It should be in range 0..=32 or 0..=128 for Ipv4 and Ipv6 respectively")
  }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WintunError {
  TooLongName { max: usize, got: usize },
  ContainsNull(usize),
  InvalidRingCapacity,
  InvalidPacketSize,
  InvalidIpMaskPrefix,
  WintunNotLoaded,
  AdapterIsTerminating,
  WouldBlock,
  InvalidData,
  InterfaceNotFound,
  Other(Win32Error),
}

impl From<GetRunningDriverVersionError> for WintunError {
  fn from(value: GetRunningDriverVersionError) -> Self {
    match value {
      GetRunningDriverVersionError::WintunNotLoaded => Self::WintunNotLoaded,
      GetRunningDriverVersionError::Other(err) => Self::Other(err),
    }
  }
}

impl From<ReceivePacketError> for WintunError {
  fn from(value: ReceivePacketError) -> Self {
    match value {
      ReceivePacketError::AdapterIsTerminating => Self::AdapterIsTerminating,
      ReceivePacketError::InvalidData => Self::InvalidData,
      ReceivePacketError::WouldBlock => Self::WouldBlock,
      ReceivePacketError::Other(err) => Self::Other(err),
    }
  }
}

impl From<AllocatePacketError> for WintunError {
  fn from(value: AllocatePacketError) -> Self {
    match value {
      AllocatePacketError::AdapterIsTerminating => Self::AdapterIsTerminating,
      AllocatePacketError::WouldBlock => Self::WouldBlock,
      AllocatePacketError::Other(err) => Self::Other(err),
    }
  }
}

impl From<RingCapacityError> for WintunError {
  fn from(_: RingCapacityError) -> Self {
    Self::InvalidRingCapacity
  }
}

impl From<IpPacketSizeError> for WintunError {
  fn from(_: IpPacketSizeError) -> Self {
    Self::InvalidPacketSize
  }
}

impl From<IpMaskPrefixError> for WintunError {
  fn from(_: IpMaskPrefixError) -> Self {
    Self::InvalidIpMaskPrefix
  }
}

impl std::fmt::Debug for WintunError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WintunError::TooLongName { max, got } => f.write_fmt(format_args!(
        "WintunError: Too long string supplied. Max expected: {max}, received: {got}"
      )),
      WintunError::ContainsNull(pos) => f.write_fmt(format_args!(
        "WintunError: Received null byte in string at position: {pos}"
      )),
      WintunError::Other(err) => {
        f.write_fmt(format_args!("WintunError: Win32Error: {}", err.to_string()))
      }
      WintunError::InvalidRingCapacity => f.write_fmt(format_args!(
        "WintunError: Ring capacity should be in range {MIN_RING_CAPACITY}..={MAX_RING_CAPACITY} and be a power of two"
      )),
      WintunError::InvalidPacketSize => f.write_fmt(format_args!(
        "WintunError: Ip packet size should be in range 1..={MAX_IP_PACKET_SIZE}"
      )),
      WintunError::InvalidIpMaskPrefix => f.write_str(
        "WintunError: Ip mask prefix should be in range 0..=32 or 0..=128 for Ipv4 and Ipv6 respectively"
      ),
      WintunError::WintunNotLoaded => f.write_str("Wintun driver is not loaded"),
      WintunError::AdapterIsTerminating => f.write_str("Tried to perform operation on terminated adapter"),
      WintunError::WouldBlock => f.write_str("Requested operation would block"),
      WintunError::InvalidData => f.write_str("Buffer contained invalid data"),
      WintunError::InterfaceNotFound => f.write_str("Failed to find interface for specified guid"),
    }
  }
}

impl std::fmt::Display for WintunError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{self:?}"))
  }
}

impl std::error::Error for WintunError {}
impl std::error::Error for GetRunningDriverVersionError {}
impl std::error::Error for ReceivePacketError {}
impl std::error::Error for AllocatePacketError {}
impl std::error::Error for RingCapacityError {}
impl std::error::Error for IpPacketSizeError {}
impl std::error::Error for IpMaskPrefixError {}

impl From<Win32Error> for WintunError {
  fn from(value: Win32Error) -> Self {
    Self::Other(value)
  }
}

pub type WintunResult<T> = Result<T, WintunError>;

pub fn delete_driver() -> WintunResult<()> {
  let result = unsafe { WintunDeleteDriver() };
  if result {
    return Ok(());
  }
  Err(Win32Error::get_last_error().into())
}
pub fn get_running_driver_version() -> Result<DWORD, GetRunningDriverVersionError> {
  let version = unsafe { WintunGetRunningDriverVersion() };
  if version != 0 {
    return Ok(version);
  }
  let error = Win32Error::get_last_error();
  if error.code() == winerror::ERROR_FILE_NOT_FOUND {
    Err(GetRunningDriverVersionError::WintunNotLoaded)
  } else {
    Err(GetRunningDriverVersionError::Other(error))
  }
}
pub trait LoggerCallback:
  Fn(WINTUN_LOGGER_LEVEL, std::time::SystemTime, &str) + Send + Sync
{
}
static CURRENT_LOGGER: std::sync::RwLock<Option<Box<dyn LoggerCallback>>> =
  std::sync::RwLock::new(None);
extern "C" fn logger_callback_wrapper(
  level: WINTUN_LOGGER_LEVEL,
  timestamp: DWORD64,
  message: LPCWSTR,
) {
  let Ok(logger) = CURRENT_LOGGER.read() else {return;};
  let Some(logger) = logger.as_ref() else {return};

  const SECS_SINCE_1610_01_01_UNTIL_UNIX_TIMESTAMP: u64 = 131487 * 3600 * 24;
  let diff = std::time::Duration::from_micros(timestamp) / 10
    - std::time::Duration::from_secs(SECS_SINCE_1610_01_01_UNTIL_UNIX_TIMESTAMP);
  let timestamp = std::time::SystemTime::UNIX_EPOCH + diff;
  let message = unsafe { WideCStr::from_ptr_str(message) }.to_string_lossy();
  logger(level, timestamp, &message)
}
pub fn set_logger(new_logger: Option<impl LoggerCallback + 'static>) {
  let Ok(mut logger) = CURRENT_LOGGER.write() else {return;};
  if let Some(new_logger) = new_logger {
    let new_logger = Box::new(new_logger);
    logger.replace(new_logger);
    unsafe { WintunSetLogger(Some(logger_callback_wrapper)) }
  } else {
    logger.take();
    unsafe { WintunSetLogger(None) }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RingCapacity(u32);

impl TryFrom<u32> for RingCapacity {
  type Error = RingCapacityError;
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    if !(MIN_RING_CAPACITY..=MAX_RING_CAPACITY).contains(&value) || !value.is_power_of_two() {
      return Err(RingCapacityError::InvalidInput);
    }
    Ok(Self(value))
  }
}

impl RingCapacity {
  pub fn max() -> Self {
    Self(MAX_RING_CAPACITY)
  }
  pub fn min() -> Self {
    Self(MIN_RING_CAPACITY)
  }
  pub fn cap(self) -> u32 {
    self.0
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpPacketSize(u32);

impl TryFrom<u32> for IpPacketSize {
  type Error = IpPacketSizeError;
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    if !(1..=MAX_IP_PACKET_SIZE).contains(&value) {
      return Err(IpPacketSizeError::InvalidInput);
    }
    Ok(Self(value))
  }
}

impl IpPacketSize {
  pub fn max() -> Self {
    Self(MAX_IP_PACKET_SIZE)
  }
  pub fn size(self) -> u32 {
    self.0
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv4MaskPrefix(u8);

impl TryFrom<u8> for Ipv4MaskPrefix {
  type Error = IpMaskPrefixError;
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    if !(0..=32).contains(&value) {
      return Err(IpMaskPrefixError::InvalidInput);
    }
    Ok(Self(value))
  }
}

impl Ipv4MaskPrefix {
  pub fn mask(self) -> u8 {
    self.0
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv6MaskPrefix(u8);

impl TryFrom<u8> for Ipv6MaskPrefix {
  type Error = IpMaskPrefixError;
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    if !(0..=128).contains(&value) {
      return Err(IpMaskPrefixError::InvalidInput);
    }
    Ok(Self(value))
  }
}

impl Ipv6MaskPrefix {
  pub fn mask(self) -> u8 {
    self.0
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpAndMaskPrefix {
  V4 {
    ip: Ipv4Addr,
    prefix: Ipv4MaskPrefix,
  },
  V6 {
    ip: Ipv6Addr,
    prefix: Ipv6MaskPrefix,
  },
}
