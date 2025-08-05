
fn main() {
    println!("cargo:rustc-link-search=native=C:\\Windows\\System32");
    println!("cargo:rustc-link-lib=Rstrtmgr");
}