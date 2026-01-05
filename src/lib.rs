use napi::bindgen_prelude::*;
use napi_derive::napi;

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{
      CloseHandle,
      HANDLE,
      HMODULE,
      GENERIC_READ,
    },
    System::{
      Threading::{
        OpenProcess,
        QueryFullProcessImageNameW,
        PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_VM_READ,
        PROCESS_NAME_WIN32,
      },
      ProcessStatus::{
        EnumProcesses,
        EnumProcessModulesEx,
        GetModuleBaseNameW,
        GetModuleFileNameExW,
        LIST_MODULES_DEFAULT,
      },
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

#[napi]
pub fn get_executable_path_from_process_name(process_name: String) -> Result<String> {
    let target_process_name = process_name.to_lowercase();
    
    // Get list of all process IDs
    let mut process_ids = vec![0u32; 1024];
    let mut bytes_needed = 0u32;
    
    let enum_result = unsafe {
        EnumProcesses(
            process_ids.as_mut_ptr(), 
            (process_ids.len() * std::mem::size_of::<u32>()) as u32,
            &mut bytes_needed
        )
    };
    
    if enum_result.is_err() {
        return Err(Error::from_reason("Failed to enumerate processes"));
    }
    
    let process_count = (bytes_needed as usize) / std::mem::size_of::<u32>();
    
    for i in 0..process_count {
        let pid = process_ids[i];
        if pid == 0 {
            continue;
        }
        
        // Try to open process with limited information first (works for most processes)
        let handle = unsafe {
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, false, pid)
        };
        
        if let Err(_) = handle {
            continue;
        }
        
        let handle = handle.unwrap();
        
        // Method 1: Try QueryFullProcessImageNameW (works on Windows Vista and later)
        let mut buffer = vec![0u16; 1024];
        let mut size = buffer.len() as u32;
        
        // Create a PWSTR from the buffer
        let buffer_ptr = buffer.as_mut_ptr();
        
        let result = unsafe {
            QueryFullProcessImageNameW(
                handle, 
                PROCESS_NAME_WIN32, 
                std::mem::transmute(buffer_ptr), 
                &mut size
            )
        };
        
        if result.is_ok() {
            let exe_path = String::from_utf16_lossy(&buffer[..size as usize]);
            let exe_name = std::path::Path::new(&exe_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            if exe_name == target_process_name {
                unsafe { CloseHandle(handle); }
                return Ok(exe_path);
            }
        }
        
        // Method 2: Fall back to GetModuleFileNameExW
        let mut modules = vec![HMODULE::default(); 1024];
        let mut bytes_needed = 0u32;
        
        let enum_result = unsafe {
            EnumProcessModulesEx(
                handle,
                modules.as_mut_ptr(),
                (modules.len() * std::mem::size_of::<HMODULE>()) as u32,
                &mut bytes_needed,
                LIST_MODULES_DEFAULT
            )
        };
        
        if enum_result.is_ok() {
            let module_count = (bytes_needed as usize) / std::mem::size_of::<HMODULE>();
            if module_count > 0 {
                let mut buffer = vec![0u16; 1024];
                
                // Get the main executable module (first module)
                // Don't pass Some(), pass the raw HMODULE
                let len = unsafe {
                    GetModuleFileNameExW(
                        handle, 
                        modules[0], 
                        &mut buffer
                    )
                };
                
                if len > 0 {
                    let exe_path = String::from_utf16_lossy(&buffer[..len as usize]);
                    let exe_name = std::path::Path::new(&exe_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    if exe_name == target_process_name {
                        unsafe { CloseHandle(handle); }
                        return Ok(exe_path);
                    }
                }
                
                // Alternative: Get base name and compare
                let mut name_buffer = vec![0u16; 256];
                let name_len = unsafe {
                    GetModuleBaseNameW(
                        handle, 
                        modules[0], 
                        &mut name_buffer
                    )
                };
                
                if name_len > 0 {
                    let current_process_name = String::from_utf16_lossy(&name_buffer[..name_len as usize])
                        .to_lowercase();
                    
                    if current_process_name == target_process_name {
                        // Get full path
                        let mut path_buffer = vec![0u16; 1024];
                        let path_len = unsafe {
                            GetModuleFileNameExW(
                                handle, 
                                modules[0], 
                                &mut path_buffer
                            )
                        };
                        
                        if path_len > 0 {
                            let exe_path = String::from_utf16_lossy(&path_buffer[..path_len as usize]);
                            unsafe { CloseHandle(handle); }
                            return Ok(exe_path);
                        }
                    }
                }
            }
        }
        
        unsafe { CloseHandle(handle); }
    }
    
    Err(Error::from_reason(format!("Process '{}' not found", process_name)))
}

#[cfg(target_os = "windows")]
#[napi]
pub fn get_executable_path(process_name: String) -> Result<String> {
    get_executable_path_from_process_name(process_name)
}

#[cfg(not(target_os = "windows"))]
#[napi]
pub fn get_executable_path(process_name: String) -> Result<String> {
    // Placeholder for other OS implementations
    Err(Error::from_reason("This function is only supported on Windows"))
}