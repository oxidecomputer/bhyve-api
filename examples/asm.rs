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

const RTC_LMEM_LSB: i32 = 0x34;
const RTC_LMEM_MSB: i32 = 0x35;

const KB: usize = 1024;
const MB: usize = (1024 * KB);

const m_64KB: usize = (64*KB);
const m_16MB: usize = (16*MB);



fn main() {
    let vm_name = "helloworld";
    let mem_size: usize = 20 * MB;
    let guest_addr: usize = 0xfff0;
    let asm_code: &[u8] = &[
        0xba, 0xf8, 0x03, /* mov $0x3f8, %dx */
        0x00, 0xd8,       /* add %bl, %al */
        0x04, b'0',       /* add $'0', %al */
        0xee,             /* out %al, %dx */
//        0xb0, 0x0a,       /* mov $'\n', %al */
//        0xee,             /* out %al, %dx */
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

    vm.reinit().expect("failed to re-initialize VM");
    vm.set_topology(1, 1, 1).expect("failed to set CPU topology");
    vm.set_x2apic_state(BSP, false).expect("failed to disable x2APIC");
    vm.set_capability(BSP, vm_cap_type::VM_CAP_UNRESTRICTED_GUEST, 1).expect("unrestricted guest capability not available");
    vm.set_capability(BSP, vm_cap_type::VM_CAP_HALT_EXIT, 1).expect("exit on halt guest capability not available");

    vm.setup_lowmem(host_addr as u64, mem_size).expect("failed to set guest memory");

    let lomem = (mem_size - m_16MB) / m_64KB;
    vm.rtc_write(RTC_LMEM_LSB, lomem as u8).expect("failed to set RTC memory size");
    vm.rtc_write(RTC_LMEM_MSB, (lomem >> 8) as u8).expect("failed to set RTC memory size");

    // Write the x86 assembly code in the guest memory.
    let offset: u64 = host_addr as u64 + guest_addr as u64;
    let slice_size = mem_size - guest_addr;
    unsafe {
        let mut slice = slice::from_raw_parts_mut(offset as *mut u8, slice_size);
        slice.write(&asm_code).unwrap();
    }

    // Setup registers
    vm.vcpu_reset(BSP).expect("failed to set initial state of registers");

    let (base, limit, access) = vm.get_desc(BSP, vm_reg_name::VM_REG_GUEST_CS).expect("failed to get CS desc");
    vm.set_desc(BSP, vm_reg_name::VM_REG_GUEST_CS, guest_addr as u64, limit, access).expect("failed to set CS desc");

    vm.set_register(BSP, vm_reg_name::VM_REG_GUEST_RIP, guest_addr as u64).expect("failed to set RIP register");
    println!("Setting inputs to multiply 2 x 3");
    vm.set_register(BSP, vm_reg_name::VM_REG_GUEST_RAX, 2).expect("failed to set RAX register");
    vm.set_register(BSP, vm_reg_name::VM_REG_GUEST_RBX, 3).expect("failed to set RBX register");


    match vm.activate_vcpu(BSP) {
        Ok(_) => println!("Activated CPU 0 for VM at /dev/vmm/{}", vm_name),
        Err(e) => println!("Failed to activate CPU 0 for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };

    loop {
        let rip = vm.get_register(BSP, vm_reg_name::VM_REG_GUEST_RIP).unwrap();
        println!("RIP reg before run is {}", rip);

        match vm.run(BSP).expect("failed to run VM") {
            VmExit::InOut(port, eax) => {
                println!("exit for InOut, port={}, eax={}", port, eax);
                if eax == 53 {
                    println!("Got expected result, ASCII code for the number 5");
                }
            }
            VmExit::InOutStr(port, eax) => {
                println!("exit for InOutStr, port={}, eax={}", port, eax);
            }
            VmExit::Vmx(s, r, q, t, e) => {
                println!("exit for Vmx, source={}, reason={}, qualification={:b}, inst type={}, inst error={}", s, r, q, t, e);
                if r == 2 {
                    println!("Exit reason is triple fault");
                    break;
                }
            }
            VmExit::Bogus => {
                println!("exit for Bogus");
                break;
            }
            VmExit::Halt => {
                println!("exit for Halt");
                break;
            }
            VmExit::Suspended => {
                println!("exit for Suspended");
                break;
            }
            reason => println!("Unhandled exit reason {:?}", reason)
        }
    }


    vmmctl.destroy_vm(vm_name).expect("failed to destroy VM");
    println!("Destroyed a device at /dev/vmm/{}", vm_name);
}
