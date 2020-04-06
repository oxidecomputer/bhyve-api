//! Bhyve virtual machine operations.

use libc::{ioctl, open, O_RDWR};
use std::ffi::CString;
use std::fs::File;
use std::io::Error;
use std::os::unix::io::{AsRawFd, FromRawFd};

use crate::include::vmm::{vm_suspend_how};
use crate::include::vmm_dev::*;

const MB: u64 = (1024 * 1024);
const GB: u64 = (1024 * MB);

/// The VirtualMachine module handles Bhyve virtual machine operations.
/// It owns the filehandle for these operations.
pub struct VirtualMachine {
    vm: File,
    pub name: String,
    pub lowmem_limit: u64,
    pub memflags: i32,
}

impl VirtualMachine {
    /// Opens a filehandle to an existing virtual machine device by name, and
    /// returns a `Result`. If the open  operation fails, the `Result` unwraps
    /// as an `Error`. If it succeeds, the `Result` unwraps as an instance of
    /// `VirtualMachine`.

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
            lowmem_limit: 3 * GB,
            memflags: 0,
        })
    }

    /// Sets basic attributes of CPUs on the VirtualMachine: sockets, cores,
    /// threads, and maximum number of CPUs.
    pub fn set_topology(&self, sockets: u16, cores: u16, threads: u16, maxcpus: u16) -> Result<bool, Error> {
        let top_data = vm_cpu_topology {
            sockets: sockets,
            cores: cores,
            threads: threads,
            maxcpus: maxcpus,
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SET_TOPOLOGY, &top_data) };
        println!("Attempting to set CPU topology: sockets={}, cores={}, threads={}, maxcpus={}", top_data.sockets, top_data.cores, top_data.threads, top_data.maxcpus);
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Gets current settings for CPUs on the VirtualMachine: sockets, cores,
    /// threads, and maximum number of CPUs.
    pub fn get_topology(&self) -> Result<(u16, u16, u16, u16), Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut top = vm_cpu_topology::default();
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_GET_TOPOLOGY, &mut top) };
        if result == 0 {
            return Ok((top.sockets, top.cores, top.threads, top.maxcpus));
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Gets current stats for a CPUs on the VirtualMachine.
    pub fn get_stats(&self, vcpu_id: i32) -> Result<i32, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut stats_data = vm_stats {
            cpuid: vcpu_id,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_STATS_IOC, &mut stats_data) };
        if result == 0 {
            return Ok(stats_data.num_entries);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Activates a Virtual CPU on the VirtualMachine.
    pub fn activate_vcpu(&self, vcpu_id: i32) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let cpu_data = vm_activate_cpu { vcpuid: vcpu_id };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_ACTIVATE_CPU, &cpu_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Suspends a Virtual CPU on the VirtualMachine.
    pub fn suspend_vcpu(&self, vcpu_id: i32) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let cpu_data = vm_activate_cpu { vcpuid: vcpu_id };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SUSPEND_CPU, &cpu_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Resumes a Virtual CPU on the VirtualMachine.
    pub fn resume_vcpu(&self, vcpu_id: i32) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let cpu_data = vm_activate_cpu { vcpuid: vcpu_id };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_RESUME_CPU, &cpu_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Runs the VirtualMachine, and returns an exit reason.
    pub fn run(&self, vcpu_id: i32) -> Result<VmExit, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut run_data = vm_run {
            cpuid: vcpu_id,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_RUN, &mut run_data) };
        if result == 0 {
            //let cid = run_data.cpuid;
            // println!("VCPU ID is {}", cid);
            //let exitcode = run_data.vm_exit.exitcode;
            //println!("Exit code is {}", exitcode);
            return Ok(VmExit::Bogus);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Resets the VirtualMachine.
    pub fn reset(&self) -> Result<i32, Error> {
        let suspend_data = vm_suspend { how: vm_suspend_how::VM_SUSPEND_RESET };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SUSPEND, &suspend_data) };
        if result == 0 {
            return Ok(result);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Halts the VirtualMachine.
    pub fn halt(&self) -> Result<i32, Error> {
        let suspend_data = vm_suspend { how: vm_suspend_how::VM_SUSPEND_HALT };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SUSPEND, &suspend_data) };
        if result == 0 {
            return Ok(result);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Suspends the VirtualMachine with power off.
    pub fn poweroff(&self) -> Result<i32, Error> {
        let suspend_data = vm_suspend { how: vm_suspend_how::VM_SUSPEND_POWEROFF };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SUSPEND, &suspend_data) };
        if result == 0 {
            return Ok(result);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Suspends the VirtualMachine with triple fault.
    pub fn triplefault(&self) -> Result<i32, Error> {
        let suspend_data = vm_suspend { how: vm_suspend_how::VM_SUSPEND_TRIPLEFAULT };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SUSPEND, &suspend_data) };
        if result == 0 {
            return Ok(result);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Reinitializes the VirtualMachine.
    pub fn reinit(&self) -> Result<i32, Error> {
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_REINIT) };
        if result == 0 {
            return Ok(result);
        } else {
            return Err(Error::last_os_error());
        }
    }
}

/// Reasons for virtual machine exits.
///
/// The exit reasons are mapped to the `VM_EXIT_*` defines in `machine/vmm.h`.
///
#[derive(Debug, Copy, Clone)]
pub enum VmExit {
    InOut,
    Vmx,
    Bogus,
    RdMsr,
    WrMsr,
    Hlt,
    Mtrap,
    Pause,
    Paging,
    InstEmul,
    SpinupAp,
    Deprecated1,
    RunBlock,
}
