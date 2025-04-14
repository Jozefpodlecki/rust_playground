
use rand::{distr::{uniform::{SampleRange, SampleUniform}, Alphanumeric}, rng, Rng};

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

pub fn random_alphabetic_string(length: usize) -> String {
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rng();
    
    (0..length)
        .map(|_| charset[rng.random_range(0..charset.len())] as char)
        .collect()
}

pub fn random_alphanumeric_string(length: usize) -> String {
    let rng = rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    random_string
}

pub fn random_number_in_range<T: SampleUniform, R: SampleRange<T>>(range: R) -> T {
    let mut rng = rng();
    rng.random_range(range)
}

pub enum Action {
    Send,
    Write,
    Inspect
}

fn select_action(actions: &[(Action, f64)]) -> &Action {
    let mut rng = rng();
    let result = rng.random_range(0.0..=1.0);

    let mut cumulative_probability = 0.0;

    for (action, probability) in actions {
        cumulative_probability += *probability;
        if result < cumulative_probability {
            return action
        }
    }

    &actions.last().unwrap().0
}
