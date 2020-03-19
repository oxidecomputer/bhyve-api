//! Constants and structs for interfacing with the Bhyve ioctl API.
//!
//! These are defined in Rust, but mimic the C constants and structs
//! defined in `machine/vmm_dev.h`.

use std::os::raw::c_int;

// ioctls used against ctl device for vm create/destroy
const VMM_IOC_BASE: c_int = ((86 << 16) | (77 << 8)); // ASCII for 'V' and 'M'
pub const VMM_CREATE_VM: c_int = (VMM_IOC_BASE | 0x01);
pub const VMM_DESTROY_VM: c_int = (VMM_IOC_BASE | 0x02);
