use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use core::mem;

#[repr(C)]
struct Block {
    size: usize,
    next: Option<&'static mut Block>,
}

struct BlockHeader;
impl BlockHeader {
    const fn size() -> usize {
        mem::size_of::<Block>()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Address(usize);

impl Address {
    fn new(ptr: *mut u8) -> Self {
        Address(ptr as usize)
    }
    
    fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
    
    fn add(&self, offset: usize) -> Self {
        Address(self.0 + offset)
    }
    
    fn sub(&self, offset: usize) -> Self {
        Address(self.0 - offset)
    }
    
    fn align_to(&self, align: usize) -> Self {
        Address((self.0 + align - 1) & !(align - 1))
    }
    
    fn xor(&self, other: usize) -> Self {
        Address(self.0 ^ other)
    }
    
    fn is_valid(&self, heap_start: Address, heap_end: Address) -> bool {
        self.0 >= heap_start.0 && self.0 < heap_end.0
    }
}

#[derive(Clone, Copy)]
struct Size(usize);

impl Size {
    fn to_order(&self) -> usize {
        let mut order = 0;
        let mut block_size = MIN_BLOCK_SIZE;
        while block_size < self.0 && order < MAX_ORDER - 1 {
            block_size <<= 1;
            order += 1;
        }
        order
    }
    
    fn from_order(order: usize) -> Self {
        Size(MIN_BLOCK_SIZE << order)
    }
    
    fn with_padding(&self, padding: usize) -> Self {
        Size(self.0 - padding - BlockHeader::size())
    }
}

pub struct SegregatedAllocator {
    free_lists: [Option<&'static mut Block>; MAX_ORDER],
    heap_start: Address,
    heap_end: Address,
    used: usize,
    total: usize,
}

impl SegregatedAllocator {
    const fn new() -> Self {
        SegregatedAllocator {
            free_lists: [const { None }; MAX_ORDER],
            heap_start: Address(0),
            heap_end: Address(0),
            used: 0,
            total: 0,
        }
    }

    fn init(&mut self, heap: &'static mut [u8]) {
        unsafe {
            self.heap_start = Address::new(heap.as_mut_ptr());
            self.heap_end = Address::new(heap.as_mut_ptr().add(heap.len()));
            self.total = heap.len();
            
            let block_ptr = self.heap_start.as_mut_ptr::<Block>();
            let block = &mut *block_ptr;
            block.size = heap.len() - BlockHeader::size();
            block.next = None;
            
            let order = Size(block.size).to_order();
            self.free_lists[order] = Some(block);
        }
    }

    fn allocate(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
            let size = layout.size().max(MIN_BLOCK_SIZE);
            let align = layout.align();
            let order = Size(size).to_order();
            
            let (block_ptr, current_order) = match self.find_free_block(order) {
                Some(result) => result,
                None => return core::ptr::null_mut(),
            };

            let block_size = Size::from_order(current_order).0;
            
            let aligned_user_ptr = self.align_user_ptr(
                block_ptr.add(BlockHeader::size()),
                align
            );
            
            let padding = aligned_user_ptr.0 - block_ptr.0 - BlockHeader::size();
            
            if padding > 0 && padding >= MIN_BLOCK_SIZE + BlockHeader::size() {
                self.split_padding(block_ptr, padding);
            }
            
            self.split_remaining_blocks(
                block_ptr.add(BlockHeader::size() + padding),
                order,
                current_order
            );
            
            let result_ptr = block_ptr.add(BlockHeader::size() + padding);
            let result_block = &mut *result_ptr.as_mut_ptr::<Block>();
            result_block.size = block_size - padding - BlockHeader::size();
            result_block.next = None;
            
            self.used += block_size;
            result_ptr.add(BlockHeader::size()).as_mut_ptr()
        }
    }

    fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        unsafe {
            let user_ptr = Address::new(ptr);
            let block_ptr = user_ptr.sub(BlockHeader::size());
            let block = &mut *block_ptr.as_mut_ptr::<Block>();
            let block_size = block.size + BlockHeader::size();
            let order = Size(block.size).to_order();
            
            self.used -= block_size;
            self.insert_into_free_list(block_ptr, order);
            self.coalesce(order, block_ptr);
        }
    }

    unsafe fn find_free_block(&mut self, order: usize) -> Option<(Address, usize)> {
        let mut current_order = order;
        
        while current_order < MAX_ORDER {
            if let Some(block) = self.free_lists[current_order].take() {
                let block_ptr = Address::new(block as *mut Block as *mut u8);
                return Some((block_ptr, current_order));
            }
            current_order += 1;
        }
        
        None
    }

    fn align_user_ptr(&self, user_ptr: Address, align: usize) -> Address {
        user_ptr.align_to(align)
    }

    fn split_padding(&mut self, block_ptr: Address, padding: usize) {
        unsafe {
            let left_block_ptr = block_ptr;
            let left_block = &mut *left_block_ptr.as_mut_ptr::<Block>();
            left_block.size = padding - BlockHeader::size();
            left_block.next = None;
            
            let left_order = Size(left_block.size).to_order();
            self.insert_into_free_list(left_block_ptr, left_order);
        }
    }

    unsafe fn split_remaining_blocks(&mut self, start_ptr: Address, start_order: usize, current_order: usize) {
        let mut remaining_ptr = start_ptr;
        
        for order in (start_order..current_order).rev() {
            let half_size = Size::from_order(order).0;
            let buddy_ptr = remaining_ptr.add(half_size);
            let buddy_block = &mut *buddy_ptr.as_mut_ptr::<Block>();
            buddy_block.size = half_size - BlockHeader::size();
            buddy_block.next = None;
            self.insert_into_free_list(buddy_ptr, order);
        }
    }

    fn insert_into_free_list(&mut self, block_ptr: Address, order: usize) {
        unsafe {
            let block = &mut *block_ptr.as_mut_ptr::<Block>();
            
            if let Some(head) = self.free_lists[order].take() {
                let mut prev_ptr: *mut Block = core::ptr::null_mut();
                let mut current: *mut Block = head;
                
                while !current.is_null() {
                    let current_block = &mut *current;
                    
                    if block_ptr.0 < (current_block as *mut Block as usize) {
                        block.next = Some(&mut *current);
                        if prev_ptr.is_null() {
                            self.free_lists[order] = Some(block);
                        } else {
                            let prev_block = &mut *prev_ptr;
                            prev_block.next = Some(block);
                        }
                        return;
                    }
                    
                    prev_ptr = current;
                    current = match &mut current_block.next {
                        Some(next) => *next as *mut Block,
                        None => core::ptr::null_mut(),
                    };
                }
                
                // Insert at the end
                block.next = None;
                if prev_ptr.is_null() {
                    self.free_lists[order] = Some(block);
                } else {
                    let prev_block = &mut *prev_ptr;
                    prev_block.next = Some(block);
                }
            } else {
                // List is empty
                block.next = None;
                self.free_lists[order] = Some(block);
            }
        }
    }

    fn coalesce(&mut self, order: usize, block_ptr: Address) {
        unsafe {
            let block_size = Size::from_order(order).0;
            let buddy_addr = block_ptr.xor(block_size);
            
            if buddy_addr.is_valid(self.heap_start, self.heap_end) {
                let mut found = false;
                let mut prev_ptr: *mut Block = core::ptr::null_mut();
                let mut current = self.free_lists[order].take();
                
                while let Some(free_block) = current {
                    if (free_block as *mut Block as usize) == buddy_addr.0 {
                        found = true;
                        if !prev_ptr.is_null() {
                            let prev_block = &mut *prev_ptr;
                            prev_block.next = free_block.next.take();
                        } else {
                            self.free_lists[order] = free_block.next.take();
                        }
                        break;
                    }
                    prev_ptr = free_block;
                    current = free_block.next.take();
                }
                
                if found {
                    let merged_ptr = if block_ptr.0 < buddy_addr.0 {
                        block_ptr
                    } else {
                        buddy_addr
                    };
                    
                    if merged_ptr.0 == self.heap_start.0 && order + 1 < MAX_ORDER {
                        let merged_block = &mut *merged_ptr.as_mut_ptr::<Block>();
                        merged_block.size = Size::from_order(order + 1).0 - BlockHeader::size();
                        merged_block.next = None;
                        self.insert_into_free_list(merged_ptr, order + 1);
                    }
                }
            }
        }
    }
    }

static mut ALLOCATOR: UnsafeCell<SegregatedAllocator> = 
    UnsafeCell::new(SegregatedAllocator::new());

const HEAP_SIZE: usize = 65536;
const MIN_BLOCK_SIZE: usize = 16;
const MAX_ORDER: usize = 12;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init_allocator() {
    unsafe {
        (*ALLOCATOR.get()).init(&mut HEAP);
    }
}

pub fn allocate(layout: Layout) -> *mut u8 {
    unsafe {
        (*ALLOCATOR.get()).allocate(layout)
    }
}

pub fn deallocate(ptr: *mut u8, layout: Layout) {
    unsafe {
        (*ALLOCATOR.get()).deallocate(ptr, layout)
    }
}