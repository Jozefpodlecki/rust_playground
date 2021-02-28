
pub fn factorial_v1(number: i64) -> i64 {
    let mut result = 1i64;

    for n in 2..number {
        result = result * n;
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_factorial() {
        let result = factorial_v1(1);

        assert!(result == 1);
    }
}