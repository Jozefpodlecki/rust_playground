#![feature(asm)] 

fn main() {
    let tsc: u64;

    unsafe {
        asm!(
            "rdtsc",
            "shl rdx, 32",
            "or rax, rdx",
            out("rax") tsc,
            out("rdx") _,
        );
    }

    println!("CPU Timestamp Counter: {}", tsc);
}
