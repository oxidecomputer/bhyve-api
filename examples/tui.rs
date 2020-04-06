// A simplistic text user interface (command-line interface) for the library.
// Avoiding dev dependencies on external crates like Clap, so the argument
// handling is primitive.

extern crate bhyve_api;

use std::env;
use bhyve_api::system::*;
use bhyve_api::vm::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if "create" == &args[1] {
        cmd_create(&args[2]);
    } else if "destroy" == &args[1] {
        cmd_destroy(&args[2]);
    } else if "run" == &args[1] {
        cmd_run_vm(&args[2]);
    } else if "stats" == &args[1] {
        cmd_stats_vm(&args[2]);
    } else if "topology" == &args[1] {
        cmd_vcpu_top(&args[2]);
    } else if "activate" == &args[1] {
        cmd_vcpu_activate(&args[2]);
    } else if "suspend" == &args[1] {
        cmd_vcpu_suspend(&args[2]);
    } else if "resume" == &args[1] {
        cmd_vcpu_resume(&args[2]);
    }
}

fn cmd_create(vm_name: &str) {
    let vmmctl = VMMSystem::new().expect("failed to create VMM system ioctl handle");
    match vmmctl.create_vm(vm_name) {
        Ok(_) => println!("Created a device at /dev/vmm/{}", vm_name),
        Err(e) => println!("Unable to create device at /dev/vmm/{}, with error: {}", vm_name, e),
    };
}

fn cmd_destroy(vm_name: &str) {
    let vmmctl = VMMSystem::new().expect("failed to create VMM system ioctl handle");
    match vmmctl.destroy_vm(vm_name) {
        Ok(_) => println!("Destroyed a device at /dev/vmm/{}", vm_name),
        Err(e) => println!("Unable to destroy device at /dev/vmm/{}, with error: {}", vm_name, e),
    };
}

fn cmd_run_vm(vm_name: &str) {
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    match vm.set_topology(1, 1, 1, 2) {
        Ok(_) => println!("Set CPU topology for VM at /dev/vmm/{}", vm_name),
        Err(e) => println!("Failed to set CPU topology for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };

    let (sockets, cores, threads, maxcpus) = vm.get_topology().expect("failed to get CPU topology for VM");
    println!("CPU topology current values: sockets={}, cores={}, threads={}, maxcpus={}", sockets, cores, threads, maxcpus);

    match vm.run(0) {
        Ok(_) => println!("Successful run for VM at /dev/vmm/{}", vm_name),
        Err(e) => println!("Failed run for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };
}

fn cmd_vcpu_top(vm_name: &str) {
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    let (sockets, cores, threads, maxcpus) = vm.get_topology().expect("failed to get CPU topology for VM");
    println!("CPU topology current values: sockets={}, cores={}, threads={}, maxcpus={}", sockets, cores, threads, maxcpus);
}

fn cmd_stats_vm(vm_name: &str) {
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    match vm.get_stats(0) {
        Ok(entries) => println!("Got stats for VM at /dev/vmm/{}, {} entries", vm_name, entries),
        Err(e) => println!("Failed to get stats for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };
}

fn cmd_vcpu_activate(vm_name: &str) {
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    match vm.activate_vcpu(0) {
        Ok(_) => println!("Activated CPU 0 for VM at /dev/vmm/{}", vm_name),
        Err(e) => println!("Failed to activate CPU 0 for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };
}

fn cmd_vcpu_suspend(vm_name: &str) {
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    match vm.suspend_vcpu(0) {
        Ok(_) => println!("Suspended CPU 0 for VM at /dev/vmm/{}", vm_name),
        Err(e) => println!("Failed to suspend CPU 0 for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };
}

fn cmd_vcpu_resume(vm_name: &str) {
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    println!("Opened a filehandle to /dev/vmm/{}", vm.name);

    match vm.resume_vcpu(0) {
        Ok(_) => println!("Resumed CPU 0 for VM at /dev/vmm/{}", vm_name),
        Err(e) => println!("Failed to resume CPU 0 for VM at /dev/vmm/{}, with error: {}", vm_name, e),
    };
}
