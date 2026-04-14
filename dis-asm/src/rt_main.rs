use core::arch::{asm, naked_asm};

#[repr(transparent)]
pub struct SyncPtr<T>(*const T);
unsafe impl<T> Sync for SyncPtr<T> {}

#[repr(transparent)]
pub struct SyncMutPtr<T>(*mut T);
unsafe impl<T> Sync for SyncMutPtr<T> {}

unsafe extern "system" {
    fn AddVectoredExceptionHandler(first: u32, handler: *const ()) -> *mut ();
    fn SetThreadStackGuarantee(stack_size_in_bytes: *mut u32) -> i32;
    fn GetCurrentThread() -> *mut ();
}

#[unsafe(no_mangle)]
pub static TlsIndex: u32 = 0;

#[unsafe(no_mangle)]
pub static qword_142ED0B68: u64 = 0;

#[unsafe(no_mangle)]
pub static qword_142ED0B70: u64 = 0;

#[unsafe(no_mangle)]
pub static dword_142ECA500: u32 = 0;

#[unsafe(no_mangle)]
static main_str: &[u8] = b"main\0";

#[unsafe(no_mangle)]
static error_str: &[u8] = b"Error: \0";

#[unsafe(no_mangle)]
static rt_rs_path: &[u8] = b"library\\std\\src\\rt.rs\0";

// Placeholder for unknown data
#[unsafe(no_mangle)]
static sub_140262084: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static sub_1402DCE39: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static unk_142DAF3C0: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static sub_14004148A: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static sub_140500380: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static sub_14079ED30: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static sub_14079ED70: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static off_142ECA550: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
static sub_1404F4930: SyncPtr<()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn main(_argc: i32, _argv: *mut *mut u8, _envp: *mut *mut u8) -> i32 {
    naked_asm!(
        "push rbp",
        "push rsi",
        "sub rsp, 0x88",
        "lea rbp, [rsp+0x80]",
        
        // Initialize var_18 to -2
        "mov qword ptr [rbp-0x18], 0xFFFFFFFFFFFFFFFE",
        
        // Add vectored exception handler
        "lea rdx, [rip + Handler]",
        "xor esi, esi",
        "xor ecx, ecx",
        "call {}",
        
        // Set thread stack guarantee
        "mov dword ptr [rbp-0x10], 0x5000",
        "lea rcx, [rbp-0x10]",
        "call {}",
        
        // Get current thread and call initialization
        "call {}",
        "mov r8, [rip + off_142ECA550]",
        "lea rdx, [rip + main_str]",
        "mov rcx, rax",
        "call r8",
        
        // TLS initialization
        "mov eax, [rip + TlsIndex]",
        "mov rcx, gs:[0x58]",
        "mov rax, [rcx+rax*8]",
        "mov rdx, [rax+0x140]",
        "test rdx, rdx",
        "jnz 1f",
        
        // Initialize TLS slot
        "lea rcx, [rax+0x140]",
        "mov rax, [rip + qword_142ED0B68]",
        "0:",
        "cmp rax, 0xFFFFFFFFFFFFFFFF",
        "jz 3f",
        "lea rdx, [rax+1]",
        "lock cmpxchg [rip + qword_142ED0B68], rdx",
        "jnz 0b",
        "mov [rcx], rdx",
        
        "1:",
        // Store TLS pointer
        "mov [rip + qword_142ED0B70], rdx",
        
        // Call sub_14004148A
        "lea rcx, [rip + sub_140262084]",
        "call {}",
        "test rax, rax",
        "jz 2f",
        
        // Error handling - call error formatter
        "mov [rbp-0x20], rax",
        "lea rax, [rbp-0x20]",
        "mov [rbp-0x30], rax",
        "lea rax, [rip + sub_1402DCE39]",
        "mov [rbp-0x28], rax",
        "lea rax, [rip + error_str]",
        "mov qword ptr [rbp-0x10], rax",
        "mov dword ptr [rbp-0x58], 2",
        "mov qword ptr [rbp-0x40], 0",
        "lea rax, [rbp-0x30]",
        "mov [rbp-0x50], rax",
        "mov dword ptr [rbp-0x48], 1",
        "lea rcx, [rbp-0x10]",
        "call {}",
        "mov rcx, [rbp-0x20]",
        "mov rax, [rcx]",
        "call qword ptr [rax]",
        "mov esi, 1",
        
        "2:",
        // Check global flag
        "mov eax, [rip + dword_142ECA500]",
        "test eax, eax",
        "jnz 4f",
        
        "5:",
        "mov eax, esi",
        "add rsp, 0x88",
        "pop rsi",
        "pop rbp",
        "ret",
        
        "3:",  // TLS initialization failed
        "call {}",
        "ud2",
        
        "4:",  // Error path with panic
        "mov byte ptr [rbp-0x30], 1",
        "lea rax, [rbp-0x30]",
        "mov qword ptr [rbp-0x10], rax",
        "lea rax, [rip + rt_rs_path]",
        "mov [rsp+0x90-0x20], rax",  // var_70
        "lea rcx, [rip + dword_142ECA500]",
        "lea r9, [rip + unk_142DAF3C0]",
        "lea r8, [rbp-0x10]",
        "xor edx, edx",
        "call {}",
        "jmp 5b",
        
        // Symbols
        sym AddVectoredExceptionHandler,
        sym SetThreadStackGuarantee,
        sym GetCurrentThread,
        sym sub_14004148A,
        sym sub_140500380,
        sym sub_14079ED30,
        sym sub_14079ED70
    );
}

const STATUS_STACK_OVERFLOW: u32 = 0xC00000FD;

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn Handler(exception_pointers: *mut EXCEPTION_POINTERS) -> i32 {
    naked_asm!(
        "push rbp",
        "sub rsp, 0x30",
        "lea rbp, [rsp+0x30]",
        
        // Initialize var_8 to -2
        "mov qword ptr [rbp-0x8], 0xFFFFFFFFFFFFFFFE",
        
        // Get exception code
        "mov rax, [rcx]",
        "cmp dword ptr [rax], 0xC00000FD",  // STATUS_STACK_OVERFLOW
        "jnz 9f",
        
        // Check TLS state
        "mov eax, [rip + TlsIndex]",
        "mov rcx, gs:[0x58]",
        "mov rax, [rcx+rax*8]",
        "mov rax, [rax+0x1B8]",
        "cmp rax, 2",
        "jbe 1f",
        
        // rax > 2 case
        "mov rcx, [rax+8]",
        "test rcx, rcx",
        "jz 3f",
        "mov rdx, [rax+0x10]",
        "dec rdx",
        "call {}",
        "jmp 9f",
        
        "1:",  // rax <= 2 case
        "mov rax, [rip + qword_142ED0B70]",
        "test rax, rax",
        "jz 2f",
        
        // Check TLS slot
        "mov ecx, [rip + TlsIndex]",
        "mov rdx, gs:[0x58]",
        "mov rcx, [rdx+rcx*8]",
        "cmp [rcx+0x140], rax",
        "jnz 2f",
        
        // Match - call with "mainmain"
        "lea rcx, [rip + mainmain_str]",
        "mov edx, 4",
        "call {}",
        "jmp 9f",
        
        "2:",  // No match
        "xor ecx, ecx",
        "call {}",
        "jmp 9f",
        
        "3:",  // rax > 2 but rcx == 0 case
        "mov rcx, [rip + qword_142ED0B70]",
        "cmp [rax], rcx",
        "jnz 4f",
        "lea rcx, [rip + mainmain_str]",
        "mov edx, 4",
        "call {}",
        "jmp 9f",
        
        "4:",  // No match
        "xor ecx, ecx",
        "call {}",
        
        "9:",  // Return
        "xor eax, eax",
        "add rsp, 0x30",
        "pop rbp",
        "ret",
        
        sym sub_1404F4930,
        sym sub_1404F4930,
        sym sub_1404F4930,
        sym sub_1404F4930,
        sym sub_1404F4930
    );
}

#[repr(C)]
pub struct EXCEPTION_POINTERS {
    pub exception_record: *mut EXCEPTION_RECORD,
    pub context_record: *mut (),
}

#[repr(C)]
pub struct EXCEPTION_RECORD {
    pub exception_code: u32,
    pub exception_flags: u32,
    pub exception_record: *mut EXCEPTION_RECORD,
    pub exception_address: *mut (),
    pub number_parameters: u32,
    pub exception_information: [u64; 15],
}


#[unsafe(no_mangle)]
static mainmain_str: &[u8] = b"mainmain\0";