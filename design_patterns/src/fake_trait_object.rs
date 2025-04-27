use std::mem;
use std::ptr;

// The trait we want to "fake"
trait Animal {
    fn speak(&self);
}

// A real struct
struct Dog {
    name: String,
}

impl Animal for Dog {
    fn speak(&self) {
        println!("Woof! I am {}", self.name);
    }
}

// A fake vtable
#[repr(C)]
struct AnimalVTable {
    speak_fn: unsafe fn(*const ()),
}

// A fake trait object layout
#[repr(C)]
struct AnimalObject {
    data_ptr: *const (),
    vtable_ptr: *const AnimalVTable,
}

unsafe fn create_animal_object<T: Animal>(val: &T) -> &'static dyn Animal {
     // We create the vtable for the Dog type
     static VTABLE: AnimalVTable = AnimalVTable {
        speak_fn: speak_impl::<Dog>,
    };

    // Box the AnimalObject (heap allocation)
    let obj = Box::new(AnimalObject {
        data_ptr: val as *const _ as *const (),
        vtable_ptr: &VTABLE as *const _,
    });

    // Convert the boxed object into a raw pointer
    let obj_ptr: *const AnimalObject = Box::into_raw(obj);

    // Safety: We assume that the raw pointer is valid and correctly cast to the correct trait object type.
    // We can safely create a reference to a dyn Animal trait object from the raw pointer.
    // This is still unsafe, but we need to do it to work with trait objects.
    let raw_ref: *const dyn Animal = &*obj_ptr;
    
    // Return the reference to the trait object
    &*raw_ref
}

// The generic dispatch function for our vtable
unsafe fn speak_impl<T: Animal>(data: *const ()) {
    let real = &*(data as *const T);
    real.speak();
}

// We provide a custom impl of Animal for our fake object
impl Animal for AnimalObject {
    fn speak(&self) {
        unsafe {
            let vtable = &*self.vtable_ptr;
            (vtable.speak_fn)(self.data_ptr);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_trait() {
        let dog = Dog { name: "Rex".to_string() };

        let fake_animal: &dyn Animal = unsafe { create_animal_object(&dog) };
    
        // Now call speak on the fake trait object
        fake_animal.speak();
    }
}
