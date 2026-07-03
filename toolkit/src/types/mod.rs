mod byte_block; 
mod heap;

pub use byte_block::*;
pub use heap::*;
use winapi::shared::ntdef::UNICODE_STRING;

pub trait ToUnicode {
    fn as_unicode(&self) -> UNICODE_STRING;
}