//! Bhyve virtual machine operations.

use libc::{ioctl, open, O_RDWR, c_void, sysconf, _SC_PAGESIZE};
use std::ffi::{CString, CStr};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::os::unix::io::{AsRawFd, FromRawFd};

pub use crate::include::vmm::{vm_cap_type, vm_reg_name};
use crate::include::vmm::{vm_suspend_how, vm_exitcode, x2apic_state, seg_desc};
use crate::include::vmm_dev::*;
use crate::include::specialreg::{CR0_NE};

const MB: u64 = (1024 * 1024);
const GB: u64 = (1024 * MB);

const MAX_BOOTROM_SIZE: usize = 16 * MB as usize;

// Size of the guard region before and after the virtual address space
// mapping the guest physical memory. This must be a multiple of the
// superpage size for performance reasons.
//const VM_MMAP_GUARD_SIZE: usize = 4 * MB as usize;

/// The VirtualMachine module handles Bhyve virtual machine operations.
/// It owns the filehandle for these operations.
pub struct VirtualMachine {
    vm: File,
    pub name: String,
    pub lowmem_limit: usize,
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
            lowmem_limit: 3 * GB as usize,
            memflags: 0,
        })
    }

    /// Map the memory segment identified by 'segid' into the guest address space
    /// at [gpa,gpa+len) with protection 'prot'.
    pub fn mmap_memseg(&self, gpa: u64, segid: MemSegId, off: i64, len: usize, prot: i32) -> Result<bool, Error> {
        let mut flags = 0;
        if (self.memflags & VM_MEM_F_WIRED) != 0 {
            flags = VM_MEMMAP_F_WIRED;
        }

        let mem_data = vm_memmap {
            gpa: gpa,
            segid: segid as i32,
            segoff: off,
            len: len,
            prot: prot,
            flags: flags,
        };

	// If this mapping already exists then don't create it again. This
	// is the common case for SYSMEM mappings created by bhyveload(8).
        match self.mmap_getnext(gpa) {
            Ok(exists) => if exists.gpa == mem_data.gpa {
                // A memory segment already exists at the same guest physical address
                // we are trying to create.
                if exists.segid == mem_data.segid && exists.segoff == mem_data.segoff &&
                   exists.prot == mem_data.prot && exists.flags == mem_data.flags {
                    // The existing memory segment is identical to the one we want to
                    // create, so do nothing, and return a success value.
                    return Ok(true);
                } else {
                    // The existing memory segment is not identical to the one we want
                    // to create, so return an error value.
                    return Err(Error::from(ErrorKind::AlreadyExists));
                }
            }
            Err(_) => (), // The memory segment wasn't found, so we should create it
        };

        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_MMAP_MEMSEG, &mem_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Iterate over the guest address space. This function finds an address range
    /// that starts at an address >= 'gpa'.
    ///
    /// Returns Ok if the next address range was found and an Error otherwise.

    fn mmap_getnext(&self, gpa: u64) -> Result<vm_memmap, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut memseg_data = vm_memmap {
            gpa: gpa,
            ..Default::default()
        };

        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_MMAP_GETNEXT, &mut memseg_data) };
        if result == 0 {
            return Ok(memseg_data);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Unmap the memory segment at the guest physical address range [gpa,gpa+len)
    pub fn munmap_memseg(&self, gpa: u64, len: usize) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let mem_data = vm_munmap {
            gpa: gpa,
            len: len,
        };

        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_MUNMAP_MEMSEG, &mem_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    pub fn alloc_memseg(&self, segid: MemSegId, len: usize, name: &str) -> Result<bool, Error> {
        let c_name = CString::new(name)?;

        // If the memory segment has already been created then just return.
        // This is the usual case for the SYSMEM segment created by userspace
        // loaders like bhyveload(8).
        match self.get_memseg(segid) {
            Ok(exists) => if exists.len != 0 {
                // A memory segment already exists with the same segment ID as the one
                // we are trying to allocate.
                let r_name = unsafe { CStr::from_ptr(exists.name.as_ptr()) };
                let exists_name = r_name.to_owned();
                if exists.len == len && exists_name == c_name {
                    // The existing memory segment is identical to the one we want to
                    // allocate, so do nothing, and return a success value.
                    return Ok(true);
                } else {
                    // The existing memory segment is not identical to the one we want
                    // to allocate, so return an error value.
                    return Err(Error::from(ErrorKind::InvalidInput));
                }
            }
            Err(e) => return Err(e),
        };

        // Struct is allocated (and owned) by Rust
        let mut memseg_data = vm_memseg {
            segid: segid as i32,
            len: len,
            ..Default::default()
        };

        let name_length = name.len();
        if name_length > 0 {
            // Don't copy the name if the string is empty (zero length)
            if name_length >= memseg_data.name.len() {
                // name is too long for vm_memseg struct
                return Err(Error::from(ErrorKind::InvalidInput));
            } else {
                // Copy each character from the CString to the char array
                for (to, from) in memseg_data.name.iter_mut().zip(c_name.as_bytes_with_nul()) {
                    *to = *from as i8;
                }
            }
        }

        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_ALLOC_MEMSEG, &memseg_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    fn get_memseg(&self, segid: MemSegId) -> Result<vm_memseg, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut memseg_data = vm_memseg {
            segid: segid as i32,
            ..Default::default()
        };

        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_GET_MEMSEG, &mut memseg_data) };
        if result == 0 {
            return Ok(memseg_data);
        } else {
            return Err(Error::last_os_error());
        }
    }

    fn add_devmem(&self, segid: MemSegId, name: &str, base: u64, len: usize) -> Result<bool, Error> {
        self.alloc_memseg(segid, len, name)?;
        let mapoff = self.get_devmem_offset(segid)?;

//        let len2 = VM_MMAP_GUARD_SIZE + len + VM_MMAP_GUARD_SIZE;
//        let base: *mut u8 = unsafe {
//            libc::mmap(
//                null_mut(),
//                len2,
//                libc::PROT_NONE,
//                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
//                -1,
//                0,
//            ) as *mut u8
//        };

        // mmap the devmem region in the host address space
        let _ptr: *mut u8 = unsafe {
            libc::mmap(
                base as *mut c_void,
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_FIXED,
                self.vm.as_raw_fd(),
                mapoff,
            ) as *mut u8
        };
        return Ok(true);

    }

    fn add_guest_memory(&self, segid: MemSegId, gpa: u64, base: u64, len: usize) -> Result<bool, Error> {
        self.alloc_memseg(segid, len, "")?; // only devices name their memory regions

        // Map the guest memory into the guest address space
	let prot = libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC;
	self.mmap_memseg(gpa, MemSegId::VM_LOWMEM, 0, len, prot)?;

        // mmap into the process address space on the host
        let ptr = unsafe {
            libc::mmap(
                base as *mut c_void,
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_FIXED,
                self.vm.as_raw_fd(),
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(Error::from(ErrorKind::AddrNotAvailable));
        }

        return Ok(true);

    }

    /// Gets the map offset for the device memory segment 'segid'.
    ///
    /// Returns Ok containing the offset if successful, and an Error otherwise.
    fn get_devmem_offset(&self, segid: MemSegId) -> Result<i64, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut memseg_data = vm_devmem_offset {
            segid: segid as i32,
            ..Default::default()
        };

        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_DEVMEM_GETOFFSET, &mut memseg_data) };
        if result == 0 {
            return Ok(memseg_data.offset);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Sets up a memory segment for the bootrom
    ///
    /// Returns Ok if successful, and an Error otherwise.
    pub fn setup_bootrom(&self, base: u64, len: usize) -> Result<bool, Error> {

        let page_size: usize = unsafe { sysconf(_SC_PAGESIZE) as usize };
        // Limit bootrom size to 16MB so it doesn't encroach into reserved
        // MMIO space (e.g. APIC, HPET, MSI).
        if len > MAX_BOOTROM_SIZE || len < page_size {
            return Err(Error::from(ErrorKind::InvalidInput));
        }
        // Map the bootrom into the host address space
        self.add_devmem(MemSegId::VM_BOOTROM, "bootrom", base, len)?;

        // Map the bootrom into the guest address space
	let prot = libc::PROT_READ | libc::PROT_EXEC;
	let gpa: u64 = (1 << 32) - len as u64;
	self.mmap_memseg(gpa, MemSegId::VM_BOOTROM, 0, len, prot)?;

        Ok(true)
    }

    pub fn setup_lowmem(&self, base: u64, len: usize) -> Result<bool, Error> {
        if len > self.lowmem_limit {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

	let gpa: u64 = 0;
        // Map the guest memory into the host address space
        self.add_guest_memory(MemSegId::VM_LOWMEM, gpa, base, len)?;

        Ok(true)
    }

    pub fn setup_highmem(&self, base: u64, len: usize) -> Result<bool, Error> {
	let gpa: u64 = 4 * GB;
        // Map the guest memory into the host address space
        self.add_guest_memory(MemSegId::VM_HIGHMEM, gpa, base, len)?;

        Ok(true)
    }

    /// Set the base, limit, and access values of a descriptor register on the VCPU
    pub fn set_desc(&self, vcpu_id: i32, reg: vm_reg_name, base: u64, limit: u32, access: u32) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let seg_data = vm_seg_desc {
            cpuid: vcpu_id,
            regnum: reg as i32,
            desc: seg_desc {base: base, limit: limit, access: access},
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SET_SEGMENT_DESCRIPTOR, &seg_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Get the base, limit, and access values of a descriptor register on the VCPU
    pub fn get_desc(&self, vcpu_id: i32, reg: vm_reg_name) -> Result<(u64, u32, u32), Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut seg_data = vm_seg_desc {
            cpuid: vcpu_id,
            regnum: reg as i32,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_GET_SEGMENT_DESCRIPTOR, &mut seg_data) };
        if result == 0 {
            return Ok((seg_data.desc.base, seg_data.desc.limit, seg_data.desc.access));
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Set the value of a single register on the VCPU
    pub fn set_register(&self, vcpu_id: i32, reg: vm_reg_name, val: u64) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let reg_data = vm_register {
            cpuid: vcpu_id,
            regnum: reg as i32,
            regval: val,
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SET_REGISTER, &reg_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Get the value of a single register on the VCPU
    pub fn get_register(&self, vcpu_id: i32, reg: vm_reg_name) -> Result<u64, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut reg_data = vm_register {
            cpuid: vcpu_id,
            regnum: reg as i32,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_GET_REGISTER, &mut reg_data) };
        if result == 0 {
            return Ok(reg_data.regval);
        } else {
            return Err(Error::last_os_error());
        }
    }

    pub fn rtc_write(&self, offset: i32, value: u8) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let rtc_data = vm_rtc_data {
            offset: offset,
            value: value,
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_RTC_WRITE, &rtc_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    pub fn rtc_read(&self, offset: i32) -> Result<u8, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut rtc_data = vm_rtc_data {
            offset: offset,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_RTC_READ, &mut rtc_data) };
        if result == 0 {
            return Ok(rtc_data.value);
        } else {
            return Err(Error::last_os_error());
        }
    }

    pub fn rtc_settime(&self, secs: i64) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let rtc_data = vm_rtc_time {
            secs: secs,
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_RTC_SETTIME, &rtc_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    pub fn rtc_gettime(&self) -> Result<i64, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut rtc_data = vm_rtc_time::default();
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_RTC_GETTIME, &mut rtc_data) };
        if result == 0 {
            return Ok(rtc_data.secs);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Sets basic attributes of CPUs on the VirtualMachine: sockets, cores,
    /// and threads.
    pub fn set_topology(&self, sockets: u16, cores: u16, threads: u16) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let top_data = vm_cpu_topology {
            sockets: sockets,
            cores: cores,
            threads: threads,
            maxcpus: 0, // any other value is invalid
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SET_TOPOLOGY, &top_data) };
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

    pub fn set_x2apic_state(&self, vcpu_id: i32, enable: bool) -> Result<bool, Error> {
        let state = match enable {
            true => x2apic_state::X2APIC_ENABLED,
            false => x2apic_state::X2APIC_DISABLED
        };

        // Struct is allocated (and owned) by Rust
        let x2apic_data = vm_x2apic {
            cpuid: vcpu_id,
            state: state,
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SET_X2APIC_STATE, &x2apic_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }

    pub fn get_x2apic_state(&self, vcpu_id: i32) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut x2apic_data = vm_x2apic {
            cpuid: vcpu_id,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_GET_X2APIC_STATE, &mut x2apic_data) };
        if result == 0 {
            match x2apic_data.state {
                x2apic_state::X2APIC_ENABLED => return Ok(true),
                x2apic_state::X2APIC_DISABLED => return Ok(false),
                x2apic_state::X2APIC_STATE_LAST => return Err(Error::from(ErrorKind::InvalidData)),
            }
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// From Intel Vol 3a:
    /// Table 9-1. IA-32 Processor States Following Power-up, Reset or INIT
    pub fn vcpu_reset(&self, vcpu_id: i32) -> Result<bool, Error> {
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RFLAGS, 0x2)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RIP, 0xfff0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_CR0, CR0_NE)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_CR3, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_CR4, 0)?;

        // CS: present, r/w, accessed, 16-bit, byte granularity, usable
	let cs_base = 0xffff0000;
	let cs_limit = 0xffff;
	let cs_access = 0x0093;
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_CS, cs_base, cs_limit, cs_access)?;

        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_CS, 0xf000)?;


        // SS,DS,ES,FS,GS: present, r/w, accessed, 16-bit, byte granularity
	let desc_base = 0;
	let desc_limit = 0xffff;
	let desc_access = 0x0093;
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_SS, desc_base, desc_limit, desc_access)?;
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_DS, desc_base, desc_limit, desc_access)?;
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_ES, desc_base, desc_limit, desc_access)?;
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_FS, desc_base, desc_limit, desc_access)?;
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_GS, desc_base, desc_limit, desc_access)?;

        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_SS, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_DS, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_ES, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_FS, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_GS, 0)?;

        // General purpose registers
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RAX, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RBX, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RCX, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RDX, 0xf00)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RSI, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RDI, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RBP, 0)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_RSP, 0)?;


        // GDTR, IDTR
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_GDTR, 0, 0xffff, 0)?;
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_IDTR, 0, 0xffff, 0)?;

        // TR
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_TR, 0, 0, 0x0000008b)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_TR, 0)?;

        // LDTR
        self.set_desc(vcpu_id, vm_reg_name::VM_REG_GUEST_LDTR, 0, 0xffff, 0x00000082)?;
        self.set_register(vcpu_id, vm_reg_name::VM_REG_GUEST_LDTR, 0)?;

        Ok(true)
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
            let rip = run_data.vm_exit.rip;
            println!("RIP after run is {}", rip);
            let cid = run_data.cpuid;
            println!("VCPU ID is {}", cid);
            match run_data.vm_exit.exitcode {
                vm_exitcode::VM_EXITCODE_INOUT => {
                    let io = unsafe { run_data.vm_exit.u.inout };
                    let port = io.port;
                    let eax = io.eax;
                    println!("bitfield bytes is {}", io.bytes());
                    println!("bitfield in is {}", io.is_in());
                    println!("bitfield string is {}", io.is_string());
                    println!("bitfield rep is {}", io.is_rep());
                    return Ok(VmExit::InOut(port, eax));
                }
                vm_exitcode::VM_EXITCODE_VMX => {
                    let status = unsafe { run_data.vm_exit.u.vmx.status };
                    let reason = unsafe { run_data.vm_exit.u.vmx.exit_reason };
                    let qual = unsafe { run_data.vm_exit.u.vmx.exit_qualification };
                    let inst_type = unsafe { run_data.vm_exit.u.vmx.inst_type };
                    let inst_error = unsafe { run_data.vm_exit.u.vmx.inst_error };
                    return Ok(VmExit::Vmx(status, reason, qual, inst_type, inst_error));
                }
                vm_exitcode::VM_EXITCODE_BOGUS => {
                    return Ok(VmExit::Bogus);
                }
                vm_exitcode::VM_EXITCODE_RDMSR => {
                    return Ok(VmExit::RdMsr);
                }
                vm_exitcode::VM_EXITCODE_WRMSR => {
                    return Ok(VmExit::WrMsr);
                }
                vm_exitcode::VM_EXITCODE_HLT => {
                    return Ok(VmExit::Halt);
                }
                vm_exitcode::VM_EXITCODE_MTRAP => {
                    return Ok(VmExit::Mtrap);
                }
                vm_exitcode::VM_EXITCODE_PAUSE => {
                    return Ok(VmExit::Pause);
                }
                vm_exitcode::VM_EXITCODE_PAGING => {
                    return Ok(VmExit::Paging);
                }
                vm_exitcode::VM_EXITCODE_INST_EMUL => {
                    return Ok(VmExit::InstEmul);
                }
                vm_exitcode::VM_EXITCODE_SPINUP_AP => {
                    return Ok(VmExit::SpinupAp);
                }
                vm_exitcode::VM_EXITCODE_DEPRECATED1 => {
                    return Ok(VmExit::Deprecated);
                }
                vm_exitcode::VM_EXITCODE_RUNBLOCK => {
                    return Ok(VmExit::RunBlock);
                }
                vm_exitcode::VM_EXITCODE_IOAPIC_EOI => {
                    return Ok(VmExit::IoApicEoi);
                }
                vm_exitcode::VM_EXITCODE_SUSPENDED => {
                    return Ok(VmExit::Suspended);
                }
                vm_exitcode::VM_EXITCODE_INOUT_STR => {
                    let io = unsafe { run_data.vm_exit.u.inout_str.inout };
                    let port = io.port;
                    let eax = io.eax;
                    println!("bitfield bytes is {}", io.bytes());
                    println!("bitfield in is {}", io.is_in());
                    println!("bitfield string is {}", io.is_string());
                    println!("bitfield rep is {}", io.is_rep());
                    return Ok(VmExit::InOutStr(port, eax));
                }
                vm_exitcode::VM_EXITCODE_TASK_SWITCH => {
                    return Ok(VmExit::TaskSwitch);
                }
                vm_exitcode::VM_EXITCODE_MONITOR => {
                    return Ok(VmExit::Monitor);
                }
                vm_exitcode::VM_EXITCODE_MWAIT => {
                    return Ok(VmExit::Mwait);
                }
                vm_exitcode::VM_EXITCODE_SVM => {
                    return Ok(VmExit::Svm);
                }
                vm_exitcode::VM_EXITCODE_REQIDLE => {
                    return Ok(VmExit::ReqIdle);
                }
                vm_exitcode::VM_EXITCODE_DEBUG => {
                    return Ok(VmExit::Debug);
                }
                vm_exitcode::VM_EXITCODE_VMINSN => {
                    return Ok(VmExit::VmInsn);
                }
                vm_exitcode::VM_EXITCODE_HT => {
                    return Ok(VmExit::Ht);
                }
                vm_exitcode::VM_EXITCODE_MAX => {
                    return Ok(VmExit::Max);
                }
            }
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

    /// Get the value of an optional capability on the VCPU
    pub fn get_capability(&self, vcpu_id: i32, cap: vm_cap_type) -> Result<i32, Error> {
        // Struct is allocated (and owned) by Rust, but modified by C
        let mut cap_data = vm_capability {
            cpuid: vcpu_id,
            captype: cap,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_GET_CAPABILITY, &mut cap_data) };
        if result == 0 {
            return Ok(cap_data.capval);
        } else {
            return Err(Error::last_os_error());
        }
    }

    /// Set the value of an optional capability on the VCPU
    pub fn set_capability(&self, vcpu_id: i32, cap: vm_cap_type, val: i32) -> Result<bool, Error> {
        // Struct is allocated (and owned) by Rust
        let cap_data = vm_capability {
            cpuid: vcpu_id,
            captype: cap,
            capval: val,
            ..Default::default()
        };
        let result = unsafe { ioctl(self.vm.as_raw_fd(), VM_SET_CAPABILITY, &cap_data) };
        if result == 0 {
            return Ok(true);
        } else {
            return Err(Error::last_os_error());
        }
    }
}

// Different styles of mapping the memory assigned to a VM into the address
// space of the controlling process.
#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Debug, Copy, Clone)]
enum vm_mmap_style {
	VM_MMAP_NONE,		/* no mapping */
	VM_MMAP_ALL,		/* fully and statically mapped */
	VM_MMAP_SPARSE,		/* mappings created on-demand */
}

// 'flags' value passed to 'vm_set_memflags()'.
//const VM_MEM_F_INCORE: i32 = 0x01;    // include guest memory in core file
const VM_MEM_F_WIRED: i32 = 0x02;	// guest memory is wired

/// Identifiers for memory segments, both system memory and devmem segments.
#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Debug, Copy, Clone)]
pub enum MemSegId{
        VM_LOWMEM = 0,
        VM_HIGHMEM = 1,
        VM_BOOTROM = 2,
        VM_FRAMEBUFFER = 3,
}

/// Reasons for virtual machine exits.
///
/// The exit reasons are mapped to the `VM_EXIT_*` defines in `machine/vmm.h`.
///
#[derive(Debug, Copy, Clone)]
pub enum VmExit {
    InOut(u16 /* port */, u32 /* eax */),
    Vmx(i32 /* status */, u32 /* exit reason */, u64 /* exit qualification */, i32 /* instruction type */, i32 /* instruction error */),
    Bogus,
    RdMsr,
    WrMsr,
    Halt,
    Mtrap,
    Pause,
    Paging,
    InstEmul,
    SpinupAp,
    Deprecated,
    RunBlock,
    IoApicEoi,
    Suspended,
    InOutStr(u16 /* port */, u32 /* eax */),
    TaskSwitch,
    Monitor,
    Mwait,
    Svm,
    ReqIdle,
    Debug,
    VmInsn,
    Ht,
    Max,
}
