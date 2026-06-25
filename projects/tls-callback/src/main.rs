use utils::println;

unsafe extern "system" fn tls_callback(_h: *const (), _dw_reason: u32, _pv: *const ()) {
    println!("in tls_callback");
}

#[unsafe(link_section = ".CRT$XLB")]
#[used]
pub static CALLBACK: unsafe extern "system" fn(*const (), u32, *const ()) = tls_callback;


fn main() {
    println!("in mainCRTStartup");
}