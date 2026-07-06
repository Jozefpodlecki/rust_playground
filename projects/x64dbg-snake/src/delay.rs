
#[allow(non_snake_case)]
#[unsafe(naked)]
unsafe extern "system" fn NtDelayExecution(
    alertable: u8,
    time: *mut i64,
) -> i32 {
    core::arch::naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x34",
        "syscall",
        "ret"
    );
}

pub fn sleep_ms(millis: u64) {
    unsafe {
        let mut time = -(millis as i64) * 10_000; // 100ns intervals
        NtDelayExecution(0, &mut time);
    }
}