// A minimal example running x86 assembly language.

// This example is based on https://lwn.net/Articles/658511/.
// Portions Copyright 2017 The Chromium OS Authors, and Copyright 2018
// Amazon.com, Inc.

extern crate bhyve_api;

use bhyve_api::system::*;
use bhyve_api::vm::*;

use std::io::Write;
use std::slice;
use std::ptr::null_mut;

const MB: u64 = (1024 * 1024);
//const GB: u64 = (1024 * MB);
//const MAX_BOOTROM_SIZE: i32 = 16 * MB;

const BSP: i32 = 0;

fn main() {
    let vm_name = "helloworld";
    let bootrom_size: usize = 0x1000;
    let guest_mem_size: usize = MB as usize;
    let asm_code: &[u8] = &[
        0xba, 0xf8, 0x03, /* mov $0x3f8, %dx */
        0x00, 0xd8,       /* add %bl, %al */
        0x04, b'0',       /* add $'0', %al */
        0xee,             /* out %al, %dx */
        0xec,             /* in %dx, %al */
        0xf4,             /* hlt */
    ];


    let guest_mem_addr: *mut u8 = unsafe {
        libc::mmap(
            null_mut(),
            guest_mem_size,
            libc::PROT_NONE,
            libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_NORESERVE,
            -1,
            0,
        ) as *mut u8
    };

    let bootrom_addr: *mut u8 = unsafe {
        libc::mmap(
            null_mut(),
            bootrom_size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_NORESERVE,
            -1,
            0,
        ) as *mut u8
    };


    let vmmctl = VMMSystem::new().expect("failed to create VMM system ioctl handle");
    println!("Opened a filehandle to /dev/vmmctl");
    vmmctl.create_vm(vm_name).expect("failed to create VM device");
    println!("Created a device at /dev/vmm/{}", vm_name);

    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    vm.set_topology(1, 1, 1).expect("failed to set CPU topology");
    vm.set_x2apic_state(BSP, false).expect("failed to disable x2APIC");

    vm.setup_lowmem(guest_mem_addr as u64, guest_mem_size).expect("failed to set guest memory");
    vm.setup_bootrom(bootrom_addr as u64, bootrom_size).expect("failed to set bootrom memory");

    // Write the x86 assembly code in the guest bootrom memory.
    unsafe {
        let mut slice = slice::from_raw_parts_mut(bootrom_addr, bootrom_size);
        slice.write(&asm_code).unwrap();
    }

    // Setup registers
//    let mut vcpu_sregs = vcpu_fd.get_sregs().unwrap();
//    vcpu_sregs.cs.base = 0;
//    vcpu_sregs.cs.selector = 0;
//    vcpu_fd.set_sregs(&vcpu_sregs).unwrap();

//    let mut vcpu_regs = vcpu_fd.get_regs().unwrap();
//    vcpu_regs.rip = guest_addr;
//    vcpu_regs.rax = 2;
//    vcpu_regs.rbx = 3;
//    vcpu_regs.rflags = 2;
//    vcpu_fd.set_regs(&vcpu_regs).unwrap();

//    loop {
//        match vm.run(1).expect("failed to run VM") {
//            VmExit::InOut => {
//                println!("exit for InOut");
//            }
//            VmExit::Halt => {
//                println!("exit for Halt");
//                break;
//            }
//            reason => println!("Unhandled exit reason {:?}", reason)
//        }
//    }


    vmmctl.destroy_vm(vm_name).expect("failed to destroy VM");
    println!("Destroyed a device at /dev/vmm/{}", vm_name);
}
