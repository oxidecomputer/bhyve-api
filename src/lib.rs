//! # Rust library interface to Bhyve
//!
//! The Bhyve API library is a userspace interface to the hardware
//! virtualization features in the Illumos kernel provided by Bhyve. It is
//! a minimal interface, which can serve as the lowest userspace base
//! layer for any hypervisor to use Bhyve's hardware virtualization features. It
//! is essentially no more than a clean, safe wrapper around Bhyve's ioctl
//! API (i.e `/dev/vvmctl`).
//!
//! As part of being a minimal interface, `bhyve-api` avoids external
//! dependencies as much as possible, using only `libc` to access the
//! ioctl interface. The constants and structs required for the Bhyve
//! API are defined in Rust, rather than automatically generating
//! bindings to the C header files, which has benefits for usability
//! and maintainability, and simplifies reasoning from a security
//! perspective.

pub mod system;
pub mod vm;
mod vmm_dev;
