use core::{
    alloc::Layout, borrow::{Borrow, BorrowMut}, cmp::Ordering, fmt, hash::{Hash, Hasher}, marker::Unsize, mem::{self, ManuallyDrop, MaybeUninit}, ops::{CoerceUnsized, Deref, DerefMut}, ptr::{self, NonNull, Unique}, slice,
};
use crate::alloc::{allocate, deallocate};
use core::mem::SizedTypeProperties;

pub struct Box<T: ?Sized>(pub(crate) Unique<T>);

impl<T> Box<T> {
    pub fn new(value: T) -> Self {
        unsafe {
            // let layout = Layout::new::<T>();
            let layout = <T as SizedTypeProperties>::LAYOUT;
            let ptr = allocate(layout);
            if ptr.is_null() {
                panic!("allocation failed");
            }
            (ptr as *mut T).write(value);
            Box(Unique::new_unchecked(ptr as *mut T))
        }
    }

    pub fn new_uninit() -> Box<MaybeUninit<T>> {
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = allocate(layout);
            if ptr.is_null() {
                panic!("allocation failed");
            }
            Box(Unique::new_unchecked(ptr as *mut MaybeUninit<T>))
        }
    }

    pub unsafe fn assume_init(self) -> Box<T> {
        unsafe {
            Box(Unique::new_unchecked(self.0.as_ptr() as *mut T))
        }
    }

    pub fn into_raw(b: Self) -> *mut T {
        let boxed = ManuallyDrop::new(b);
        boxed.0.as_ptr()
    }

    pub unsafe fn from_raw(raw: *mut T) -> Self {
        unsafe {
            Box(Unique::new_unchecked(raw))
        }
    }

    pub fn leak<'a>(b: Self) -> &'a mut T {
        let ptr = Self::into_raw(b);
        unsafe { &mut *ptr }
    }
}

impl<T: Clone> Box<T> {
    pub fn clone(&self) -> Self {
        Box::new((**self).clone())
    }
}

impl<T: Default> Box<T> {
    pub fn default() -> Self {
        Box::new(T::default())
    }
}

impl<T> Box<[T]> {
    pub fn new_uninit_slice(len: usize) -> Box<[MaybeUninit<T>]> {
        unsafe {
            let layout = Layout::array::<MaybeUninit<T>>(len).unwrap();
            let ptr = allocate(layout);
            if ptr.is_null() {
                panic!("allocation failed");
            }
            
            // Create a fat pointer from thin pointer + length
            let slice_ptr = ptr as *mut MaybeUninit<T>;
            let fat_ptr = core::ptr::slice_from_raw_parts_mut(slice_ptr, len);
            
            Box(Unique::new_unchecked(fat_ptr))
        }
    }
}

impl<T> Box<[MaybeUninit<T>]> {
    pub unsafe fn assume_init(self) -> Box<[T]> {
        unsafe {
            let ptr = self.0.as_ptr() as *mut [T];
            Box(Unique::new_unchecked(ptr))
        }
    }
}

impl<T: Clone> Box<[T]> {
    pub fn clone(&self) -> Self {
        let len = (**self).len();
        let new_box = Box::<[T]>::new_uninit_slice(len);
        unsafe {
            let slice_ptr = new_box.0.as_ptr() as *mut [MaybeUninit<T>];
            let data_ptr = (*slice_ptr).as_mut_ptr();
            
            for i in 0..len {
                data_ptr.add(i).write(MaybeUninit::new(self[i].clone()));
            }

            new_box.assume_init()
        }
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            if mem::size_of_val(&**self) > 0 {
                self.0.as_ptr().drop_in_place();
                let layout = Layout::for_value_raw(self.0.as_ptr());
                deallocate(self.0.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Box<T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<T: ?Sized + Eq> Eq for Box<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for Box<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Ord> Ord for Box<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: ?Sized + Hash> Hash for Box<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Pointer> fmt::Pointer for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}

impl<T> From<T> for Box<T> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl<T: Clone> From<&[T]> for Box<[T]> {
    fn from(slice: &[T]) -> Self {
        let mut boxed = Box::<[T]>::new_uninit_slice(slice.len());
        unsafe {
            let slice_ptr = boxed.0.as_ptr() as *mut [MaybeUninit<T>];
            let data_ptr = (*slice_ptr).as_mut_ptr();
            
            for (i, value) in slice.iter().enumerate() {
                data_ptr.add(i).write(MaybeUninit::new(value.clone()));
            }
            
            boxed.assume_init()
        }
    }
}

impl<T: Clone> From<&mut [T]> for Box<[T]> {
    fn from(slice: &mut [T]) -> Self {
        Box::from(&*slice)
    }
}

impl<T, const N: usize> From<[T; N]> for Box<[T]> {
    fn from(array: [T; N]) -> Self {
        let mut boxed = Box::<[T]>::new_uninit_slice(N);
        unsafe {
            let slice_ptr = boxed.0.as_ptr() as *mut [MaybeUninit<T>];
            let data_ptr = (*slice_ptr).as_mut_ptr();
            
            for (i, value) in array.into_iter().enumerate() {
                data_ptr.add(i).write(MaybeUninit::new(value));
            }
            
            boxed.assume_init()
        }
    }
}

impl<T: ?Sized> Borrow<T> for Box<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T: ?Sized> BorrowMut<T> for Box<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: ?Sized + AsRef<U>, U: ?Sized> AsRef<U> for Box<T> {
    fn as_ref(&self) -> &U {
        (**self).as_ref()
    }
}

impl<T: ?Sized + AsMut<U>, U: ?Sized> AsMut<U> for Box<T> {
    fn as_mut(&mut self) -> &mut U {
        (**self).as_mut()
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Box<U>> for Box<T> {}