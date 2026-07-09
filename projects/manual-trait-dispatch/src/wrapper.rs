use core::{arch::naked_asm, intrinsics::black_box, marker::PhantomData, panic::PanicInfo, ptr::{DynMetadata, Pointee, metadata}};

use crate::oper_trait::{CustomResult, OperationArgs, OperationResult};

#[repr(C)]
pub struct TraitObjectWrapper<T: ?Sized> {
    data: *const (),
    vtable: *const (),
    _marker: PhantomData<T>,
}


impl<T: ?Sized> TraitObjectWrapper<T> {
    pub fn from_trait_ptr(ptr: *const T) -> Self
    where
        T: Pointee<Metadata = DynMetadata<T>>,
    {
        let (data_ptr, vtable_metadata) = ptr.to_raw_parts();
        let vtable_ptr = unsafe {
            core::mem::transmute::<DynMetadata<T>, *const ()>(vtable_metadata)
        };
        TraitObjectWrapper {
            data: data_ptr,
            vtable: vtable_ptr,
            _marker: PhantomData,
        }
    }

    pub fn data_ptr(&self) -> *const () {
        self.data
    }

    pub fn vtable_ptr(&self) -> *const () {
        self.vtable
    }

    pub fn execute<V, Z>(&self, args: *mut Z) -> V {
        let execute_fn_ptr = unsafe {
            let vtable = self.vtable as *const *const ();
            *vtable.add(3)
        };
        let execute_fn: unsafe fn(*const (), *mut Z) -> V = unsafe { core::mem::transmute(execute_fn_ptr) };
        unsafe { execute_fn(self.data, args) }
    }
}
