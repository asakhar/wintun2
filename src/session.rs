use winapi::shared::{ntdef::HANDLE, winerror};

use crate::{
  wintun_raw::{
    WintunAllocateSendPacket, WintunEndSession, WintunGetReadWaitEvent, WintunReceivePacket,
    WintunReleaseReceivePacket, WintunSendPacket, DWORD, WINTUN_SESSION_HANDLE,
  },
  AllocatePacketError, IpPacketSize, ReceivePacketError, WintunResult,
};

use super::{
  packet::{RecvPacket, SendPacket},
  utility::UnsafeHandle,
};

pub struct Session {
  handle: UnsafeHandle<WINTUN_SESSION_HANDLE>,
}

impl Session {
  pub fn end(self) {
    drop(self)
  }
  pub fn get_read_wait_event(&self) -> WintunResult<HANDLE> {
    let event = unsafe { WintunGetReadWaitEvent(self.handle.0) };
    if event == std::ptr::null_mut() {
      return Err(get_last_error::Win32Error::get_last_error().into());
    }
    Ok(event)
  }
  pub fn recv(&self) -> Result<RecvPacket, ReceivePacketError> {
    let mut packet_size: DWORD = 0;
    let packet_raw = unsafe { WintunReceivePacket(self.handle.0, &mut packet_size as *mut _) };
    if packet_raw == std::ptr::null_mut() {
      let error = get_last_error::Win32Error::get_last_error();
      return Err(match error.code() {
        winerror::ERROR_HANDLE_EOF => ReceivePacketError::AdapterIsTerminating,
        winerror::ERROR_NO_MORE_ITEMS => ReceivePacketError::WouldBlock,
        winerror::ERROR_INVALID_DATA => ReceivePacketError::InvalidData,
        _ => ReceivePacketError::Other(error),
      });
    }
    Ok(unsafe { RecvPacket::from_raw(self, packet_raw, packet_size) })
  }
  pub fn allocate(&self, size: IpPacketSize) -> Result<SendPacket, AllocatePacketError> {
    let packet_raw = unsafe { WintunAllocateSendPacket(self.handle.0, size.size()) };
    if packet_raw == std::ptr::null_mut() {
      let error = get_last_error::Win32Error::get_last_error();
      return Err(match error.code() {
        winerror::ERROR_HANDLE_EOF => AllocatePacketError::AdapterIsTerminating,
        winerror::ERROR_BUFFER_OVERFLOW => AllocatePacketError::WouldBlock,
        _ => AllocatePacketError::Other(error),
      });
    }
    Ok(unsafe { SendPacket::from_raw(self, packet_raw, size.size()) })
  }
  pub(crate) fn send_packet(&self, packet: &mut SendPacket) {
    unsafe { WintunSendPacket(self.handle.0, packet.as_raw_ptr()) }
  }
  pub(crate) fn release_packet(&self, packet: &mut RecvPacket) {
    unsafe { WintunReleaseReceivePacket(self.handle.0, packet.as_raw_ptr()) }
  }
  pub(crate) fn new(handle: WINTUN_SESSION_HANDLE) -> Self {
    let handle = UnsafeHandle(handle);
    Self { handle }
  }
}

impl Drop for Session {
  fn drop(&mut self) {
    unsafe { WintunEndSession(self.handle.0) };
  }
}

#[cfg(test)]
mod tests {
  use std::net::Ipv4Addr;

  use crate::Adapter;

  #[test]
  fn create_session() {
    let mut adapter = Adapter::create("name", "tunnel_type", None).unwrap();
    let session = adapter.session(crate::RingCapacity::max()).unwrap();
    session.end();
  }
  #[test]
  fn send_packet() {
    let mut adapter = Adapter::create("name", "tunnel_type", None).unwrap();
    let session = adapter.session(crate::RingCapacity::max()).unwrap();
    let packet = session.allocate(crate::IpPacketSize::max()).unwrap();
    packet.send();
    session.end();
  }

  #[test]
  fn recv_packet() {
    let mut adapter = Adapter::create("name", "tunnel_type", None).unwrap();
    adapter
      .set_ip_address(crate::IpAndMaskPrefix::V4 {
        ip: Ipv4Addr::new(192, 168, 10, 1),
        prefix: 24.try_into().unwrap(),
      })
      .unwrap();
    let session = adapter.session(crate::RingCapacity::max()).unwrap();
    let packet = loop {
      match session.recv() {
        Ok(packet) => break packet,
        Err(err) if err.is_would_block() => continue,
        other => {
          other.unwrap();
        }
      }
    };
    packet.release();
    session.end();
  }
}
