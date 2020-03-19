//! Bhyve virtual machine operations.

use libc::{ioctl, open, O_RDWR};
use std::ffi::CString;
use std::fs::File;
use std::io::Error;
use std::os::unix::io::{AsRawFd, FromRawFd};

/// The VirtualMachine module handles KVM virtual machine operations.
/// It owns the filehandle for these operations.
pub struct VirtualMachine {
    vm: File,
    pub name: String,
}

impl VirtualMachine {
    /// Opens a filehandle to an existing virtual machine device by name, and returns a `Result`. If the open
    /// operation fails, the `Result` unwraps as an `Error`. If it succeeds, the
    /// `Result` unwraps as an instance of `VirtualMachine`.

    pub fn new(name: &str) -> Result<VirtualMachine, Error> {
        let path = format!("/dev/vmm/{}", name);
        let c_path = CString::new(path)?;
        let raw_fd = unsafe { open(c_path.as_ptr(), O_RDWR) };
        if raw_fd < 0 {
            return Err(Error::last_os_error());
        }
        let safe_handle = unsafe { File::from_raw_fd(raw_fd) };

        // Return value is safe because raw file descriptor result is checked
        // and ownership of File struct is consumed by VirtualMachine struct.
        Ok(VirtualMachine {
            vm: safe_handle,
            name: name.to_string(),
        })
    }
}
