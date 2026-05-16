#[cfg(target_os = "macos")]
use std::fs::File;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use std::os::unix::io::AsRawFd;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::*;

#[derive(Debug)]
pub enum SingleInstanceError {
    AlreadyRunning,
    Io(#[allow(dead_code)] std::io::Error),
}

pub struct SingleInstanceGuard {
    #[cfg(target_os = "macos")]
    _file: File,
    #[cfg(target_os = "macos")]
    lock_path: PathBuf,
    
    #[cfg(target_os = "windows")]
    handle: HANDLE,
}

#[cfg(target_os = "macos")]
pub fn acquire() -> Result<SingleInstanceGuard, SingleInstanceError> {
    let mut path = std::env::temp_dir();
    path.push("softveil.lock");
    
    let file = File::create(&path).map_err(SingleInstanceError::Io)?;
    let fd = file.as_raw_fd();
    
    unsafe {
        let res = libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB);
        if res != 0 {
            return Err(SingleInstanceError::AlreadyRunning);
        }
    }
    
    Ok(SingleInstanceGuard { _file: file, lock_path: path })
}

#[cfg(target_os = "windows")]
pub fn acquire() -> Result<SingleInstanceGuard, SingleInstanceError> {
    let name = "Local\\SoftveilMutex\0".encode_utf16().collect::<Vec<u16>>();
    unsafe {
        let handle = CreateMutexW(std::ptr::null(), TRUE, name.as_ptr());
        if handle.is_null() {
            return Err(SingleInstanceError::Io(std::io::Error::last_os_error()));
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(handle);
            return Err(SingleInstanceError::AlreadyRunning);
        }
        Ok(SingleInstanceGuard { handle })
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let fd = self._file.as_raw_fd();
            unsafe {
                libc::flock(fd, libc::LOCK_UN);
            }
            let _ = std::fs::remove_file(&self.lock_path);
        }
        
        #[cfg(target_os = "windows")]
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}
