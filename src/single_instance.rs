use std::fs::File;
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
    Io(std::io::Error),
    #[cfg(target_os = "windows")]
    WindowsError(u32),
}

pub struct SingleInstanceGuard {
    #[cfg(target_os = "macos")]
    _file: File,
    #[cfg(target_os = "macos")]
    lock_path: PathBuf,
    
    #[cfg(target_os = "windows")]
    handle: HANDLE,
}

pub fn acquire() -> Result<SingleInstanceGuard, SingleInstanceError> {
    #[cfg(target_os = "macos")]
    {
        let lock_path = std::env::temp_dir().join("softveil.lock");
        let file = File::create(&lock_path).map_err(SingleInstanceError::Io)?;
        
        unsafe {
            let fd = file.as_raw_fd();
            if libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) != 0 {
                return Err(SingleInstanceError::AlreadyRunning);
            }
        }
        
        Ok(SingleInstanceGuard { _file: file, lock_path })
    }

    #[cfg(target_os = "windows")]
    {
        unsafe {
            let name: Vec<u16> = "Local\\SoftveilMutex".encode_utf16().chain(std::iter::once(0)).collect();
            let handle = CreateMutexW(std::ptr::null(), TRUE, name.as_ptr());
            if handle == 0 {
                return Err(SingleInstanceError::WindowsError(GetLastError()));
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return Err(SingleInstanceError::AlreadyRunning);
            }
            Ok(SingleInstanceGuard { handle })
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let fd = self._file.as_raw_fd();
            libc::flock(fd, libc::LOCK_UN);
        }
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

#[cfg(target_os = "windows")]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}
