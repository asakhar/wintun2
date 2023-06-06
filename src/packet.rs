use winapi::shared::minwindef::BYTE;

use crate::wintun_raw::DWORD;

use super::session::Session;

pub struct RecvPacket<'session> {
  session: &'session Session,
  data: *mut BYTE,
  size: DWORD,
}

impl<'session> RecvPacket<'session> {
  pub(crate) unsafe fn from_raw(session: &'session Session, data: *mut BYTE, size: DWORD) -> Self {
    Self {
      session,
      data,
      size,
    }
  }
  pub(crate) fn as_raw_ptr(&self) -> *const BYTE {
    self.data
  }
  pub fn slice(&self) -> &[u8] {
    unsafe { std::slice::from_raw_parts(self.data, self.size as usize) }
  }
  pub fn mut_slice(&mut self) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(self.data, self.size as usize) }
  }
  pub fn release(self) {
    drop(self)
  }
}

impl<'session> Drop for RecvPacket<'session> {
  fn drop(&mut self) {
    self.session.release_packet(self)
  }
}

impl<'session> AsRef<[u8]> for RecvPacket<'session> {
  fn as_ref(&self) -> &[u8] {
    self.slice()
  }
}

impl<'session> AsMut<[u8]> for RecvPacket<'session> {
  fn as_mut(&mut self) -> &mut [u8] {
    self.mut_slice()
  }
}


pub struct SendPacket<'session> {
  session: &'session Session,
  data: *mut BYTE,
  size: DWORD,
}

impl<'session> SendPacket<'session> {
  pub(crate) unsafe fn from_raw(session: &'session Session, data: *mut BYTE, size: DWORD) -> Self {
    Self {
      session,
      data,
      size,
    }
  }
  pub(crate) fn as_raw_ptr(&self) -> *const BYTE {
    self.data
  }
  pub fn slice(&self) -> &[u8] {
    unsafe { std::slice::from_raw_parts(self.data, self.size as usize) }
  }
  pub fn mut_slice(&mut self) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(self.data, self.size as usize) }
  }
  pub fn send(self) {
    drop(self)
  }
}

impl<'session> Drop for SendPacket<'session> {
  fn drop(&mut self) {
    self.session.send_packet(self)
  }
}

impl<'session> AsRef<[u8]> for SendPacket<'session> {
  fn as_ref(&self) -> &[u8] {
    self.slice()
  }
}

impl<'session> AsMut<[u8]> for SendPacket<'session> {
  fn as_mut(&mut self) -> &mut [u8] {
    self.mut_slice()
  }
}