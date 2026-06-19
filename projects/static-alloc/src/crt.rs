
#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn __CxxFrameHandler3() {
    
}

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        let mut dest_ptr = dest as *mut usize;
        let mut src_ptr = src as *const usize;
        
        while i + 8 <= n {
            *dest_ptr = *src_ptr;
            dest_ptr = dest_ptr.add(1);
            src_ptr = src_ptr.add(1);
            i += 8;
        }
        
        let mut dest_ptr = dest_ptr as *mut u8;
        let mut src_ptr = src_ptr as *const u8;
        while i < n {
            *dest_ptr = *src_ptr;
            dest_ptr = dest_ptr.add(1);
            src_ptr = src_ptr.add(1);
            i += 1;
        }
        
        dest
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn memset(mut dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        let original_dest = dest;
        let mut i = 0;
        while i < n {
            unsafe {
                *dest.add(i) = c as u8;
            }
            i += 1;
        }
        original_dest
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest < src as *mut u8 {
        let mut i = 0;
        while i < n {
            unsafe {
                *dest.add(i) = *src.add(i);
            }
            i += 1;
        }
    } else {
        let mut i = n;
        while i > 0 {
            i -= 1;
            unsafe {
                *dest.add(i) = *src.add(i);
            }
        }
    }
    dest
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        for i in 0..n {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b {
                return a as i32 - b as i32;
            }
        }
        0
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn strlen(s: *const u8) -> usize {
    unsafe {
        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}