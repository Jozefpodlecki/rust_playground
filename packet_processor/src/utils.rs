use rand::{rng, Rng};


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