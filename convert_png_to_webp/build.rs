use std::env;
use std::fs;
use std::path::PathBuf;

macro_rules! build_println {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let target_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    build_println!("Out dir {:?}", target_dir);
    let output_dir = format!("{}/target/debug", target_dir); 

    // let out_dir = env::var("OUT_DIR").unwrap();
    // build_println!("Out dir {:?}", out_dir);

    let src = "assets/image.png";
    let dest = PathBuf::from(output_dir).join("image.png");

    fs::copy(src, dest).expect("Failed to copy file");

    println!("cargo:rerun-if-changed={}", src);
}