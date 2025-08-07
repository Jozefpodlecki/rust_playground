use std::{collections::HashMap, fs::File, path::PathBuf};

use anyhow::*;
use walkdir::WalkDir;
use crate::processor::ProcessorStep;

const SUBST_TABLE: &[(char, [&'static str; 4])] = &[
    ('Q', ["QP", "QD", "QW", "Q4"]),
    ('-', ["QL", "QB", "QO", "Q5"]),
    ('_', ["QC", "QN", "QT", "Q9"]),
    ('X', ["XU", "XN", "XH", "X3"]),
    ('!', ["XW", "XS", "XZ", "X0"]),
];

/// Character substitutions for deobfuscation purposes.
const SUBST_TABLE_REV: &[(&'static str, char, usize)] = &[
    // Q
    ("QP", 'Q', 0),
    ("QD", 'Q', 1),
    ("QW", 'Q', 2),
    ("Q4", 'Q', 3),
    // -
    ("QL", '-', 0),
    ("QB", '-', 1),
    ("QO", '-', 2),
    ("Q5", '-', 3),
    // _
    ("QC", '_', 0),
    ("QN", '_', 1),
    ("QT", '_', 2),
    ("Q9", '_', 3),
    // X
    ("XU", 'X', 0),
    ("XN", 'X', 1),
    ("XH", 'X', 2),
    ("X3", 'X', 3),
    // !
    ("XW", '!', 0),
    ("XS", '!', 1),
    ("XZ", '!', 2),
    ("X0", '!', 3),
];

pub struct DecryptUpkStep {
    path: PathBuf,
    decrypted_map_path: PathBuf,
}

impl ProcessorStep for DecryptUpkStep {
    fn name(&self) -> String {
        format!("Decrypt Upk in {:?}", self.path)
    }

    fn can_execute(&self) -> bool {
        if !self.path.exists() {
            return false
        }

        if self.decrypted_map_path.exists() {
            return false
        }

        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let writer = File::create(self.decrypted_map_path)?;
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        let file_paths = WalkDir::new(&self.path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file()
                && e.path().extension().filter(|&pr| pr == "upk").is_some())
            .map(|e| e.path().to_path_buf());

        for file_path in file_paths {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            let items = map.entry(file_name.to_string()).or_default();

            let deobfuscated = deobfuscate_string(&file_name);
            items.push(deobfuscated);
            
            let deobfuscated = decrypt(&file_name);
            items.push(deobfuscated);

            // info!("{} -> {}", file_name, deobfuscated);
        }

        serde_json::to_writer_pretty(writer, &map)?;

        Ok(())
    }
}

impl DecryptUpkStep {
    pub fn new(path: PathBuf) -> Self {
        let decrypted_map_path = path.join("decrypted.json");

        Self {
            path,
            decrypted_map_path
        }
    }
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

/// Escape the given string by replacing all characters in SUBST_TABLE.
pub fn escape_string(input: &str) -> String {
    let input = input.to_uppercase();
    let mut out = String::new();

    for c in input.chars() {
        let subst = SUBST_TABLE.iter().find(|(c2, _)| *c2 == c);
        if let Some((_, subst)) = subst {
            out.push_str(subst[out.len() % subst.len()]);
        } else {
            out.push(c);
        }
    }

    out
}

/// Unescape the given string by checking whether every pair of characters
/// is in SUBST_TABLE_REV.
pub fn unescape_string(input: &str) -> String {
    let input = input.to_uppercase();
    let mut out = String::new();

    let mut i = 0;
    while i < input.len() {
        let subst = SUBST_TABLE_REV
            .iter()
            .find(|(c2, _, _)| input[i..].starts_with(c2));

        match subst {
            Some(&(str, c, pos)) if i % 4 == pos => {
                out.push(c);
                i += str.len();
            }
            _ => {
                out.push(input.chars().nth(i).unwrap());
                i += 1;
            }
        }
    }

    out
}

/// Obfuscate the given string.
pub fn obfuscate_string(orig_input: &str) -> String {
    let mut input = escape_string(orig_input);
    let unpadded_length = orig_input.len();

    // if the input is less than 20 characters, pad with ......
    if unpadded_length < 20 {
        // escape leaves . intact but replaces ! with relevant char
        input =
            escape_string(&(orig_input.to_string() + "!" + &".".repeat(20 - unpadded_length - 1)));
    }
    let length = input.len() as i32;

    let mut out = String::new();
    let mut chars: Vec<_> = input.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        let mut x = c as u8 as i32;
        if c >= '0' && c <= '9' {
            x += 43; // wrap up if numerical
        }

        let mut new_c = (length + 7i32 * (x - 65)) % 36 + 65;
        if new_c >= 91 {
            new_c -= 43; // wrap back down to numerical
        }
        let new_c = new_c as u8 as char;
        out.push(new_c);

        if i + unpadded_length < chars.len() && chars[i + unpadded_length] == '.' {
            // replace placeholder .s with this character, they'll get obfuscated later
            chars[i + unpadded_length] = new_c;
        }
    }

    out
}

/// Deobfuscate the given string.
pub fn deobfuscate_string(input: &str) -> String {
    let input = input.to_uppercase();
    let mut out = String::new();

    for c in input.chars() {
        let mut x = c as u8 as i32;
        if c >= '0' && c <= '9' {
            x += 43; // wrap up if numerical
        }

        let mut i = (31 * (x - input.len() as i32 - 65) % 36 + 36) % 36 + 65;
        if i >= 91 {
            i -= 43; // wrap back down to numerical
        }
        out.push(i as u8 as char);
    }

    // strip out !<padding> if needed
    let unescaped = unescape_string(&out);
    if unescaped.contains("!") {
        unescaped.split("!").next().unwrap().to_string()
    } else {
        unescaped
    }
}