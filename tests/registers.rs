extern crate bhyve_api;

use bhyve_api::system::*;
use bhyve_api::vm::*;

const TEST_CPUID: i32 = 0;

fn setup_vm(vm_name: &str) -> VirtualMachine {
    let vmmctl = VMMSystem::new().expect("failed to create VMM system ioctl handle");
    vmmctl.create_vm(vm_name).expect("failed to create VM device");
    let vm = VirtualMachine::new(vm_name).expect("failed to open filehandle to VM device");
    return vm;
}

fn teardown_vm(vm_name: &str) {
    let vmmctl = VMMSystem::new().expect("failed to create VMM system ioctl handle");
    vmmctl.destroy_vm(vm_name).expect("failed to destroy VM");
}

#[test]
fn test_caller_save_registers() {
    let testname = "test_caller_save_registers";
    let vm = setup_vm(testname);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RAX, 1100).expect("failed to set RAX register");
    let rax = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RAX).expect("failed to get RAX register");
    assert_eq!(rax, 1100);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RCX, 1200).expect("failed to set RCX register");
    let rcx = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RCX).expect("failed to get RCX register");
    assert_eq!(rcx, 1200);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RDX, 1300).expect("failed to set RDX register");
    let rdx = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RDX).expect("failed to get RDX register");
    assert_eq!(rdx, 1300);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RDI, 1400).expect("failed to set RDI register");
    let rdi = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RDI).expect("failed to get RDI register");
    assert_eq!(rdi, 1400);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RSI, 1500).expect("failed to set RSI register");
    let rsi = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RSI).expect("failed to get RSI register");
    assert_eq!(rsi, 1500);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RSP, 1600).expect("failed to set RSP register");
    let rsp = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RSP).expect("failed to get RSP register");
    assert_eq!(rsp, 1600);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R8, 1008).expect("failed to set R8 register");
    let r8 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R8).expect("failed to get R8 register");
    assert_eq!(r8, 1008);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R9, 1009).expect("failed to set R9 register");
    let r9 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R9).expect("failed to get R9 register");
    assert_eq!(r9, 1009);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R10, 1010).expect("failed to set R10 register");
    let r10 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R10).expect("failed to get R10 register");
    assert_eq!(r10, 1010);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R11, 1011).expect("failed to set R11 register");
    let r11 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R11).expect("failed to get R11 register");
    assert_eq!(r11, 1011);

    teardown_vm(testname);
}

#[test]
fn test_callee_save_registers() {
    let testname = "test_callee_save_registers";
    let vm = setup_vm(testname);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RBX, 2100).expect("failed to set RBX register");
    let rbx = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RBX).expect("failed to get RBX register");
    assert_eq!(rbx, 2100);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RBP, 2200).expect("failed to set RBP register");
    let rbp = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RBP).expect("failed to get RBP register");
    assert_eq!(rbp, 2200);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R12, 2012).expect("failed to set R12 register");
    let r12 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R12).expect("failed to get R12 register");
    assert_eq!(r12, 2012);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R13, 2013).expect("failed to set R13 register");
    let r13 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R13).expect("failed to get R13 register");
    assert_eq!(r13, 2013);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R14, 2014).expect("failed to set R14 register");
    let r14 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R14).expect("failed to get R14 register");
    assert_eq!(r14, 2014);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R15, 2015).expect("failed to set R15 register");
    let r15 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_R15).expect("failed to get R15 register");
    assert_eq!(r15, 2015);

    teardown_vm(testname);
}

#[test]
fn test_debug_registers() {
    let testname = "test_debug_registers";
    let vm = setup_vm(testname);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR0, 3000).expect("failed to set DR0 register");
    let dr0 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR0).expect("failed to get DR0 register");
    assert_eq!(dr0, 3000);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR1, 3001).expect("failed to set DR1 register");
    let dr1 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR1).expect("failed to get DR1 register");
    assert_eq!(dr1, 3001);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR2, 3002).expect("failed to set DR2 register");
    let dr2 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR2).expect("failed to get DR2 register");
    assert_eq!(dr2, 3002);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR3, 3003).expect("failed to set DR3 register");
    let dr3 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR3).expect("failed to get DR3 register");
    assert_eq!(dr3, 3003);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR6, 3006).expect("failed to set DR6 register");
    let dr6 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR6).expect("failed to get DR6 register");
    assert_eq!(dr6, 3006);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR7, 3007).expect("failed to set DR7 register");
    let dr7 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_DR7).expect("failed to get DR7 register");
    assert_eq!(dr7, 3007);

    teardown_vm(testname);
}

#[test]
fn test_control_registers() {
    let testname = "test_control_registers";
    let vm = setup_vm(testname);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR0, 4000).expect("failed to set CR0 register");
    let cr0 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR0).expect("failed to get CR0 register");
    assert_eq!(cr0, 4000);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR2, 4002).expect("failed to set CR2 register");
    let cr2 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR2).expect("failed to get CR2 register");
    assert_eq!(cr2, 4002);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR3, 4003).expect("failed to set CR3 register");
    let cr3 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR3).expect("failed to get CR3 register");
    assert_eq!(cr3, 4003);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR4, 4004).expect("failed to set CR4 register");
    let cr4 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR4).expect("failed to get CR4 register");
    assert_eq!(cr4, 4004);

    // NOT SUPPORTED
    //
    // CR8 or TPR (task priority register) is a new register introduced in AMD64
    // to speed interrupt management.
    // See "AMD64 Architecture Programmer's Manual, Volume 2: System Programming",
    // section 2.6.5 "Task-Priority Register (CR8)" and figure 1-7 "System Registers".
    //
    // vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR8, 4008).expect("failed to set CR8 register");
    // let cr8 = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_CR8).expect("failed to get CR8 register");
    // assert_eq!(cr8, 4008);

    teardown_vm(testname);
}

#[test]
fn test_system_flags_register() {
    let testname = "test_system_flags_register";
    let vm = setup_vm(testname);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RFLAGS, 5000).expect("failed to set RFLAGS register");
    let rflags = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_RFLAGS).expect("failed to get RFLAGS register");
    assert_eq!(rflags, 5000);

    teardown_vm(testname);
}

#[test]
fn test_descriptor_table_registers() {
    let testname = "test_descriptor_table_registers";
    let vm = setup_vm(testname);

    // Global Descriptor-Table Register
    vm.set_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_GDTR, 0, 0xffff, 0).expect("failed to set GDTR register");
    match vm.get_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_GDTR) {
        Ok(gdtr) => {
            assert_eq!(gdtr.0, 0);      // GDTR base address
            assert_eq!(gdtr.1, 0xffff); // GDTR limit
            assert_eq!(gdtr.2, 0);      // GDTR attributes
        }
        Err(error) => panic!("failed to get GDTR register: {:?}", error),
    }

    // Local Descriptor-Table Register
    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_LDTR, 6000).expect("failed to set LDTR register");
    let ldtr = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_LDTR).expect("failed to get LDTR register");
    assert_eq!(ldtr, 6000);

    vm.set_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_LDTR, 0, 0xffff, 0x082).expect("failed to set LDTR register");
    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_LDTR, 0).expect("failed to set LDTR register");

    match vm.get_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_LDTR) {
        Ok(ldtr) => {
            assert_eq!(ldtr.0, 0);      // LDTR base address
            assert_eq!(ldtr.1, 0xffff); // LDTR limit
            assert_eq!(ldtr.2, 0x082);  // LDTR attributes
        }
        Err(error) => panic!("failed to get LDTR register: {:?}", error),
    }

    // Interrupt Descriptor-Table Register
    vm.set_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_IDTR, 0, 0xffff, 0).expect("failed to set IDTR register");

    match vm.get_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_IDTR) {
        Ok(idtr) => {
            assert_eq!(idtr.0, 0);      // IDTR base address
            assert_eq!(idtr.1, 0xffff); // IDTR limit
            assert_eq!(idtr.2, 0);      // IDTR attributes
        }
        Err(error) => panic!("failed to get IDTR register: {:?}", error),
    }

    teardown_vm(testname);
}

#[test]
fn test_task_register() {
    let testname = "test_task_register";
    let vm = setup_vm(testname);

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_TR, 7000).expect("failed to set TR register");
    let tr = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_TR).expect("failed to get TR register");
    assert_eq!(tr, 7000);

    vm.set_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_TR, 0, 0, 0x08b).expect("failed to set TR register");
    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_TR, 0).expect("failed to set TR register");

    match vm.get_desc(TEST_CPUID, vm_reg_name::VM_REG_GUEST_TR) {
        Ok(ldtr) => {
            assert_eq!(ldtr.0, 0);     // TR base address
            assert_eq!(ldtr.1, 0);     // TR limit
            assert_eq!(ldtr.2, 0x08b); // TR attributes
        }
        Err(error) => panic!("failed to get TR register: {:?}", error),
    }

    teardown_vm(testname);
}

#[test]
fn test_extended_feature_register() {
    let testname = "test_extended_feature_register";
    let vm = setup_vm(testname);

    const EFER_LMA: u64 = 0x400;
    const EFER_LME: u64 = 0x100;

    let efer_orig = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_EFER).expect("failed to get EFER register");

    let longmode = efer_orig | EFER_LME | EFER_LMA;

    vm.set_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_EFER, longmode).expect("failed to set EFER register");
    let efer = vm.get_register(TEST_CPUID, vm_reg_name::VM_REG_GUEST_EFER).expect("failed to get EFER register");
    assert_eq!(efer, longmode);

    teardown_vm(testname);
}
