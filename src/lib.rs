use napi::bindgen_prelude::*;
use napi_derive::napi;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{
      CloseHandle,
      HANDLE,
      GENERIC_READ,
    },
    Storage::FileSystem::{
      CreateFileW,
      GetFinalPathNameByHandleW,
      FILE_ATTRIBUTE_NORMAL,
      FILE_SHARE_READ,
      FILE_SHARE_WRITE,
      OPEN_EXISTING,
      GETFINALPATHNAMEBYHANDLE_FLAGS,
    },
  },
};

#[napi]
pub fn get_final_path_name_by_handle(path: String) -> Result<String> {
  // UTF-16 + NUL terminator
  let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();

  // CreateFileW now returns Result<HANDLE>
  let handle: HANDLE = unsafe {
    CreateFileW(
      PCWSTR(wide.as_ptr()),
      GENERIC_READ.0, // ← convert to u32
      FILE_SHARE_READ | FILE_SHARE_WRITE,
      None,
      OPEN_EXISTING,
      FILE_ATTRIBUTE_NORMAL,
      None,
    )
  }
  .map_err(|e| Error::from_reason(format!("CreateFileW failed: {e}")))?;

  let mut buffer = vec![0u16; 1024];

  let len = unsafe {
    GetFinalPathNameByHandleW(
      handle,
      &mut buffer,
      GETFINALPATHNAMEBYHANDLE_FLAGS(0), // no flags (MT5 behavior)
    )
  };

  unsafe {
    CloseHandle(handle);
  }

  if len == 0 {
    return Err(Error::from_reason(
      "GetFinalPathNameByHandleW failed",
    ));
  }

  Ok(String::from_utf16_lossy(&buffer[..len as usize]))
}
