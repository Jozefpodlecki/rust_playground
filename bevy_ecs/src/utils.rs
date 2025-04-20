use rand::{rng, Rng};

pub fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}b", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}m", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn random_alphabetic_string_capitalized(length: usize) -> String {
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rng();
    
    let mut result: String = (0..length)
        .map(|_| charset[rng.random_range(0..charset.len())] as char)
        .collect();

    let first_char = result.get_mut(0..1).unwrap();
    first_char.make_ascii_uppercase();

    result
}
