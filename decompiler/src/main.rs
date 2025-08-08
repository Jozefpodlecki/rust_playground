#![allow(warnings)]

use std::{fmt::format, fs::{self, File}, io::Write, path::PathBuf};
use anyhow::*;
use crate::decompiler::Decompiler;

mod decompiler;

fn main() -> Result<()> {
    let path = r"C:\repos\rust_playground\a.exe";
    let content = fs::read(path)?;

    let input_path = PathBuf::from(path);
    let file_name = input_path.file_stem().unwrap().to_string_lossy();
    let output_path = format!("{}.txt", file_name);
    let mut decompiler = Decompiler::new();
    let decompiled = decompiler.run(&content);

    let mut file = File::create(output_path)?;
    file.write_all(&decompiled)?;

    Ok(())
}