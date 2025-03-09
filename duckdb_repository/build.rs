
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
    let dest_dir = target_dir.join("migrations");

    let src_dir = Path::new("src/migrations");

    if dest_dir.exists() {
        for entry in fs::read_dir(&dest_dir).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            if entry_path.is_file() {
                fs::remove_file(entry_path).unwrap();
            }
        }
    }
    else {
        fs::create_dir_all(&dest_dir).unwrap();
    }

    println!("cargo:rerun-if-changed={}", src_dir.display());
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let src_file = entry.path();
        let dest_file = dest_dir.join(entry.file_name());

        fs::copy(&src_file, &dest_file).unwrap();
    }
}