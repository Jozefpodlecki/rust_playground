use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;
use crate::{lpk::{get_lpks, LpkInfo}, types::RunArgs};




pub fn extract_lpk(args: RunArgs) -> Result<()> {
    let output_path = args.output_path.clone();

    for mut lpk_info in get_lpks(&args)? {
 
        let lpk_name = lpk_info.file_path.file_stem().unwrap().to_str().unwrap();
        let output_path = output_path.join(lpk_name);
        info!("Creating directory {}", output_path.to_str().unwrap());
        fs::create_dir_all(&output_path)?;

        lpk_info.load()?;

        for entry in lpk_info.get_entries().iter_mut() {
            let content = entry.content.to_bytes()?;
            let file_path = &entry.metadata.file_path;
            let output_path = output_path.join(file_path);
            fs::create_dir_all(&output_path.parent().unwrap())?;
            save_to_disk(&output_path, &content)?;
        }
    }

    Ok(())
}

pub fn save_to_disk(output_path: &Path, content: &[u8]) -> Result<()> {
    info!("Saving to {}", output_path.to_str().unwrap());

    let mut file = File::create(&output_path)?;
    file.write_all(&content)?;

    Ok(())
}

pub fn collect_lpk_paths<P: AsRef<Path>>(lpk_dir: P) -> Result<Vec<PathBuf>> {
    let mut lpk_paths = Vec::new();

    for entry in fs::read_dir(lpk_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("lpk") {
            lpk_paths.push(path);
        }
    }

    Ok(lpk_paths)
}

fn clean(source: &str) -> String {
    let mut out_str = String::new();
    let source = source.to_uppercase();
    let key_table = [
        ("QP", 'Q', 0), ("QD", 'Q', 1), ("QW", 'Q', 2), ("Q4", 'Q', 3),
        ("QL", '-', 0), ("QB", '-', 1), ("QO", '-', 2), ("Q5", '-', 3),
        ("QC", '_', 0), ("QN", '_', 1), ("QT", '_', 2), ("Q9", '_', 3),
        ("XU", 'X', 0), ("XN", 'X', 1), ("XH", 'X', 2), ("X3", 'X', 3),
        ("XW", '!', 0), ("XS", '!', 1), ("XZ", '!', 2), ("X0", '!', 3),
    ];

    let mut i = 0;
    let chars = source.chars().collect::<Vec<_>>();

    while i < chars.len() {
        let rest: String = chars[i..].iter().collect();
        let subst = key_table.iter().find(|(key, _, pos)| {
            rest.starts_with(*key) && (i % 4 == *pos)
        });

        if let Some((_, replacement, key_len)) = subst {
            out_str.push(*replacement);
            i += 2;
        } else {
            out_str.push(chars[i]);
            i += 1;
        }
    }

    out_str
}

pub fn decrypt(source: &str) -> String {
    let source = source.to_uppercase();
    let mut out_str = String::new();

    for c in source.chars() {
        let mut x = c as i32;

        if c >= '0' && c <= '9' {
            x += 43;
        }

        let len = source.len() as i32;
        let mut i = (31 * (x - len - 65) % 36 + 36) % 36 + 65;
        if i >= 91 {
            i -= 43;
        }

        out_str.push(i as u8 as char);
    }

    let cleaned = clean(&out_str);
    cleaned.split('!').next().unwrap_or("").to_string()
}