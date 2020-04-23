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

const BSP: i32 = 0;

fn main() {
    let vm_name = "helloworld";
    let mem_size: usize = 0x4000;
    let asm_code: &[u8] = &[
        0xba, 0xf8, 0x03, /* mov $0x3f8, %dx */
        0x00, 0xd8,       /* add %bl, %al */
        0x04, b'0',       /* add $'0', %al */
        0xee,             /* out %al, %dx */
        0xec,             /* in %dx, %al */
        0xf4,             /* hlt */
    ];


    let host_addr: *mut u8 = unsafe {
        libc::mmap(
            null_mut(),
            mem_size,
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

    vm.setup_lowmem(host_addr as u64, mem_size).expect("failed to set guest memory");

    // Write the x86 assembly code in the guest memory.
    unsafe {
        let mut slice = slice::from_raw_parts_mut(host_addr, mem_size);
        slice.write(&asm_code).unwrap();
    }

    let rip = vm.get_register(BSP, vm_reg_name::VM_REG_GUEST_RIP).unwrap();
    println!("RIP reg is {}", rip);

    // Setup registers
    vm.set_register(BSP, vm_reg_name::VM_REG_GUEST_CS, 0).expect("failed to set CS register");

    vm.set_register(BSP, vm_reg_name::VM_REG_GUEST_RAX, 2).expect("failed to set RAX register");
    vm.set_register(BSP, vm_reg_name::VM_REG_GUEST_RBX, 3).expect("failed to set RBX register");
    vm.set_register(BSP, vm_reg_name::VM_REG_GUEST_RFLAGS, 3).expect("failed to set RFLAGS register");

    match vm.activate_vcpu(BSP) {
        Ok(_) => println!("Activated CPU 0 for VM at /dev/vmm/{}", vm_name),
        Err(e) => println!("Failed to activate CPU 0 for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };

    loop {
        match vm.run(BSP).expect("failed to run VM") {
            VmExit::InOut => {
                println!("exit for InOut");
            }
            VmExit::Vmx => {
                println!("exit for Vmx");
                break;
            }
            VmExit::Halt => {
                println!("exit for Halt");
                break;
            }
            reason => println!("Unhandled exit reason {:?}", reason)
        }
    }


    vmmctl.destroy_vm(vm_name).expect("failed to destroy VM");
    println!("Destroyed a device at /dev/vmm/{}", vm_name);
}
