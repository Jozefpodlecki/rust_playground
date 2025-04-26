fn raw_pointer_to_array_elements() {
    let arr = [10, 20, 30, 40];
    let ptr = arr.as_ptr();
    unsafe {
        println!("First element: {}", *ptr);
    }
}

fn loop_using_ptr() {
    let arr = [10, 20, 30, 40];
    let ptr = arr.as_ptr();
    let len = arr.len();

    unsafe {
        let mut current_ptr = ptr;
        for _ in 0..len {
            println!("Element: {}", *current_ptr);
            current_ptr = current_ptr.add(1);
        }
    }
}

fn pointer_to_reference() {
    let num = 10;
    let ptr = &num as *const i32;
    unsafe {
        let ref_num = &*ptr;
        println!("Dereferenced pointer: {}", ref_num);
    }
}

struct MyStruct {
    a: i32,
    b: i32,
}

fn modify_struct_elements() {
    let mut my_struct = MyStruct { a: 10, b: 20 };
    let ptr = &mut my_struct as *mut MyStruct;

    println!("Initial: a = {}, b = {}", my_struct.a, my_struct.b);

    unsafe {
        let ptr_mut_ref = &mut *ptr;
        ptr_mut_ref.a = 50;
        ptr_mut_ref.b = 100;
    }

    println!("Modified struct: a = {}, b = {}", my_struct.a, my_struct.b);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        modify_struct_elements()
        // pointer_to_reference()
        // raw_pointer_to_array_elements();
        // loop_using_ptr();
        //pointer_to_reference()
    }
}