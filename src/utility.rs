use get_last_error::Win32Error;
use widestring::U16CString;
use winapi::shared::{
  guiddef::GUID, ifdef::NET_LUID, netioapi::ConvertInterfaceLuidToGuid, winerror,
};

use crate::{WintunError, WintunResult};
pub(crate) fn encode_utf16(string: &str, max_characters: usize) -> WintunResult<U16CString> {
  let utf16 =
    U16CString::from_str(string).map_err(|e| WintunError::ContainsNull(e.nul_position()))?;
  if utf16.len() >= max_characters {
    //max_characters is the maximum number of characters including the null terminator. And .len() measures the
    //number of characters (excluding the null terminator). Therefore we can hold a string with
    //max_characters - 1 because the null terminator sits in the last element. However a string
    //of length max_characters needs max_characters + 1 to store the null terminator the >=
    //check holds
    Err(WintunError::TooLongName {
      max: max_characters,
      got: utf16.len(),
    })
  } else {
    Ok(utf16)
  }
}

/// A wrapper struct that allows a type to be Send and Sync
#[derive(Clone, Copy)]
pub(crate) struct UnsafeHandle<T>(pub T);

/// We never read from the pointer. It only serves as a handle we pass to the kernel or C code that
/// doesn't have the same mutable aliasing restrictions we have in Rust
unsafe impl<T> Send for UnsafeHandle<T> {}
unsafe impl<T> Sync for UnsafeHandle<T> {}

pub(crate) fn interface_luid_to_guid(luid: u64) -> Result<GUID, Win32Error> {
  let luid = NET_LUID { Value: luid };
  let luid_ptr = &luid as *const _;
  let mut guid = GUID::default();
  let guid_ptr = &mut guid as *mut _;
  let result = unsafe { ConvertInterfaceLuidToGuid(luid_ptr, guid_ptr) };
  if result != winerror::NO_ERROR {
    return Err(get_last_error::Win32Error::get_last_error());
  }
  Ok(guid)
}

pub(crate) fn guid_be_to_ne(mut guid: GUID) -> GUID {
  guid.Data1 = guid.Data1.to_be();
  guid.Data2 = guid.Data2.to_be();
  guid.Data3 = guid.Data3.to_be();
  #[cfg(target_endian = "big")]
  guid.Data4.reverse();
  guid
}

pub(crate) fn guid_from_u128(guid: u128) -> GUID {
  unsafe { std::mem::transmute(guid) }
}
pub(crate) fn guid_to_u128(guid: GUID) -> u128 {
  unsafe { std::mem::transmute(guid) }
}
pub(crate) struct GuidParseError<'data> {
  pub invalid_data: &'data [u8],
  pub error: chomp::prelude::Error<u8>
}
impl<'data> std::fmt::Debug for GuidParseError<'data> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.write_fmt(format_args!("GUID string contained invalid data: '{:?}'. Actual error: {}", self.invalid_data, self.error))
  }
}
impl<'data> std::fmt::Display for GuidParseError<'data> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.write_fmt(format_args!("{:?}", self))
  }
}
impl<'data> std::error::Error for GuidParseError<'data> {}
fn parse_chunks(src: &str) -> Result<guid_parser::Chunks, GuidParseError> {
  chomp::parse_only(guid_parser::chunks, src.as_bytes()).map_err(|(invalid_data, error)| GuidParseError{invalid_data, error})
}

/// Parse a source string as a GUID, and return the GUID as a sequence of bytes.
pub(crate) fn parse_guid(src: &str) -> Result<[u8; 16], GuidParseError> {
  parse_chunks(src).map(|chunks| chunks.to_bytes())
}
