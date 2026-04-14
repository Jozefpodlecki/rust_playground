use core::arch::{asm, naked_asm};
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

#[unsafe(no_mangle)]
pub static mut __security_cookie: u64 = 0x2B992DDFA232;

#[unsafe(no_mangle)]
pub static mut __security_cookie_complement: u64 = 0;

// CRT initialization state
#[unsafe(no_mangle)]
pub static mut dword_142ED69B0: u32 = 0;  // startup state: 0=uninit, 1=initializing, 2=initialized

#[unsafe(no_mangle)]
pub static mut byte_142ED69C0: u8 = 0;    // CRT initialized flag

#[unsafe(no_mangle)]
pub static mut qword_142ED69B8: u64 = 0;  // startup lock

#[link(name = "kernel32")]
unsafe extern "system" {}

unsafe extern "system" {
    fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut u64);
    fn GetCurrentThreadId() -> u32;
    fn GetCurrentProcessId() -> u32;
    fn QueryPerformanceCounter(lpPerformanceCount: *mut u64) -> i32;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = c as u8;
        i += 1;
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn __CxxFrameHandler3() -> ! {
    panic!("C++ exception thrown");
}

#[link(name = "ucrt")]
unsafe extern "system" {
    fn _exit(code: i32) -> !;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __security_init_cookie() {
    asm!(
        "push rbp",
        "mov rbp, rsp",
        "sub rsp, 30h",
        
        "mov [rbp+0x10], rbx",
        
        "lea rdx, [rip + __security_cookie]",
        "mov rax, [rdx]",
        "mov rbx, 0x2B992DDFA232",
        "cmp rax, rbx",
        "jnz 2f",
        
        "and qword ptr [rbp-0x8], 0",
        "lea rcx, [rbp-0x8]",
        "call {}",
        
        "mov rax, [rbp-0x8]",
        "mov [rbp-0x10], rax",
        
        "call {}",
        "mov eax, eax",
        "xor [rbp-0x10], rax",
        
        "call {}",
        "mov eax, eax",
        "xor [rbp-0x10], rax",
        
        "lea rcx, [rbp-0x18]",
        "call {}",
        "mov eax, [rbp-0x18]",
        "lea rcx, [rbp-0x10]",
        "shl rax, 0x20",
        "xor rax, [rbp-0x18]",
        "xor rax, [rbp-0x10]",
        "xor rax, rcx",
        
        "mov rcx, 0xFFFFFFFFFFFF",
        "and rax, rcx",
        "mov rcx, 0x2B992DDFA233",
        "cmp rax, rbx",
        "cmovz rax, rcx",
        "lea rdx, [rip + __security_cookie]",
        "mov [rdx], rax",
        
        "2:",
        "not rax",
        "lea rdx, [rip + __security_cookie_complement]",
        "mov [rdx], rax",
        "mov rbx, [rbp+0x10]",
        
        "add rsp, 0x30",
        "pop rbp",
        "ret",
        sym GetSystemTimeAsFileTime,
        sym GetCurrentThreadId,
        sym GetCurrentProcessId,
        sym QueryPerformanceCounter,
        options(nostack),
    );
}

// __scrt_initialize_crt - initializes the C runtime
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn __scrt_initialize_crt() -> u8 {
    naked_asm!(
        "sub rsp, 0x28",
        "test ecx, ecx",
        "jnz 1f",
        "mov byte ptr [rip + byte_142ED69C0], 1",
        "1:",
        "call {}",      // sub_140774F00 - internal CRT init
        "call {}",      // __vcrt_initialize
        "test al, al",
        "jnz 2f",
        "xor al, al",
        "add rsp, 0x28",
        "ret",
        "2:",
        "call {}",      // sub_140775884
        "test al, al",
        "jnz 3f",
        "xor ecx, ecx",
        "call {}",      // __vcrt_uninitialize
        "xor al, al",
        "add rsp, 0x28",
        "ret",
        "3:",
        "mov al, 1",
        "add rsp, 0x28",
        "ret",
        sym sub_140774F00,
        sym __vcrt_initialize,
        sym sub_140775884,
        sym __vcrt_uninitialize,
    );
}

// __scrt_acquire_startup_lock - acquires the startup lock
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn __scrt_acquire_startup_lock() -> u8 {
    naked_asm!(
        "sub rsp, 0x28",
        "call {}",      // __scrt_is_ucrt_dll_in_use
        "test eax, eax",
        "jz 2f",
        "mov rax, gs:[0x30]",
        "mov rcx, [rax+8]",
        "1:",
        "xor eax, eax",
        "lock cmpxchg qword ptr [rip + qword_142ED69B8], rcx",
        "jnz 3f",
        "2:",
        "xor al, al",
        "add rsp, 0x28",
        "ret",
        "3:",
        "cmp rcx, rax",
        "jz 4f",
        "jmp 1b",
        "4:",
        "mov al, 1",
        "add rsp, 0x28",
        "ret",
        sym __scrt_is_ucrt_dll_in_use,
    );
}

// __scrt_release_startup_lock - releases the startup lock
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn __scrt_release_startup_lock(_cl: u8) {
    naked_asm!(
        "sub rsp, 0x28",
        "call {}",      // __scrt_is_ucrt_dll_in_use
        "test eax, eax",
        "jz 1f",
        "mov qword ptr [rip + qword_142ED69B8], 0",
        "1:",
        "add rsp, 0x28",
        "ret",
        sym __scrt_is_ucrt_dll_in_use,
    );
}

// _initterm - runs through a table of function pointers
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _initterm(first: *const *const fn(), last: *const *const fn()) {
    let mut current = first;
    while current < last {
        let func_ptr = unsafe { *current };
        if !func_ptr.is_null() {
            unsafe { (*func_ptr)(); }  // Dereference the pointer first
        }
        current = unsafe { current.add(1) };
    }
}

// _initterm_e - runs through a table of function pointers, returns error code
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _initterm_e(first: *const *const fn() -> i32, last: *const *const fn() -> i32) -> i32 {
    let mut current = first;
    while current < last {
        let func_ptr = unsafe { *current };
        if !func_ptr.is_null() {
            let result = unsafe { (*func_ptr)() };  // Dereference the pointer first
            if result != 0 {
                return result;
            }
        }
        current = unsafe { current.add(1) };
    }
    0
}

// __scrt_fastfail - fast fail with a code
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn __scrt_fastfail(code: u32) -> ! {
    naked_asm!(
        "sub rsp, 0x28",
        "mov ecx, ecx",     // code is already in ecx
        "call {}",          // __fastfail
        sym __fastfail,
    );
}

// __fastfail - Windows fastfail syscall
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn __fastfail(code: u32) -> ! {
    naked_asm!(
        "int 0x29",
    );
}

// __scrt_is_managed_app - returns 0 (not managed)
#[unsafe(no_mangle)]
pub extern "C" fn __scrt_is_managed_app() -> u8 {
    0
}

// __scrt_is_ucrt_dll_in_use - returns 0 (not using UCRT as DLL)
#[unsafe(no_mangle)]
pub extern "C" fn __scrt_is_ucrt_dll_in_use() -> i32 {
    0
}

// __vcrt_initialize - returns 1 (success)
#[unsafe(no_mangle)]
pub extern "C" fn __vcrt_initialize() -> u8 {
    1
}

// __vcrt_uninitialize
#[unsafe(no_mangle)]
pub extern "C" fn __vcrt_uninitialize(_terminating: u32) {}

// sub_140774F00 - internal CRT initialization (stub)
#[unsafe(no_mangle)]
pub extern "C" fn sub_140774F00() {}

// sub_140775884 - internal CRT initialization (stub)
#[unsafe(no_mangle)]
pub extern "C" fn sub_140775884() -> u8 {
    1
}

// __scrt_is_nonwritable_in_current_image
#[unsafe(no_mangle)]
pub extern "C" fn __scrt_is_nonwritable_in_current_image(_ptr: *const ()) -> u8 {
    0
}

// _register_thread_local_exe_atexit_callback
#[unsafe(no_mangle)]
pub extern "C" fn _register_thread_local_exe_atexit_callback(_callback: *const ()) {}

// _get_initial_narrow_environment
#[unsafe(no_mangle)]
pub extern "C" fn _get_initial_narrow_environment() -> *mut *mut u8 {
    core::ptr::null_mut()
}

// __p___argv - returns pointer to argv
#[unsafe(no_mangle)]
pub static mut __argv: *mut *mut u8 = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub extern "C" fn __p___argv() -> *mut *mut *mut u8 {
    unsafe { &raw mut __argv } 
}

#[unsafe(no_mangle)]
pub static mut __argc: i32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn __p___argc() -> *mut i32 {
    unsafe { &raw mut __argc }
}

// _cexit - calls exit-time functions
#[unsafe(no_mangle)]
pub extern "C" fn _cexit() {}

// _c_exit - quick exit without flushing buffers
#[unsafe(no_mangle)]
pub extern "C" fn _c_exit() {}

// exit - normal exit (calls atexit functions)
#[unsafe(no_mangle)]
pub extern "C" fn exit(code: i32) -> ! {
    unsafe { _exit(code) }
}

#[unsafe(no_mangle)]
pub extern "C" fn __scrt_uninitialize_crt(_unknown: u32, _unknown2: u8) {}

unsafe extern "C" {
    pub static __guard_dispatch_icall_fptr: *const ();
}

#[repr(transparent)]
pub struct SyncPtr<T>(*const T);
unsafe impl<T> Sync for SyncPtr<T> {}

static EMPTY_FUNC_TABLE: SyncPtr<fn()> = SyncPtr(core::ptr::null());

#[unsafe(no_mangle)]
pub extern "C" fn sub_1407758B8() -> *const *const fn() {
    unsafe { &raw const EMPTY_FUNC_TABLE.0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn sub_1407758C0() -> *const *const fn() {
    unsafe { &raw const EMPTY_FUNC_TABLE.0 }
}

#[repr(transparent)]
pub struct InitTermPtr(*const fn());
unsafe impl Sync for InitTermPtr {}

#[unsafe(no_mangle)]
pub static First: InitTermPtr = InitTermPtr(core::ptr::null());

#[unsafe(no_mangle)]
pub static Last: InitTermPtr = InitTermPtr(core::ptr::null());

#[unsafe(no_mangle)]
pub static qword_1407AB0E8: InitTermPtr = InitTermPtr(core::ptr::null());

#[unsafe(no_mangle)]
pub static qword_1407AB120: InitTermPtr = InitTermPtr(core::ptr::null());

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe fn __scrt_common_main_seh() -> ! {
    naked_asm!(
        // Save non-volatile registers
        "mov [rsp+0x8], rbx",
        "mov [rsp+0x10], rsi",
        "push rdi",
        "sub rsp, 0x30",
        
        // Initialize CRT
        "mov ecx, 1",
        "call __scrt_initialize_crt",
        "test al, al",
        "jz 9f",
        
        "xor sil, sil",
        "mov [rsp+0x20], sil",
        
        // Acquire startup lock
        "call __scrt_acquire_startup_lock",
        "mov bl, al",
        
        // Check initialization state
        "cmp dword ptr [rip + dword_142ED69B0], 1",
        "jz 8f",
        "cmp dword ptr [rip + dword_142ED69B0], 0",
        "jnz 1f",
        
        // First initialization
        "mov dword ptr [rip + dword_142ED69B0], 1",
        "lea rdx, [rip + Last]",
        "lea rcx, [rip + First]",
        "call _initterm_e",
        "test eax, eax",
        "jz 2f",
        "mov eax, 0xFF",
        "jmp 7f",
        
        "2:",
        "lea rdx, [rip + qword_1407AB120]",
        "lea rcx, [rip + qword_1407AB0E8]",
        "call _initterm",
        "mov dword ptr [rip + dword_142ED69B0], 2",
        "jmp 3f",
        
        // Already initialized
        "1:",
        "mov sil, 1",
        "mov [rsp+0x20], sil",
        
        "3:",
        // Release startup lock
        "mov cl, bl",
        "call __scrt_release_startup_lock",
        
        // Call pre-main initialization
        "call sub_1407758B8",
        "mov rbx, rax",
        "cmp qword ptr [rax], 0",
        "jz 4f",
        "mov rcx, rax",
        "call __scrt_is_nonwritable_in_current_image",
        "test al, al",
        "jz 4f",
        "xor r8d, r8d",
        "lea edx, [r8+2]",
        "xor ecx, ecx",
        "mov rax, [rbx]",
        "call qword ptr [rip + __guard_dispatch_icall_fptr]",
        
        "4:",
        // Thread-local initialization
        "call sub_1407758C0",
        "mov rbx, rax",
        "cmp qword ptr [rax], 0",
        "jz 5f",
        "mov rcx, rax",
        "call __scrt_is_nonwritable_in_current_image",
        "test al, al",
        "jz 5f",
        "mov rcx, [rbx]",
        "call _register_thread_local_exe_atexit_callback",
        
        "5:",
        "call _get_initial_narrow_environment",
        "mov rdi, rax",
        "call __p___argv",
        "mov rbx, [rax]",
        "call __p___argc",
        "mov r8, rdi",
        "mov rdx, rbx",
        "mov ecx, [rax]",
        "call main",
        "mov ebx, eax",
        
        // Check if managed app
        "call __scrt_is_managed_app",
        "test al, al",
        "jz 6f",
        "test sil, sil",
        "jnz 7f",
        "call _cexit",
        
        "7:",
        "xor edx, edx",
        "mov cl, 1",
        "call __scrt_uninitialize_crt",
        "mov eax, ebx",
        "jmp 7f",
        
        // Fastfail cases
        "8:",
        "mov ecx, 7",
        "call __scrt_fastfail",
        
        "9:",
        "mov ecx, 7",
        "call __scrt_fastfail",
        
        "6:",
        "mov ecx, ebx",
        "call exit",
        
        "7:",
        // Epilogue
        "mov rbx, [rsp+0x38]",
        "mov rsi, [rsp+0x40]",
        "add rsp, 0x30",
        "pop rdi",
        "ret"
    );
}