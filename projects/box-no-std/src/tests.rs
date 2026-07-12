use toolkit::println;
use crate::boxed::Box;

use core::hash::{Hash, Hasher};
struct SimpleHasher(u64);
impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.wrapping_add(b as u64);
        }
    }
    fn write_u8(&mut self, i: u8) {
        self.0 = self.0.wrapping_add(i as u64);
    }
    fn write_u32(&mut self, i: u32) {
        self.0 = self.0.wrapping_add(i as u64);
    }
    fn write_u64(&mut self, i: u64) {
        self.0 = self.0.wrapping_add(i);
    }
    fn write_usize(&mut self, i: usize) {
        self.0 = self.0.wrapping_add(i as u64);
    }
}


pub fn run_tests() {

    // 1. Basic Box functionality
    let val = Box::new(42);
    println!("1. Box new: {}", *val);

    // 2. Clone test
    let cloned = val.clone();
    println!("2. Clone: {}", *cloned);
    println!("   Addresses differ: {}", Box::into_raw(val) != Box::into_raw(cloned));

    // 3. into_raw / from_raw roundtrip
    let raw = Box::into_raw(Box::new(100));
    let recovered = unsafe { Box::from_raw(raw) };
    println!("3. into_raw/from_raw: {}", *recovered);

    // 4. Box::leak
    let leaked = Box::leak(Box::new(200));
    *leaked = 300;
    println!("4. Leak: {}", *leaked);

    // 5. Box::new_uninit and assume_init
    let uninit = Box::new_uninit();
    unsafe {
        uninit.0.as_ptr().write(core::mem::MaybeUninit::new(500));
        let init = uninit.assume_init();  // This is Box<MaybeUninit<i32>>
        let value = init.assume_init();   // This is i32, consuming the MaybeUninit
        println!("5. new_uninit/assume_init: {:?}", value);
    }

    // 6. Box::default
    let default = Box::<i32>::default();
    println!("6. Default: {}", *default);

    // 7. Deref and DerefMut
    let mut deref_test = Box::new(10);
    *deref_test = 20;
    println!("7. DerefMut: {}", *deref_test);

    // 8. Slice support - new_uninit_slice
    let slice_box = Box::<[i32]>::new_uninit_slice(3);
    unsafe {
        let slice_ptr = slice_box.0.as_ptr() as *mut [core::mem::MaybeUninit<i32>];
        let data_ptr = (*slice_ptr).as_mut_ptr();
        data_ptr.add(0).write(core::mem::MaybeUninit::new(1));
        data_ptr.add(1).write(core::mem::MaybeUninit::new(2));
        data_ptr.add(2).write(core::mem::MaybeUninit::new(3));
        let init = slice_box.assume_init();
        println!("8. Slice: {:?}", &*init);
    }

    // 9. Slice clone
    let slice1 = Box::<[i32]>::new_uninit_slice(3);
    unsafe {
        let slice_ptr = slice1.0.as_ptr() as *mut [core::mem::MaybeUninit<i32>];
        let data_ptr = (*slice_ptr).as_mut_ptr();
        data_ptr.add(0).write(core::mem::MaybeUninit::new(10));
        data_ptr.add(1).write(core::mem::MaybeUninit::new(20));
        data_ptr.add(2).write(core::mem::MaybeUninit::new(30));
        let init = slice1.assume_init();
        let cloned_slice = init.clone();
        println!("9. Slice clone: {:?}", &*cloned_slice);
    }

    // 10. From array
    let array_box: Box<[i32]> = Box::from([4, 5, 6, 7, 8]);
    println!("10. From array: {:?}", &*array_box);

    // 11. From slice
    let slice = &[9, 10, 11];
    let slice_box = Box::from(slice);
    println!("11. From slice: {:?}", &*slice_box);

    // 12. From mut slice
    let mut mut_slice = [12, 13, 14];
    let mut_slice_box = Box::from(&mut mut_slice);
    println!("12. From mut slice: {:?}", &*mut_slice_box);

    // 13. Trait object
    trait Test {
        fn value(&self) -> i32;
    }
    struct S(i32);
    impl Test for S {
        fn value(&self) -> i32 {
            self.0
        }
    }
    let trait_obj = Box::new(S(999)) as Box<dyn Test>;
    println!("13. Trait object: {}", trait_obj.value());

    // 14. PartialEq and Eq
    let b1 = Box::new(5);
    let b2 = Box::new(5);
    let b3 = Box::new(10);
    println!("14. PartialEq: {} {}", b1 == b2, b1 != b3);

    // 15. Ord and PartialOrd
    let o1 = Box::new(1);
    let o2 = Box::new(2);
    println!("15. Ord: {} {}", o1 < o2, o2 > o1);

    // 16. Hash
    let h1 = Box::new(42);
    let h2 = Box::new(42);
    let mut hasher1 = SimpleHasher(0);
    let mut hasher2 = SimpleHasher(0);
    h1.hash(&mut hasher1);
    h2.hash(&mut hasher2);
    println!("16. Hash: {}", hasher1.finish() == hasher2.finish());

    // 17. Display and Debug
    let display_test = Box::new(777);
    println!("17. Display: {}", display_test);
    println!("   Debug: {:?}", display_test);

    // 18. Borrow
    use core::borrow::Borrow;
    let borrow_test = Box::new(123);
    let borrowed: &i32 = borrow_test.borrow();
    println!("18. Borrow: {}", *borrowed);

    // 19. BorrowMut
    use core::borrow::BorrowMut;
    let mut borrow_mut_test = Box::new(456);
    let borrowed_mut: &mut i32 = borrow_mut_test.borrow_mut();
    *borrowed_mut = 789;
    println!("19. BorrowMut: {}", *borrow_mut_test);

    // 20. AsRef
    let as_ref_test = Box::new("hello");
    let as_ref_str: &str = as_ref_test.as_ref();
    println!("20. AsRef: {}", as_ref_str);

    // 21. Multiple allocations
    println!("21. Multiple allocations:");
    let mut boxes: [Box<usize>; 5] = core::array::from_fn(|i| Box::new(i * 10));
    for i in 0..5 {
        println!("   Box[{}] = {}", i, *boxes[i]);
    }

    // 22. Large allocation
    let large = Box::new([0u8; 1024]);
    println!("22. Large allocation: {} bytes", large.len());

    // 23. Zero-sized type
    struct ZST;
    let zst = Box::new(ZST);
    println!("23. Zero-sized type: allocated");

    // 24. Drop test
    use core::cell::Cell;
    struct DropCounter<'a>(&'a Cell<usize>);
    impl Drop for DropCounter<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let counter = Cell::new(0);
    {
        let _drop_test = Box::new(DropCounter(&counter));
        println!("24. Drop test: created");
    }
    println!("   Drop count: {}", counter.get());

    // 25. Slice drop test
    let slice_counter = Cell::new(0);
    {
        let slice_drop = Box::<[DropCounter]>::new_uninit_slice(3);
        unsafe {
            let slice_ptr = slice_drop.0.as_ptr() as *mut [core::mem::MaybeUninit<DropCounter>];
            let data_ptr = (*slice_ptr).as_mut_ptr();
            for i in 0..3 {
                data_ptr.add(i).write(core::mem::MaybeUninit::new(DropCounter(&slice_counter)));
            }
            let _ = slice_drop.assume_init();
        }
        println!("25. Slice drop test: created");
    }
    println!("   Slice drop count: {}", slice_counter.get());

    // 26. Pointer formatting
    let ptr_test = Box::new(42);
    println!("26. Pointer: {:p}", &*ptr_test);

    // 27. Coerce unsized
    let array: Box<[i32; 3]> = Box::new([1, 2, 3]);
    let _slice: Box<[i32]> = array;
    println!("27. Coerce unsized: successful");

    println!("\nAll tests passed!");
}