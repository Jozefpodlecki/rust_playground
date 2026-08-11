pub trait Reg {
    const ENC: u8;
    const REX: u8;
}

macro_rules! regs {
    ($($reg:ident => $enc:expr),* $(,)?) => {
        $(
            pub struct $reg;
            impl Reg for $reg {
                const ENC: u8 = $enc;
                const REX: u8 = if $enc >= 8 { 0x01 } else { 0 };
            }
        )*
    };
}

regs! {
    Rax => 0, Rbx => 3, Rcx => 1, Rdx => 2,
    Rsi => 6, Rdi => 7, Rbp => 5, Rsp => 4,
    R8 => 8, R9 => 9, R10 => 10, R11 => 11,
    R12 => 12, R13 => 13, R14 => 14, R15 => 15,
}