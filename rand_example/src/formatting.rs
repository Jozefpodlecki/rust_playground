
fn round_to_x_decimal(num: f64, value: f64) -> f64 {
    (num * value).round() / value
}

pub fn format_example() {
    let num = 3.141592;

    println!("Rounded: {}", round_to_x_decimal(num, 10.0));
    println!("Rounded: {}", round_to_x_decimal(num, 100.0));
    println!("Rounded: {}", round_to_x_decimal(num, 1000.0));

    let rounded_1_decimal = format!("{:.1}", num);
    let rounded_2_decimal = format!("{:.2}", num);
    let rounded_3_decimal = format!("{:.3}", num);

    println!("Rounded: {}", rounded_1_decimal);
    println!("Rounded: {}", rounded_2_decimal);
    println!("Rounded: {}", rounded_3_decimal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        
        format_example();
    }
}