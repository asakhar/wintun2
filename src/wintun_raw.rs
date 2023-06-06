#![allow(non_snake_case, non_camel_case_types)]

use std::os::windows::raw::HANDLE;

use winapi::shared::{basetsd::DWORD64, guiddef::GUID, ifdef::NET_LUID, ntdef::LPCWSTR, minwindef::BYTE};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _WINTUN_ADAPTER {
  _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _TUN_SESSION {
  _unused: [u8; 0],
}
pub type WINTUN_ADAPTER_HANDLE = *mut _WINTUN_ADAPTER;
pub type WINTUN_SESSION_HANDLE = *mut _TUN_SESSION;
#[doc = " Determines the level of logging, passed to WINTUN_LOGGER_CALLBACK."]
pub type WINTUN_LOGGER_LEVEL = ::std::os::raw::c_int;
#[doc = "< Informational"]
pub const WINTUN_LOGGER_LEVEL_WINTUN_LOG_INFO: WINTUN_LOGGER_LEVEL = 0;
#[doc = "< Warning"]
pub const WINTUN_LOGGER_LEVEL_WINTUN_LOG_WARN: WINTUN_LOGGER_LEVEL = 1;
#[doc = "< Error"]
pub const WINTUN_LOGGER_LEVEL_WINTUN_LOG_ERR: WINTUN_LOGGER_LEVEL = 2;
pub type WINTUN_LOGGER_CALLBACK =
  Option<extern "C" fn(level: WINTUN_LOGGER_LEVEL, timestamp: DWORD64, message: LPCWSTR)>;
pub type DWORD = std::ffi::c_ulong;

#[link(name = "wintun", kind = "static")]
extern "C" {
  pub fn WintunCreateAdapter(
    name: LPCWSTR,
    tunnel_type: LPCWSTR,
    requested_guid: *const GUID,
  ) -> WINTUN_ADAPTER_HANDLE;
  pub fn WintunOpenAdapter(name: LPCWSTR) -> WINTUN_ADAPTER_HANDLE;
  pub fn WintunCloseAdapter(adapter: WINTUN_ADAPTER_HANDLE);
  pub fn WintunDeleteDriver() -> bool;
  pub fn WintunGetAdapterLUID(adapter: WINTUN_ADAPTER_HANDLE, luid: *mut NET_LUID);
  pub fn WintunGetRunningDriverVersion() -> DWORD;
  pub fn WintunSetLogger(new_logger: WINTUN_LOGGER_CALLBACK);
  pub fn WintunStartSession(adapter: WINTUN_ADAPTER_HANDLE, capacity: DWORD) -> WINTUN_SESSION_HANDLE;
  pub fn WintunEndSession(session: WINTUN_SESSION_HANDLE);
  pub fn WintunGetReadWaitEvent(session: WINTUN_SESSION_HANDLE) -> HANDLE;
  pub fn WintunReceivePacket(session: WINTUN_SESSION_HANDLE, packet_size: *mut DWORD) -> *mut BYTE;
  pub fn WintunReleaseReceivePacket(session: WINTUN_SESSION_HANDLE, packet: *const BYTE);
  pub fn WintunAllocateSendPacket(session: WINTUN_SESSION_HANDLE, packet_size: DWORD) -> *mut BYTE;
  pub fn WintunSendPacket(session: WINTUN_SESSION_HANDLE, packet: *const BYTE);
}

