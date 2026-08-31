use ntapi::{ntldr::LDR_DATA_TABLE_ENTRY, ntpsapi::PEB_LDR_DATA};
use winapi::shared::ntdef::LIST_ENTRY;

use crate::{ProcessEnvironmentBlock, utils::display::Utf16String};

pub enum ListOrder {
    LoadOrder,
    MemoryOrder,
    InitializationOrder,
}

pub struct ModulesIterator {
    current: *mut LIST_ENTRY,
    head: *mut LIST_ENTRY,
    kind: ListOrder,
    order: usize,
}

impl ModulesIterator {
    pub fn new(list_head: *mut LIST_ENTRY, kind: ListOrder) -> Self {
        
        unsafe {
            let current = (*list_head).Flink;
            
            ModulesIterator {
                current,
                head: list_head,
                kind,
                order: 0
            }
        }
    }
}

pub struct LdrDataTableEntry {
    pub order: usize,
    inner: LDR_DATA_TABLE_ENTRY
}

impl LdrDataTableEntry {
    pub fn base_dll_name(&self) -> Utf16String<'_> {
        Utf16String::new(self.inner.BaseDllName.Buffer, self.inner.BaseDllName.Length)
    }
}

impl Iterator for ModulesIterator {
    type Item = LdrDataTableEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }
        
        unsafe {
            if self.current == self.head {
                return None;
            }
            
            let entry = self.current as *mut LDR_DATA_TABLE_ENTRY;
            let result = *entry;
            self.current = match self.kind {
                ListOrder::LoadOrder => (*entry).InLoadOrderLinks.Flink,
                ListOrder::MemoryOrder => (*entry).InMemoryOrderLinks.Flink,
                ListOrder::InitializationOrder => (*entry).u1.InInitializationOrderLinks.Flink,
            };
            self.order += 1;
            
            Some(LdrDataTableEntry {
                order: self.order,
                inner: result
            })
        }
    }
}