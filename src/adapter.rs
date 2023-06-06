use widestring::U16CStr;
use winapi::{
  shared::{guiddef::GUID, ifdef::NET_LUID, winerror},
  um::{ipexport, iphlpapi},
};

use crate::{
  utility::{guid_be_to_ne, guid_from_u128, guid_to_u128, interface_luid_to_guid, parse_guid},
  wintun_raw::WintunStartSession,
  IpAndMaskPrefix, RingCapacity, WintunError, WintunResult, MAX_ADAPTER_NAME,
};

use super::{
  session::Session,
  utility::{encode_utf16, UnsafeHandle},
  wintun_raw::{
    WintunCloseAdapter, WintunCreateAdapter, WintunGetAdapterLUID, WintunOpenAdapter,
    WINTUN_ADAPTER_HANDLE,
  },
};

pub struct Adapter {
  handle: UnsafeHandle<WINTUN_ADAPTER_HANDLE>,
  name: String,
}

impl Adapter {
  pub fn create(
    name: impl Into<String>,
    tunnel_type: impl AsRef<str>,
    requested_guid: Option<u128>,
  ) -> WintunResult<Self> {
    let name = name.into();
    let tunnel_type = tunnel_type.as_ref();
    let name_u16 = encode_utf16(&name, MAX_ADAPTER_NAME - 1)?;
    let tunnel_type = encode_utf16(tunnel_type, MAX_ADAPTER_NAME - 1)?;
    //SAFETY: guid is a unique integer so transmuting either all zeroes or the user's preferred
    //guid to the winapi guid type is safe and will allow the windows kernel to see our GUID
    let guid_struct: Option<GUID> = requested_guid.map(|guid| unsafe { std::mem::transmute(guid) });
    //TODO: The guid of the adapter once created might differ from the one provided because of
    //the byte order of the segments of the GUID struct that are larger than a byte. Verify
    //that this works as expected

    let guid_ptr = guid_struct
      .map(|guid| &guid as *const _)
      .unwrap_or(std::ptr::null());
    let name_ptr = name_u16.as_ptr();
    let tunnel_type = tunnel_type.as_ptr();
    let handle = unsafe { WintunCreateAdapter(name_ptr, tunnel_type, guid_ptr) };
    if handle == std::ptr::null_mut() {
      return Err(get_last_error::Win32Error::get_last_error().into());
    }
    let handle = UnsafeHandle(handle);
    Ok(Self { handle, name })
  }
  pub fn open(name: impl Into<String>) -> WintunResult<Self> {
    let name = name.into();
    let name_u16 = encode_utf16(&name, MAX_ADAPTER_NAME)?;

    let handle = unsafe { WintunOpenAdapter(name_u16.as_ptr()) };
    if handle == std::ptr::null_mut() {
      return Err(get_last_error::Win32Error::get_last_error().into());
    }
    let handle = UnsafeHandle(handle);
    Ok(Self { handle, name })
  }
  pub fn close(self) {
    drop(self)
  }
  pub fn get_luid(&self) -> WintunResult<u64> {
    let mut luid = NET_LUID::default();
    unsafe { WintunGetAdapterLUID(self.handle.0, &mut luid as *mut _) }
    Ok(luid.Value)
  }

  pub fn get_guid(&self) -> WintunResult<u128> {
    let guid = interface_luid_to_guid(self.get_luid()?)?;
    Ok(unsafe { std::mem::transmute(guid) })
  }
  pub fn session(&mut self, capacity: RingCapacity) -> WintunResult<Session> {
    let capacity = capacity.cap();
    let session = unsafe { WintunStartSession(self.handle.0, capacity) };
    if session == std::ptr::null_mut() {
      return Err(get_last_error::Win32Error::get_last_error().into());
    }
    Ok(Session::new(session))
  }
  pub fn set_ip_address(&mut self, internal_ip: IpAndMaskPrefix) -> WintunResult<()> {
    let mut address_row = winapi::shared::netioapi::MIB_UNICASTIPADDRESS_ROW::default();
    unsafe {
      winapi::shared::netioapi::InitializeUnicastIpAddressEntry(&mut address_row as *mut _);
    }
    const IP_SUFFIX_ORIGIN_DHCP: winapi::shared::nldef::NL_SUFFIX_ORIGIN = 3;
    const IP_PREFIX_ORIGIN_DHCP: winapi::shared::nldef::NL_PREFIX_ORIGIN = 3;
    address_row.SuffixOrigin = IP_SUFFIX_ORIGIN_DHCP;
    address_row.PrefixOrigin = IP_PREFIX_ORIGIN_DHCP;
    const LIFETIME_INFINITE: winapi::ctypes::c_ulong = 0xffffffff;
    address_row.ValidLifetime = LIFETIME_INFINITE;
    address_row.PreferredLifetime = LIFETIME_INFINITE;
    address_row.InterfaceLuid = winapi::shared::ifdef::NET_LUID_LH {
      Value: self.get_luid()?,
    };
    match internal_ip {
      IpAndMaskPrefix::V4 { ip, prefix } => {
        unsafe {
          let ipv4 = address_row.Address.Ipv4_mut();
          ipv4.sin_family = winapi::shared::ws2def::AF_INET as _;
          *ipv4.sin_addr.S_un.S_addr_mut() = u32::from_ne_bytes(ip.octets());
        }
        address_row.OnLinkPrefixLength = prefix.mask();
      }
      IpAndMaskPrefix::V6 { ip, prefix } => {
        unsafe {
          let ipv6 = address_row.Address.Ipv6_mut();
          ipv6.sin6_family = winapi::shared::ws2def::AF_INET as _;
          *ipv6.sin6_addr.u.Byte_mut() = ip.octets();
        }
        address_row.OnLinkPrefixLength = prefix.mask();
      }
    }

    address_row.DadState = winapi::shared::nldef::IpDadStatePreferred;
    let error =
      unsafe { winapi::shared::netioapi::CreateUnicastIpAddressEntry(&mut address_row as *mut _) };
    if error != winapi::shared::winerror::ERROR_SUCCESS {
      return Err(get_last_error::Win32Error::get_last_error().into());
    }
    Ok(())
  }

  /// Returns the Win32 interface index of this adapter. Useful for specifying the interface
  /// when executing `netsh interface ip` commands
  pub fn get_adapter_index(&self) -> WintunResult<u32> {
    let guid = self.get_guid()?;
    let mut buf_len: u32 = 0;
    //First figure out the size of the buffer needed to store the adapter info
    //SAFETY: We are upholding the contract of GetInterfaceInfo. buf_len is a valid pointer to
    //stack memory
    let result =
      unsafe { iphlpapi::GetInterfaceInfo(std::ptr::null_mut(), &mut buf_len as *mut u32) };
    if result != winerror::NO_ERROR && result != winerror::ERROR_INSUFFICIENT_BUFFER {
      return Err(get_last_error::Win32Error::get_last_error().into());
    }

    //Allocate a buffer of the requested size
    //IP_INTERFACE_INFO must be aligned by at least 4 byte boundaries so use u32 as the
    //underlying data storage type
    let buf_elements = buf_len as usize / std::mem::size_of::<u32>() + 1;
    //Round up incase integer division truncated a byte that filled a partial element
    let mut buf: Vec<u32> = vec![0; buf_elements];

    let buf_bytes = buf.len() * std::mem::size_of::<u32>();
    assert!(buf_bytes >= buf_len as usize);

    //SAFETY:
    //
    //  1. We are upholding the contract of GetInterfaceInfo.
    //  2. `final_buf_len` is an aligned, valid pointer to stack memory
    //  3. buf is a valid, non-null pointer to at least `buf_len` bytes of heap memory,
    //     aligned to at least 4 byte boundaries
    //
    //Get the info
    let mut final_buf_len: u32 = buf_len;
    let result = unsafe {
      iphlpapi::GetInterfaceInfo(
        buf.as_mut_ptr() as *mut ipexport::IP_INTERFACE_INFO,
        &mut final_buf_len as *mut u32,
      )
    };
    if result != winerror::NO_ERROR {
      return Err(get_last_error::Win32Error::get_last_error().into());
    }
    let info = buf.as_mut_ptr() as *const ipexport::IP_INTERFACE_INFO;
    //SAFETY:
    // info is a valid, non-null, at least 4 byte aligned pointer obtained from
    // Vec::with_capacity that is readable for up to `buf_len` bytes which is guaranteed to be
    // larger than on IP_INTERFACE_INFO struct as the kernel would never ask for less memory then
    // what it will write. The largest type inside IP_INTERFACE_INFO is a u32 therefore
    // a painter to IP_INTERFACE_INFO requires an alignment of at leant 4 bytes, which
    // Vec<u32>::as_mut_ptr() provides
    let adapter_base = unsafe { &*info };
    let adapter_count = adapter_base.NumAdapters;
    let first_adapter = &adapter_base.Adapter as *const ipexport::IP_ADAPTER_INDEX_MAP;

    // SAFETY:
    //  1. first_adapter is a valid, non null pointer, aligned to at least 4 byte boundaries
    //     obtained from moving a multiple of 4 offset into the buf given by Vec::with_capacity.
    //  2. We gave GetInterfaceInfo a buffer of at least least `buf_len` bytes to work with and it
    //     succeeded in writing the adapter information within the bounds of that buffer, otherwise
    //     it would've failed. Because the operation succeeded, we know that reading n=NumAdapters
    //     IP_ADAPTER_INDEX_MAP structs stays within the bounds of buf's buffer
    let interfaces = unsafe { std::slice::from_raw_parts(first_adapter, adapter_count as usize) };

    for interface in interfaces {
      let name = unsafe { U16CStr::from_ptr_str(&interface.Name as *const u16).to_string_lossy() };
      //Name is something like: \DEVICE\TCPIP_{29C47F55-C7BD-433A-8BF7-408DFD3B3390}
      //where the GUID is the {29C4...90}, separated by dashes
      let Some(open) = name.chars().position(|c| c == '{') else {
        continue;
      };
      let Some(close) = name.chars().position(|c| c == '}') else {
        continue;
      };
      let Ok(digits) = parse_guid(&name[open+1..close]) else {
        continue;
      };

      let target_guid = guid_to_u128(guid_be_to_ne(guid_from_u128(u128::from_ne_bytes(digits))));
      if target_guid == guid {
        return Ok(interface.Index);
      }
    }
    Err(WintunError::InterfaceNotFound)
  }
}

impl Drop for Adapter {
  fn drop(&mut self) {
    unsafe { WintunCloseAdapter(self.handle.0) };
  }
}

pub trait TryReopen {
  fn try_reopen(&self) -> WintunResult<Self>
  where
    Self: Sized;
}

impl TryReopen for Adapter {
  fn try_reopen(&self) -> WintunResult<Self> {
    Self::open(&self.name)
  }
}

#[cfg(test)]
mod tests {
  use std::net::Ipv4Addr;

  use crate::{Adapter, TryReopen};

  #[test]
  fn create_adapter() {
    let adapter = Adapter::create("name", "tunnel_type", None).unwrap();
    adapter.get_luid().unwrap();
  }
  #[test]
  fn create_adapter_and_clone() {
    let adapter = Adapter::create("name", "tunnel_type", None).unwrap();
    let cloned = adapter.try_reopen().unwrap();
    cloned.close();
    adapter.close();
  }
  #[test]
  fn create_adapter_and_set_ip() {
    let mut adapter = Adapter::create("name", "tunnel_type", None).unwrap();
    adapter
      .set_ip_address(crate::IpAndMaskPrefix::V4 {
        ip: Ipv4Addr::new(192, 168, 10, 1),
        prefix: 24.try_into().unwrap(),
      })
      .unwrap();
    let session = adapter.session(crate::RingCapacity::max()).unwrap();
    session.end();
  }
  
  #[test]
  fn get_index() {
    let adapter = Adapter::create("name", "tunnel_type", None).unwrap();
    adapter.get_adapter_index().unwrap();
  }
}
