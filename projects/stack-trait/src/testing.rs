use utils::println;

use crate::{data_buf::Buf, stack_trait::Stacked};
use core::fmt;

trait Animal: fmt::Debug {
    fn speak(&self) -> &'static str;
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq)]
struct Dog {
    name: &'static str,
}

impl Dog {
    fn new(name: &'static str) -> Self {
        Dog { name }
    }
}

impl Animal for Dog {
    fn speak(&self) -> &'static str {
        "Woof!"
    }
    fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Cat {
    name: &'static str,
}

impl Cat {
    fn new(name: &'static str) -> Self {
        Cat { name }
    }
}

impl Animal for Cat {
    fn speak(&self) -> &'static str {
        "Meow!"
    }
    fn name(&self) -> &'static str {
        self.name
    }
}

pub fn test_stacked() {
    type Buf64 = Buf<u8, 64>;
    
    let obj = Stacked::<dyn Animal, Buf64 >::new(Dog::new("test")).unwrap();
    println!("capacity {}", obj.capacity());
    println!("{}", obj.speak());

    let dog1 = Stacked::<dyn Animal, Buf64>::new(Dog::new("Rex")).unwrap();
    let cat = Stacked::<dyn Animal, Buf64>::new(Cat::new("Whiskers")).unwrap();
    
    let animals: [Stacked<dyn Animal, Buf64>; 3] = [
        Stacked::new(Dog::new("Rex")).unwrap(),
        Stacked::new(Cat::new("Whiskers")).unwrap(),
        Stacked::new(Dog::new("Buddy")).unwrap(),
    ];
    
    println!("\n✅ Test 2 - Array of trait objects:");
    for (i, animal) in animals.iter().enumerate() {
        println!("   animal[{}]: {} says '{}'", i, animal.name(), animal.speak());
    }

     trait Counter: fmt::Debug {
        fn increment(&mut self);
        fn get(&self) -> i32;
    }
    
    #[derive(Debug, Clone, PartialEq)]
    struct MyCounter {
        value: i32,
    }
    
    impl MyCounter {
        fn new(value: i32) -> Self {
            MyCounter { value }
        }
    }
    
    impl Counter for MyCounter {
        fn increment(&mut self) {
            self.value += 1;
        }
        fn get(&self) -> i32 {
            self.value
        }
    }
    
    let mut counter = Stacked::<dyn Counter, Buf64>::new(MyCounter::new(0)).unwrap();
    counter.increment();
    counter.increment();
    counter.increment();

    println!("\n✅ Test 4 - DerefMut (mutable operations):");
    println!("   counter.get(): {}", counter.get());
    println!("   Expected: 3");

}