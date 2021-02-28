mod factorial;

fn main() {
    let number = 10;
    let result = factorial::factorial_v1(number);

    println!("Factorial {} {}", number, result);
}
