//! Constants and structs for interfacing with the Bhyve ioctl interface.
//!
//! These are defined in Rust, but mimic the C constants and structs
//! defined in `machine/vmm.h`.


use std::os::raw::{c_int, c_uint, c_ulonglong};

pub const VM_MAXCPU: usize = 32;    // maximum virtual cpus

#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone)]
pub enum vm_suspend_how {
        VM_SUSPEND_NONE,
        VM_SUSPEND_RESET,
        VM_SUSPEND_POWEROFF,
        VM_SUSPEND_HALT,
        VM_SUSPEND_TRIPLEFAULT,
        VM_SUSPEND_LAST
}

// Identifiers for architecturally defined registers.
#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone)]
pub enum vm_reg_name {
        VM_REG_GUEST_RAX,
        VM_REG_GUEST_RBX,
        VM_REG_GUEST_RCX,
        VM_REG_GUEST_RDX,
        VM_REG_GUEST_RSI,
        VM_REG_GUEST_RDI,
        VM_REG_GUEST_RBP,
        VM_REG_GUEST_R8,
        VM_REG_GUEST_R9,
        VM_REG_GUEST_R10,
        VM_REG_GUEST_R11,
        VM_REG_GUEST_R12,
        VM_REG_GUEST_R13,
        VM_REG_GUEST_R14,
        VM_REG_GUEST_R15,
        VM_REG_GUEST_CR0,
        VM_REG_GUEST_CR3,
        VM_REG_GUEST_CR4,
        VM_REG_GUEST_DR7,
        VM_REG_GUEST_RSP,
        VM_REG_GUEST_RIP,
        VM_REG_GUEST_RFLAGS,
        VM_REG_GUEST_ES,
        VM_REG_GUEST_CS,
        VM_REG_GUEST_SS,
        VM_REG_GUEST_DS,
        VM_REG_GUEST_FS,
        VM_REG_GUEST_GS,
        VM_REG_GUEST_LDTR,
        VM_REG_GUEST_TR,
        VM_REG_GUEST_IDTR,
        VM_REG_GUEST_GDTR,
        VM_REG_GUEST_EFER,
        VM_REG_GUEST_CR2,
        VM_REG_GUEST_PDPTE0,
        VM_REG_GUEST_PDPTE1,
        VM_REG_GUEST_PDPTE2,
        VM_REG_GUEST_PDPTE3,
        VM_REG_GUEST_INTR_SHADOW,
        VM_REG_GUEST_DR0,
        VM_REG_GUEST_DR1,
        VM_REG_GUEST_DR2,
        VM_REG_GUEST_DR3,
        VM_REG_GUEST_DR6,
        VM_REG_LAST
}


// The 'access' field has the format specified in Table 21-2 of the Intel
// Architecture Manual vol 3b.
//
// XXX The contents of the 'access' field are architecturally defined except
// bit 16 - Segment Unusable.

#[repr(C)]
#[derive(Copy, Clone)]
pub struct seg_desc {
    base: c_ulonglong,
    limit: c_uint,
    access: c_uint,
}

#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone)]
pub enum vm_cpu_mode {
        CPU_MODE_REAL,
        CPU_MODE_PROTECTED,
        CPU_MODE_COMPATIBILITY,         /* IA-32E mode (CS.L = 0) */
        CPU_MODE_64BIT,                 /* IA-32E mode (CS.L = 1) */
}

#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone)]
pub enum vm_paging_mode {
        PAGING_MODE_FLAT,
        PAGING_MODE_32,
        PAGING_MODE_PAE,
        PAGING_MODE_64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_guest_paging {
    pub cr3: c_ulonglong,
    pub cpl: c_int,
    pub cpu_mode: vm_cpu_mode,
    pub paging_mode: vm_paging_mode,
}

// The data structures 'vie' and 'vie_op' are meant to be opaque to the
// consumers of instruction decoding. The only reason why their contents
// need to be exposed is because they are part of the 'vm_exit' structure.
//
// These structs are not public, and should never be used. They are
// defined solely to allow Rust to calculate adequate memory allocation
// for the 'vm_exit' struct.

#[repr(C)]
#[derive(Copy, Clone)]
struct vie_op {
    op_byte: u8,
    op_type: u8,
    op_flags: u16,
}

// For now, this is punting on C bitfields by allocating arrays of bytes.
// If we end up using this code, we will want some more precise way to
// mimic C bitfields, like David Tolnay's #[bitfield] attribute macro.

#[repr(C)]
#[derive(Copy, Clone)]
struct vie {
    inst: [u8; 15],
    num_valid: u8,
    num_processed: u8,
    bitfields: [u8; 5], // This is two bits too large
    disp_bytes: u8,
    imm_bytes: u8,
    scale: u8,
    base_register: i32,
    index_register: i32,
    segment_register: i32,
    displacement: i64,
    immediate: i64,
    decoded: u8,
    op: vie_op,
}

#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone)]
pub enum vm_exitcode {
        VM_EXITCODE_INOUT,
        VM_EXITCODE_VMX,
        VM_EXITCODE_BOGUS,
        VM_EXITCODE_RDMSR,
        VM_EXITCODE_WRMSR,
        VM_EXITCODE_HLT,
        VM_EXITCODE_MTRAP,
        VM_EXITCODE_PAUSE,
        VM_EXITCODE_PAGING,
        VM_EXITCODE_INST_EMUL,
        VM_EXITCODE_SPINUP_AP,
        VM_EXITCODE_DEPRECATED1,        // used to be SPINDOWN_CPU
        VM_EXITCODE_RUNBLOCK,
        VM_EXITCODE_IOAPIC_EOI,
        VM_EXITCODE_SUSPENDED,
        VM_EXITCODE_INOUT_STR,
        VM_EXITCODE_TASK_SWITCH,
        VM_EXITCODE_MONITOR,
        VM_EXITCODE_MWAIT,
        VM_EXITCODE_SVM,
        VM_EXITCODE_REQIDLE,
        VM_EXITCODE_DEBUG,
        VM_EXITCODE_VMINSN,
        VM_EXITCODE_HT,
        VM_EXITCODE_MAX
}

#[repr(C)]
#[derive(Copy, Clone)]
struct vm_inout {
    bitfields: u8, // This is two bits too large
    port: u16,
    eax: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct vm_inout_str {
    inout: vm_inout, // must be the first element
    paging: vm_guest_paging,
    rflags: c_ulonglong,
    cr0: c_ulonglong,
    index: c_ulonglong,
    count: c_ulonglong,
    segname: vm_reg_name,
    seg_desc: seg_desc,
}

#[repr(C)]
#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone)]
pub enum task_switch_reason {
        TSR_CALL,
        TSR_IRET,
        TSR_JMP,
        TSR_IDT_GATE,   // task gate in IDT
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_task_switch {
    tsssel: u16,                         // new TSS selector
    ext: c_int,                          // task switch due to external event
    errcode: c_uint,
    errcode_valid: c_int,                // push 'errcode' on the new stack
    reason: task_switch_reason,
    paging: vm_guest_paging,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit {
    pub exitcode: vm_exitcode,
    pub inst_length: c_int,    // 0 means unknown
    pub rip: c_ulonglong,
    pub u: vm_exit_payload,
}

impl Default for vm_exit {
    fn default() -> vm_exit {
        let payload = vm_exit_payload { empty: 0 };
        vm_exit {
            exitcode: vm_exitcode::VM_EXITCODE_BOGUS,
            inst_length: 0,
            rip: 0,
            u: payload
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union vm_exit_payload {
    inout: vm_inout,
    inout_str: vm_inout_str,
    pub paging: vm_exit_paging,
    pub inst_emul: vm_exit_inst_emul,
    pub vmx: vm_exit_vmx,
    pub svm: vm_exit_svm,
    pub msr: vm_exit_msr,
    pub spinup_ap: vm_exit_spinup_ap,
    pub hlt: vm_exit_hlt,
    pub ioapic_eoi: vm_exit_ioapic_eoi,
    pub suspended: vm_exit_suspended,
    pub task_switch: vm_task_switch,
    empty: c_int,
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_paging {
    pub gpa: c_ulonglong,
    pub fault_type: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_inst_emul {
    pub gpa: c_ulonglong,
    pub gla: c_ulonglong,
    pub cs_base: c_ulonglong,
    pub cs_d: c_int,   // CS.D
    pub paging: vm_guest_paging,
    vie: vie,
}

// VMX specific payload. Used when there is no "better"
// exitcode to represent the VM-exit.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_vmx {
    pub status: c_int,             // vmx inst status

    // 'exit_reason' and 'exit_qualification' are valid
    // only if 'status' is zero.
    pub exit_reason: c_uint,
    pub exit_qualification: c_ulonglong,

    // 'inst_error' and 'inst_type' are valid
    // only if 'status' is non-zero.
    pub inst_type: c_int,
    pub inst_error: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_svm {
    pub exitcode: c_ulonglong,
    pub exitinfo1: c_ulonglong,
    pub exitinfo2: c_ulonglong,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_msr {
    pub code: c_uint,      // ecx value
    pub wval: c_ulonglong,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_spinup_ap {
    pub vcpu: c_int,
    pub rip: c_ulonglong,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_hlt {
    pub rflags: c_ulonglong,
    pub intr_status: c_ulonglong,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_ioapic_eoi {
    pub vector: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vm_exit_suspended {
    pub how: vm_suspend_how,
}
