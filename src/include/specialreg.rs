//! Constants for interfacing with the Bhyve ioctl interface.
//!
//! These are defined in Rust, but mimic the C constants defined
//! in `machine/specialreg.h`.

pub const CR0_NE: u64 = 0x00000020; // Numeric Error enable (EX16 vs IRQ13)
