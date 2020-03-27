//! Constants and structs for interfacing with the Bhyve ioctl interface.
//!
//! These are defined in Rust, but mimic the C constants and structs
//! defined in `machine/vmm_dev.h` and `sys/ioccom.h`.

use std::os::raw::{c_int, c_uint};
use std::mem::size_of;

use crate::include::vmm::*;

// Define constants from sys/ioccom.h

// Ioctl's have the command encoded in the lower word, and the size of
// any in or out parameters in the upper word.  The high 3 bits of the
// upper word are used to encode the in/out status of the parameter.

const IOCPARM_SHIFT: c_uint = 13; // number of bits for ioctl size
const IOCPARM_MASK: c_uint = ((1 << IOCPARM_SHIFT) - 1); // parameter length mask
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
}


pub const VM_RUN: c_int = define_ioctl_op!(IOC_INOUT, IocNum::IOCNUM_RUN as c_uint, (size_of::<vm_run>() as c_uint));
pub const VM_SUSPEND: c_int = define_ioctl_op!(IOC_IN, IocNum::IOCNUM_SUSPEND as c_uint, (size_of::<vm_suspend>() as c_uint));
pub const VM_REINIT: c_int = define_ioctl_op!(IOC_VOID, IocNum::IOCNUM_REINIT as c_uint, 0);


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
