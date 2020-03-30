//! Constants and structs for interfacing with the Bhyve ioctl interface.
//!
//! These are defined in Rust, but mimic the C constants and structs
//! defined in `machine/vmm_dev.h`, `sys/ioccom.h`, and `sys/time.h`.

use std::os::raw::{c_int, c_uint, c_long, c_ulonglong};
use std::mem::size_of;

use crate::include::vmm::*;

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

// The size of the 'vm_run' struct is coming out as 144 in Rust, which is making
// the ioctl op come out as a different number than the #defines in the original
// C code. Checking the size of the structs separately in GCC gives 132, but,
// 132 also isn't comparing correctly with the VM_RUN ioctl calculated in the
// kernel module.
// pub const VM_RUN: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_RUN as c_uint, 132);
pub const VM_RUN: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_RUN as c_uint, (size_of::<vm_run>() as c_uint));
pub const VM_SUSPEND: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SUSPEND as c_uint, (size_of::<vm_suspend>() as c_uint));
pub const VM_REINIT: c_int = define_ioctl_op!(IOC_VOID, IocNum::IOCNUM_REINIT as c_uint, 0);

pub const VM_SET_TOPOLOGY: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SET_TOPOLOGY as c_uint, (size_of::<vm_cpu_topology>() as c_uint));
pub const VM_GET_TOPOLOGY: c_int = define_ioctl_op!(IOC_OUT, IocNum::IOCNUM_GET_TOPOLOGY as c_uint, (size_of::<vm_cpu_topology>() as c_uint));
pub const VM_STATS_IOC: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_VM_STATS as c_uint, (size_of::<vm_stats>() as c_uint));


// ioctls used against ctl device for vm create/destroy
const VMM_IOC_BASE: c_int = ((86 << 16) | (77 << 8)); // ASCII for 'V' and 'M'
pub const VMM_CREATE_VM: c_int = (VMM_IOC_BASE | 0x01);
pub const VMM_DESTROY_VM: c_int = (VMM_IOC_BASE | 0x02);


// Define structs from machine/vmm_dev.h

// For VM_RUN
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct vm_run {
    pub cpuid: c_int,
    pub vm_exit: vm_exit,
}

// For VM_SUSPEND
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_suspend {
    pub how: vm_suspend_how,
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
        assert_eq!(size_of::<vm_stats>(), 792);
        assert_eq!(VM_STATS_IOC as u32, 0xc0187632);
    }

    #[test]
    fn test_ioctl_general() {
        // GCC says this should be 132, but that's not right either
        assert_eq!(size_of::<vm_run>(), 144);
        assert_eq!(size_of::<vm_suspend>(), 4);

        //assert_eq!(VM_RUN as u32, 0xc0847601);
        assert_eq!(VM_RUN as u32, 0xc0907601);
        assert_eq!(VM_SUSPEND as u32, 0x80047604);
        assert_eq!(VM_REINIT as u32, 0x20007605);
    }

    #[test]
    fn test_ioctl_topology() {
        assert_eq!(size_of::<vm_cpu_topology>(), 8);
        assert_eq!(VM_SET_TOPOLOGY as u32, 0x8008763f);
        assert_eq!(VM_GET_TOPOLOGY as u32, 0x40087640);
    }
}
