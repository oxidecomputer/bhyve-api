// Copyright (C) 2020, Oxide Computer Company

use libc::{ioctl, open, O_EXCL, O_RDWR};
use std::ffi::CString;
use std::fs::File;
use std::io::Error;
use std::os::unix::io::{AsRawFd, FromRawFd};

use crate::include::vmm_dev::{VMM_CREATE_VM, VMM_DESTROY_VM};

/// The VMMSystem module handles VMM system operations. It creates and
/// owns the initial filehandle on `/dev/vmmctl`.
///
///     use bhyve_api::system::*;
///     let system = VMMSystem::new().expect("failed to connect to VMM system ioctl handle");
///     let vm = system.create_vm("uniquename").expect("failed to create VM");
///     system.destroy_vm("uniquename").expect("failed to destroy VM");

pub struct VMMSystem {
    vmmctl: File,
}

impl VMMSystem {
    /// Opens a filehandle to `/dev/vmmctl`, and returns a `Result`. If the open
    /// operation fails, the `Result` unwraps as an `Error`. If it succeeds, the
    /// `Result` unwraps as an instance of `VMMSystem` for performing VMM system
    /// operations.

    pub fn new() -> Result<VMMSystem, Error> {
        let path = CString::new("/dev/vmmctl")?;
        let raw_fd = unsafe { open(path.as_ptr(), O_RDWR | O_EXCL) };
        if raw_fd < 0 {
            return Err(Error::last_os_error());
        }
        let safe_handle = unsafe { File::from_raw_fd(raw_fd) };

        // Return value is safe because raw file descriptor result is checked
        // and ownership of File struct is consumed by KVMSystem struct.
        Ok(VMMSystem {
            vmmctl: safe_handle,
        })
    }

    /// Creates a device for virtual machine operation at `/dev/vmm/[name]`,
    /// and returns a `Result`. If the creation operation fails, the `Result`
    /// unwraps as an `Error`. If it succeeds, the `Result` unwraps as `i32`
    /// integer containing the integer return value of the ioctl operation.

    pub fn create_vm(&self, name: &str) -> Result<i32, Error> {
        let c_name = CString::new(name)?;
        let result = unsafe { ioctl(self.vmmctl.as_raw_fd(), VMM_CREATE_VM, c_name.as_ptr()) };
        if result == -1 {
            return Err(Error::last_os_error());
        } else {
            return Ok(result);
        }
    }

    /// Destroys a device for virtual machine operations at `/dev/vmm/[name]`,
    /// and returns a `Result`. If the destruction operation fails, the `Result`
    /// unwraps as an `Error`. If it succeeds, the `Result` unwraps as `i32`
    /// integer containing the integer return value of the ioctl operation.

    pub fn destroy_vm(&self, name: &str) -> Result<i32, Error> {
        let c_name = CString::new(name)?;
        let result = unsafe { ioctl(self.vmmctl.as_raw_fd(), VMM_DESTROY_VM, c_name.as_ptr()) };
        if result == -1 {
            return Err(Error::last_os_error());
        } else {
            return Ok(result);
        }
    }
}
