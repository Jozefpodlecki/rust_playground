use std::{path::PathBuf, process::Command};

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let root = out_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap();

    let root_str = root.to_str().unwrap();

    let status = Command::new("mc.exe")
        .args(["provider.man", "-r", root_str, "-h", root_str])
        .status()
        .expect("failed to run mc.exe");

    let target_path = root.join("provider.rc");

    embed_resource::compile(target_path, embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}