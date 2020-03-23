extern crate bhyve_api;

use bhyve_api::system::*;
use bhyve_api::vm::*;

#[test]
fn test_create_vm() {
    let vm_name = "testname";
    let vmmctl = VMMSystem::new().expect("failed to create VMM system ioctl handle");
    vmmctl.create_vm(vm_name).expect("failed to create VM device");
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    assert_eq!(vm.name, "testname");
    vmmctl.destroy_vm(vm_name).expect("failed to destroy VM");
}
