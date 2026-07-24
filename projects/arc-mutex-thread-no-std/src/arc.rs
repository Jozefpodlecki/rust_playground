// use core::{ops::Deref, ptr::{self}, sync::atomic::{AtomicUsize, Ordering}};

// use alloc::boxed::Box;
// use toolkit::println;

// pub struct Arc<T: ?Sized>(*mut ArcInner<T>);

// struct ArcInner<T: ?Sized> {
//     count: AtomicUsize,
//     data: T,
// }

// unsafe impl<T: Send + Sync + ?Sized> Send for Arc<T> {}
// unsafe impl<T: Send + Sync + ?Sized> Sync for Arc<T> {}

// impl<T> Arc<T> {
//     pub fn new(data: T) -> Self {
//         let inner = Box::new(ArcInner {
//             count: AtomicUsize::new(1),
//             data,
//         });
        
//         Self(Box::into_raw(inner))
//     }
// }

// impl<T: ?Sized> Arc<T> {
//     pub fn clone(&self) -> Self {
//         unsafe {
//             (*self.0).count.fetch_add(1, Ordering::Relaxed);
//         }

//         Self(self.0)
//     }

//     pub fn leak(&self) -> *mut Self {
//         Box::leak(Box::new(self.clone()))
//     }
    
//     pub fn strong_count(&self) -> usize {
//         unsafe { (*self.0).count.load(Ordering::Relaxed) }
//     }

//     pub fn get_mut(&mut self) -> Option<&mut T> {
//         if self.strong_count() == 1 {
//             unsafe { Some(&mut (*self.0).data) }
//         } else {
//             None
//         }
//     }
// }


// impl<T: ?Sized> Deref for Arc<T> {
//     type Target = T;
    
//     fn deref(&self) -> &T {
//         unsafe { &(*self.0).data }
//     }
// }

// impl<T: ?Sized> Drop for Arc<T> {
//     fn drop(&mut self) {
//         unsafe {
//             if (*self.0).count.fetch_sub(1, Ordering::Release) != 1 {
//                 println!("arc dec ref count");
//                 return;
//             }
//             println!("arc drop");
//             core::sync::atomic::fence(Ordering::Acquire);
            
//             ptr::drop_in_place(&mut (*self.0).data);
//             drop(Box::from_raw(self.0));
//         }
//     }
// }