use core::{alloc::Layout, mem::{self, offset_of}, ops::Deref, ptr::{self, NonNull}, sync::atomic::{AtomicUsize, Ordering}};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::alloc::AllocError;

#[cfg(feature = "alloc")]
pub use alloc::boxed::Box;

pub struct Arc<T: ?Sized>(*mut ArcInner<T>);

pub struct ArcInner<T: ?Sized> {
    count: AtomicUsize,
    data: T,
}

unsafe impl<T: Send + Sync + ?Sized> Send for Arc<T> {}
unsafe impl<T: Send + Sync + ?Sized> Sync for Arc<T> {}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        let inner = Box::new(ArcInner {
            count: AtomicUsize::new(1),
            data,
        });
        
        Self(Box::into_raw(inner))
    }

    pub fn new_uninit() -> Arc<mem::MaybeUninit<T>> {
        let layout = Layout::new::<T>();
        let ptr = unsafe {
            let raw = alloc::alloc::alloc(layout);
            if raw.is_null() {
                alloc::alloc::handle_alloc_error(layout);
            }
            raw as *mut ArcInner<mem::MaybeUninit<T>>
        };
        unsafe {
            ptr.write(ArcInner {
                count: AtomicUsize::new(1),
                data: mem::MaybeUninit::uninit(),
            });
        }
        Arc(ptr)
    }

    pub fn new_uninit_in() -> Arc<mem::MaybeUninit<T>> {
        unsafe {
            Arc::from_ptr_in(
                Arc::allocate_for_layout(
                    Layout::new::<T>(),
                    |layout| {
                        let ptr: *mut u8 = alloc::alloc::alloc(layout);
                        let slice_ptr = core::ptr::slice_from_raw_parts_mut(ptr, layout.size());
                        let non_null = NonNull::new(slice_ptr).unwrap();
                        Ok(non_null)
                    },
                    <*mut u8>::cast,
                )
            )
        }
    }

    unsafe fn from_ptr_in(ptr: *mut ArcInner<T>) -> Self {
        unsafe { Self::from_inner_in(NonNull::new_unchecked(ptr)) }
    }

    unsafe fn from_inner_in(ptr: NonNull<ArcInner<T>>) -> Self {
        Self(ptr.as_ptr())
    }

    pub unsafe fn from_raw_arcinner(ptr: *const ArcInner<T>) -> Self {
        Self(ptr as *mut ArcInner<T>)
    }

    pub unsafe fn from_raw(ptr: *const T) -> Self {
        let data_ptr = ptr as *const u8;
        let inner_ptr = data_ptr.sub(offset_of!(ArcInner<T>, data)) as *mut ArcInner<T>;
        Self(inner_ptr)
    }

    pub fn as_ptr(this: &Self) -> *const T {
        this.deref() as *const T
    }
}

impl<T> Arc<mem::MaybeUninit<T>> {
    pub unsafe fn assume_init(self) -> Arc<T> {
        let inner = self.0;
        core::mem::forget(self);
        Arc(inner as *mut ArcInner<T>)
    }
}


impl<T: ?Sized> Arc<T> {
    pub fn clone(&self) -> Self {
        unsafe {
            (*self.0).count.fetch_add(1, Ordering::Relaxed);
        }

        Self(self.0)
    }

    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        unsafe { &mut (*this.0).data }
    }

    unsafe fn initialize_arcinner(
        ptr: NonNull<[u8]>,
        layout: Layout,
        mem_to_arcinner: impl FnOnce(*mut u8) -> *mut ArcInner<T>,
    ) -> *mut ArcInner<T> {
        let inner = mem_to_arcinner(ptr.as_non_null_ptr().as_ptr());

        unsafe {
            (&raw mut (*inner).count).write(AtomicUsize::new(1));
        }

        inner
    }

    unsafe fn allocate_for_layout(
        value_layout: Layout,
        allocate: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocError>,
        mem_to_arcinner: impl FnOnce(*mut u8) -> *mut ArcInner<T>,
    ) -> *mut ArcInner<T> {
        let layout = Layout::new::<ArcInner<()>>().extend(value_layout).unwrap().0.pad_to_align();

        let ptr = allocate(layout).unwrap_or_else(|_| panic!("allocation failed"));

        unsafe { Self::initialize_arcinner(ptr, layout, mem_to_arcinner) }
    }

    // pub fn leak(&self) -> *mut T {
    //     // Box::leak(Box::new(self.clone()))
    //     let cloned = self.clone();
    //     let ptr = cloned.deref() as *const T as *mut T;
    //     core::mem::forget(cloned);
    //     ptr
    // }

    pub fn leak(this: Self) -> *mut ArcInner<T> {
        let ptr = this.0;
        core::mem::forget(this);
        ptr
    }
    
    pub fn strong_count(&self) -> usize {
        unsafe { (*self.0).count.load(Ordering::Relaxed) }
    }

    pub fn into_raw_arcinner(this: Self) -> *mut ArcInner<T> {
        let ptr = this.0;
        core::mem::forget(this);
        ptr
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.strong_count() == 1 {
            unsafe { Some(&mut (*self.0).data) }
        } else {
            None
        }
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.0).count.fetch_add(1, Ordering::Relaxed);
        }
        Self(self.0)
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &(*self.0).data }
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.0).count.fetch_sub(1, Ordering::Release) != 1 {
                return;
            }

            core::sync::atomic::fence(Ordering::Acquire);
            
            ptr::drop_in_place(&mut (*self.0).data);
            drop(Box::from_raw(self.0));
        }
    }
}