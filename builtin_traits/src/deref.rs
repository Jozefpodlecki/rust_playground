use std::ops::{Deref, DerefMut};

struct Wrapper<T>(T);

impl<T> Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Wrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let wrapped = Wrapper(String::from("abc"));
        println!("{}", wrapped.len());

        let mut wrapped_mut = Wrapper(String::from("def"));
        *wrapped_mut += "ghi";
    }
}