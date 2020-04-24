//! Constants and structs for interfacing with the Bhyve ioctl interface.
//!
//! These are defined in Rust, but mimic the C constants and structs
//! defined in `machine/vmm_dev.h`, `sys/ioccom.h`, and `sys/time.h`.

use std::os::raw::{c_int, c_uint, c_long, c_longlong, c_ulonglong, c_char};
use std::mem::size_of;
use libc::{size_t};

use crate::include::vmm::*;

// Define const from sys/param.h

const SPECNAMELEN: usize = 63; // max length of devicename

// Define struct from sys/time.h

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct timeval {
    pub tv_sec: c_long,    // seconds
    pub tv_usec: c_long,   // and microseconds
}

// Define constants from sys/ioccom.h

// Ioctl's have the command encoded in the lower word, and the size of
// any in or out parameters in the upper word.  The high 3 bits of the
// upper word are used to encode the in/out status of the parameter.

const IOCPARM_MASK: c_uint = 0xff;      // parameters must be < 256 bytes
const IOCPARM_SIZESHIFT: c_uint = 16;

const IOC_VOID: c_uint = 0x20000000;    // no parameters
const IOC_OUT: c_uint = 0x40000000;     // copy out parameters
const IOC_IN: c_uint = 0x80000000;      // copy in parameters
const IOC_INOUT: c_uint = (IOC_IN|IOC_OUT);

// Macro for defining all Bhyve ioctl operation constants.
macro_rules! define_ioctl_op {
    ($inout:expr, $ioctl_number:expr, $param_size:expr) => {
        (($inout) | ((($param_size) & IOCPARM_MASK) << IOCPARM_SIZESHIFT) | (VMM_IOC_GROUP) | ($ioctl_number)) as c_int
    };
}

// Define constants from machine/vmm_dev.h

// Identifies ioctl ops for Bhyve
const VMM_IOC_GROUP: c_uint = (118 << 8); // 118 is ASCII for 'v'

#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone)]
enum IocNum {
        // general routines
        IOCNUM_ABIVERS = 0,
        IOCNUM_RUN = 1,
        IOCNUM_SET_CAPABILITY = 2,
        IOCNUM_GET_CAPABILITY = 3,
        IOCNUM_SUSPEND = 4,
        IOCNUM_REINIT = 5,

        // memory apis
        IOCNUM_MAP_MEMORY = 10,                 // deprecated
        IOCNUM_GET_MEMORY_SEG = 11,             // deprecated
        IOCNUM_GET_GPA_PMAP = 12,
        IOCNUM_GLA2GPA = 13,
        IOCNUM_ALLOC_MEMSEG = 14,
        IOCNUM_GET_MEMSEG = 15,
        IOCNUM_MMAP_MEMSEG = 16,
        IOCNUM_MMAP_GETNEXT = 17,
        IOCNUM_GLA2GPA_NOFAULT = 18,
        IOCNUM_MUNMAP_MEMSEG = 19,

        // register/state accessors
        IOCNUM_SET_REGISTER = 20,
        IOCNUM_GET_REGISTER = 21,
        IOCNUM_SET_SEGMENT_DESCRIPTOR = 22,
        IOCNUM_GET_SEGMENT_DESCRIPTOR = 23,
        IOCNUM_SET_REGISTER_SET = 24,
        IOCNUM_GET_REGISTER_SET = 25,

        // interrupt injection
        IOCNUM_GET_INTINFO = 28,
        IOCNUM_SET_INTINFO = 29,
        IOCNUM_INJECT_EXCEPTION = 30,
        IOCNUM_LAPIC_IRQ = 31,
        IOCNUM_INJECT_NMI = 32,
        IOCNUM_IOAPIC_ASSERT_IRQ = 33,
        IOCNUM_IOAPIC_DEASSERT_IRQ = 34,
        IOCNUM_IOAPIC_PULSE_IRQ = 35,
        IOCNUM_LAPIC_MSI = 36,
        IOCNUM_LAPIC_LOCAL_IRQ = 37,
        IOCNUM_IOAPIC_PINCOUNT = 38,
        IOCNUM_RESTART_INSTRUCTION = 39,

        // PCI pass-thru
        IOCNUM_BIND_PPTDEV = 40,
        IOCNUM_UNBIND_PPTDEV = 41,
        IOCNUM_MAP_PPTDEV_MMIO = 42,
        IOCNUM_PPTDEV_MSI = 43,
        IOCNUM_PPTDEV_MSIX = 44,
        IOCNUM_GET_PPTDEV_LIMITS = 45,

        // statistics
        IOCNUM_VM_STATS = 50,
        IOCNUM_VM_STAT_DESC = 51,

        // kernel device state
        IOCNUM_SET_X2APIC_STATE = 60,
        IOCNUM_GET_X2APIC_STATE = 61,
        IOCNUM_GET_HPET_CAPABILITIES = 62,

        // CPU Topology
        IOCNUM_SET_TOPOLOGY = 63,
        IOCNUM_GET_TOPOLOGY = 64,

        // legacy interrupt injection
        IOCNUM_ISA_ASSERT_IRQ = 80,
        IOCNUM_ISA_DEASSERT_IRQ = 81,
        IOCNUM_ISA_PULSE_IRQ = 82,
        IOCNUM_ISA_SET_IRQ_TRIGGER = 83,

        // vm_cpuset
        IOCNUM_ACTIVATE_CPU = 90,
        IOCNUM_GET_CPUSET = 91,
        IOCNUM_SUSPEND_CPU = 92,
        IOCNUM_RESUME_CPU = 93,

        // RTC
        IOCNUM_RTC_READ = 100,
        IOCNUM_RTC_WRITE = 101,
        IOCNUM_RTC_SETTIME = 102,
        IOCNUM_RTC_GETTIME = 103,

        // illumos-custom ioctls
        IOCNUM_DEVMEM_GETOFFSET = 256,
        IOCNUM_WRLOCK_CYCLE = 257,
}

pub const VM_RUN: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_RUN as c_uint, (size_of::<vm_run>() as c_uint));
pub const VM_SUSPEND: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SUSPEND as c_uint, (size_of::<vm_suspend>() as c_uint));
pub const VM_REINIT: c_int = define_ioctl_op!(IOC_VOID, IocNum::IOCNUM_REINIT as c_uint, 0);

pub const VM_ALLOC_MEMSEG: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_ALLOC_MEMSEG as c_uint, (size_of::<vm_memseg>() as c_uint));
pub const VM_GET_MEMSEG: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_GET_MEMSEG as c_uint, (size_of::<vm_memseg>() as c_uint));

pub const VM_MMAP_MEMSEG: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_MMAP_MEMSEG as c_uint, (size_of::<vm_memmap>() as c_uint));
pub const VM_MMAP_GETNEXT: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_MMAP_GETNEXT as c_uint, (size_of::<vm_memmap>() as c_uint));
pub const VM_MUNMAP_MEMSEG: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_MUNMAP_MEMSEG as c_uint, (size_of::<vm_munmap>() as c_uint));

pub const VM_SET_REGISTER: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SET_REGISTER as c_uint, (size_of::<vm_register>() as c_uint));
pub const VM_GET_REGISTER: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_GET_REGISTER as c_uint, (size_of::<vm_register>() as c_uint));
pub const VM_SET_SEGMENT_DESCRIPTOR: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SET_SEGMENT_DESCRIPTOR as c_uint, (size_of::<vm_seg_desc>() as c_uint));
pub const VM_GET_SEGMENT_DESCRIPTOR: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_GET_SEGMENT_DESCRIPTOR as c_uint, (size_of::<vm_seg_desc>() as c_uint));

pub const VM_SET_CAPABILITY: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SET_CAPABILITY as c_uint, (size_of::<vm_capability>() as c_uint));
pub const VM_GET_CAPABILITY: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_GET_CAPABILITY as c_uint, (size_of::<vm_capability>() as c_uint));

pub const VM_SET_X2APIC_STATE: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SET_X2APIC_STATE as c_uint, (size_of::<vm_x2apic>() as c_uint));
pub const VM_GET_X2APIC_STATE: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_GET_X2APIC_STATE as c_uint, (size_of::<vm_x2apic>() as c_uint));

pub const VM_SET_TOPOLOGY: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SET_TOPOLOGY as c_uint, (size_of::<vm_cpu_topology>() as c_uint));
pub const VM_GET_TOPOLOGY: c_int = define_ioctl_op!(IOC_OUT, IocNum::IOCNUM_GET_TOPOLOGY as c_uint, (size_of::<vm_cpu_topology>() as c_uint));
pub const VM_STATS_IOC: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_VM_STATS as c_uint, (size_of::<vm_stats>() as c_uint));


pub const VM_ACTIVATE_CPU: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_ACTIVATE_CPU as c_uint, (size_of::<vm_activate_cpu>() as c_uint));
pub const VM_SUSPEND_CPU: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SUSPEND_CPU as c_uint, (size_of::<vm_activate_cpu>() as c_uint));
pub const VM_RESUME_CPU: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_RESUME_CPU as c_uint, (size_of::<vm_activate_cpu>() as c_uint));

pub const VM_RTC_WRITE: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_RTC_WRITE as c_uint, (size_of::<vm_rtc_data>() as c_uint));
pub const VM_RTC_READ: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_RTC_READ as c_uint, (size_of::<vm_rtc_data>() as c_uint));
pub const VM_RTC_SETTIME: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_RTC_SETTIME as c_uint, (size_of::<vm_rtc_time>() as c_uint));
pub const VM_RTC_GETTIME: c_int = define_ioctl_op!(IOC_OUT, IocNum::IOCNUM_RTC_GETTIME as c_uint, (size_of::<vm_rtc_time>() as c_uint));

pub const VM_DEVMEM_GETOFFSET: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_DEVMEM_GETOFFSET as c_uint, (size_of::<vm_devmem_offset>() as c_uint));


// ioctls used against ctl device for vm create/destroy
const VMM_IOC_BASE: c_int = ((86 << 16) | (77 << 8)); // ASCII for 'V' and 'M'
pub const VMM_CREATE_VM: c_int = (VMM_IOC_BASE | 0x01);
pub const VMM_DESTROY_VM: c_int = (VMM_IOC_BASE | 0x02);


// Define structs from machine/vmm_dev.h

// For VM_MMAP_MEMSEG
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_memmap {
    pub gpa: c_ulonglong,
    pub segid: c_int,            // memory segment
    pub segoff: c_longlong,      // offset into memory segment
    pub len: size_t,             // mmap length
    pub prot: c_int,             // RWX
    pub flags: c_int,
}

pub const VM_MEMMAP_F_WIRED: c_int = 0x01;
#[allow(unused)]
pub const VM_MEMMAP_F_IOMMU: c_int = 0x02;

// For VM_MUNMAP_MEMSEG
#[repr(C)]
pub struct vm_munmap {
    pub gpa: c_ulonglong,
    pub len: size_t,
}


// For VM_ALLOC_MEMSEG and VM_GET_MEMSEG
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_memseg {
    pub segid: c_int,
    pub len: size_t,
    pub name: [c_char; SPECNAMELEN + 1],
}

impl Default for vm_memseg {
    fn default() -> vm_memseg {
        vm_memseg {
            segid: 0,
            len: 0,
            name: [0 as c_char; SPECNAMELEN + 1],
        }
    }
}

// For VM_RTC_SETTIME and VM_RTC_GETTIME
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_rtc_time {
    pub secs: c_longlong,
}

// For VM_RTC_WRITE and VM_RTC_READ
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_rtc_data {
    pub offset: c_int,
    pub value: u8,
}

// For VM_DEVMEM_GETOFFSET
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_devmem_offset {
    pub segid: c_int,
    pub offset: c_longlong,
}

// For VM_SET_REGISTER and VM_GET_REGISTER
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_register {
    pub cpuid: c_int,
    pub regnum: c_int,      // enum vm_reg_name
    pub regval: c_ulonglong,
}

// For VM_SET_SEGMENT_DESCRIPTOR and VM_GET_SEGMENT_DESCRIPTOR
// data or code segment
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_seg_desc {
    pub cpuid: c_int,
    pub regnum: c_int,      // enum vm_reg_name
    pub desc: seg_desc,     // struct seg_desc
}

// For VM_RUN
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_run {
    pub cpuid: c_int,
    pub vm_exit: vm_exit,
}

// For VM_SET_CAPABILITY and VM_GET_CAPABILITY
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_capability {
    pub cpuid: c_int,
    pub captype: vm_cap_type, // enum vm_cap_type
    pub capval: c_int,
    pub allcpus: c_int,
}

impl Default for vm_capability {
    fn default() -> vm_capability {
        vm_capability {
            cpuid: 0,
            captype: vm_cap_type::VM_CAP_MAX,
            capval: 0,
            allcpus: 0,
        }
    }
}

// For VM_GET_X2APIC_STATE and VM_SET_X2APIC_STATE
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_x2apic {
    pub cpuid: c_int,
    pub state: x2apic_state,
}

impl Default for vm_x2apic {
    fn default() -> vm_x2apic {
        vm_x2apic {
            cpuid: 0,
            state: x2apic_state::X2APIC_DISABLED,
        }
    }
}

// For VM_SUSPEND
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_suspend {
    pub how: vm_suspend_how,
}

// For VM_ACTIVATE_CPU, VM_SUSPEND_CPU, and VM_RESUME_CPU
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_activate_cpu {
    pub vcpuid: c_int,
}

// For VM_SET_TOPOLOGY and VM_GET_TOPOLOGY
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_cpu_topology {
    pub sockets: u16,
    pub cores: u16,
    pub threads: u16,
    pub maxcpus: u16,
}

const MAX_VM_STATS: usize = (64 + VM_MAXCPU);

// For VM_STATS_IOC
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_stats {
    pub cpuid: c_int,       // in
    pub num_entries: c_int, // out
    pub tv: timeval,
    pub statbuf: [c_ulonglong; MAX_VM_STATS],
}

impl Default for vm_stats {
    fn default() -> vm_stats {
        vm_stats {
            cpuid: 0,
            num_entries: 0,
            tv: timeval::default(),
            statbuf: [0; MAX_VM_STATS],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::include::vmm_dev::*;

    #[test]
    fn test_ioctl_stats() {
        assert_eq!(size_of::<vm_stats>(), 0x318);
        assert_eq!(VM_STATS_IOC as u32, 0xc0187632);
    }

    #[test]
    fn test_ioctl_general() {
        assert_eq!(size_of::<vm_run>(), 0x90);
        assert_eq!(size_of::<vm_suspend>(), 4);

        //assert_eq!(VM_RUN as u32, 0xc0847601);
        assert_eq!(VM_RUN as u32, 0xc0907601);
        assert_eq!(VM_SUSPEND as u32, 0x80047604);
        assert_eq!(VM_REINIT as u32, 0x20007605);
    }

    #[test]
    fn test_ioctl_topology() {
        assert_eq!(size_of::<vm_activate_cpu>(), 4);
        assert_eq!(size_of::<vm_cpu_topology>(), 8);
        assert_eq!(VM_SET_TOPOLOGY as u32, 0x8008763f);
        assert_eq!(VM_GET_TOPOLOGY as u32, 0x40087640);
    }
}
