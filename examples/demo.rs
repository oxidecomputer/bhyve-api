// A simple demonstration.

extern crate bhyve_api;

use bhyve_api::system::*;
use bhyve_api::vm::*;

fn main() {
    let vm_name = "unique";
    let vmmctl = VMMSystem::new().expect("failed to create VMM system ioctl handle");
    println!("Opened a filehandle to /dev/vmmctl");
    vmmctl.create_vm(vm_name).expect("failed to create VM device");
    println!("Created a device at /dev/vmm/{}", vm_name);

    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    vm.run(1).expect("failed to run VM");

    vm.reinit().expect("failed to reinit VM");
    vm.halt().expect("failed to halt VM");

    vmmctl.destroy_vm(vm_name).expect("failed to destroy VM");
    println!("Destroyed a device at /dev/vmm/{}", vm_name);
}
