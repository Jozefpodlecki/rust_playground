
#[repr(C)]
pub struct BuddyBlock<const MIN_BLOCK: usize> {
    pub order: u8,
    pub free: bool,
    pub next: *mut BuddyBlock<MIN_BLOCK>,
}

impl<const MIN_BLOCK: usize> BuddyBlock<MIN_BLOCK> {
    pub const fn size_of() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn init(self: *mut Self, order: u8) {
        unsafe {
            (*self).order = order;
            (*self).free = true;
            (*self).next = core::ptr::null_mut();
        }
    }

    pub fn order(self: *mut Self) -> u8 {
        unsafe { (*self).order }
    }

    pub fn set_order(self: *mut Self, order: u8) {
        unsafe { (*self).order = order; }
    }

    pub fn free(self: *mut Self) -> bool {
        unsafe { (*self).free }
    }

    pub fn set_free(self: *mut Self, free: bool) {
        unsafe { (*self).free = free; }
    }

    pub fn next(self: *mut Self) -> *mut Self {
        unsafe { (*self).next }
    }

    pub fn set_next(self: *mut Self, next: *mut Self) {
        unsafe { (*self).next = next; }
    }

    pub fn size(self: *mut Self) -> usize {
        MIN_BLOCK << unsafe { (*self).order as usize }
    }

    pub fn data(self: *mut Self) -> *mut u8 {
        unsafe { self.cast::<u8>().add(Self::size_of()) }
    }

    pub fn from_ptr(ptr: *mut u8) -> *mut Self {
        unsafe { ptr.sub(Self::size_of()).cast() }
    }

    pub fn add_offset(self: *mut Self, offset: usize) -> *mut Self {
        unsafe { self.cast::<u8>().add(offset).cast() }
    }

    pub fn buddy(self: *mut Self, order: u8) -> *mut Self {
        let block_size = MIN_BLOCK << order as usize;
        let block_addr = self as usize;
        
        if (block_addr / block_size) % 2 == 0 {
            self.add_offset(block_size).cast()
        } else {
            self.add_offset(block_size.wrapping_neg()).cast()
        }
    }
}

#[repr(align(8))]
pub struct Header(u8);

impl Header {
    pub const fn size_of() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn init(self: *mut Self) {
        unsafe { (*self).0 = 1; }
    }

    pub fn is_initialized(self: *mut Self) -> bool {
        unsafe { (*self).0 == 1 }
    }
}

#[repr(C)]
pub struct FreeListHeads<const MAX_ORDER: usize, const MIN_BLOCK: usize> {
    pub heads: [*mut BuddyBlock<MIN_BLOCK>; MAX_ORDER],
    pub extra: *mut BuddyBlock<MIN_BLOCK>,
}

impl<const MAX_ORDER: usize, const MIN_BLOCK: usize> FreeListHeads<MAX_ORDER, MIN_BLOCK> {
    pub const fn size_of() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn init(self: *mut Self) {
        unsafe {
            for i in 0..MAX_ORDER {
                (*self).heads[i] = core::ptr::null_mut();
            }
            (*self).extra = core::ptr::null_mut();
        }
    }

    pub fn head(self: *mut Self, order: usize) -> *mut *mut BuddyBlock<MIN_BLOCK> {
        unsafe {
            if order < MAX_ORDER {
                &mut (*self).heads[order]
            } else {
                &mut (*self).extra
            }
        }
    }
}


pub struct BuddyBlockIter<const MIN_BLOCK: usize>(pub *mut BuddyBlock<MIN_BLOCK>);

#[derive(Debug, Clone, Copy)]
pub struct BuddyBlockInfo<const MIN_BLOCK: usize> {
    pub ptr: *mut BuddyBlock<MIN_BLOCK>,
    pub order: u8,
    pub size: usize,
    pub free: bool,
}

impl<const MIN_BLOCK: usize> BuddyBlockInfo<MIN_BLOCK> {
    pub fn ptr(&self) -> *mut BuddyBlock<MIN_BLOCK> { self.ptr }
    pub fn order(&self) -> u8 { self.order }
    pub fn size(&self) -> usize { self.size }
    pub fn free(&self) -> bool { self.free }
}

impl<const MIN_BLOCK: usize> Iterator for BuddyBlockIter<MIN_BLOCK> {
    type Item = BuddyBlockInfo<MIN_BLOCK>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_null() {
            return None;
        }
        
        let block = self.0;
        let info = BuddyBlockInfo {
            ptr: block,
            order: block.order(),
            size: block.size(),
            free: block.free(),
        };
        
        self.0 = self.0.add_offset(block.size());
        Some(info)
    }
}

impl<const MIN_BLOCK: usize> core::fmt::Display for BuddyBlockInfo<MIN_BLOCK> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:p} order={} size={} {}",
            self.ptr,
            self.order,
            self.size,
            if self.free { "free" } else { "used" }
        )
    }
}