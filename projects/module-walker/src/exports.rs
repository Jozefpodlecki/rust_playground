use core::{fmt, ops::Deref, slice};

use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_EXPORT_DIRECTORY, IMAGE_NT_HEADERS64, IMAGE_NT_SIGNATURE};

pub struct ModuleExportsIter {
    base: usize,
    export_dir: *const IMAGE_EXPORT_DIRECTORY,
    current_index: u32,
    count: u32,
    names: *const u32,
    functions: *const u32,
    ordinals: *const u16,
}

impl ModuleExportsIter {
    pub fn new(base_ptr: *mut winapi::ctypes::c_void) -> Option<Self> {
        unsafe {
            let base = base_ptr as usize;
            let dos_header = base_ptr as *const IMAGE_DOS_HEADER;
            if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
                return None;
            }

            let nt_headers = (base + (*dos_header).e_lfanew as usize) as *const IMAGE_NT_HEADERS64;
            if (*nt_headers).Signature != IMAGE_NT_SIGNATURE {
                return None;
            }

            let export_rva = (*nt_headers).OptionalHeader.DataDirectory[0].VirtualAddress;
            if export_rva == 0 {
                return None;
            }

            let export_dir = (base + export_rva as usize) as *const IMAGE_EXPORT_DIRECTORY;
            let count = (*export_dir).NumberOfNames;

            let names = (base + (*export_dir).AddressOfNames as usize) as *const u32;
            let functions = (base + (*export_dir).AddressOfFunctions as usize) as *const u32;
            let ordinals = (base + (*export_dir).AddressOfNameOrdinals as usize) as *const u16;

            Some(Self {
                base,
                export_dir,
                current_index: 0,
                count,
                names,
                functions,
                ordinals,
            })
        }
    }
}

impl Iterator for ModuleExportsIter {
    type Item = ExportEntry;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current_index >= self.count {
                return None;
            }

            let name_rva = *self.names.add(self.current_index as usize);
            let name_ptr = (self.base + name_rva as usize) as *const u8;
            let mut name_len = 0;
            while *name_ptr.add(name_len) != 0 {
                name_len += 1;
            }
            let name_bytes = slice::from_raw_parts(name_ptr, name_len);
            let name = str::from_utf8(name_bytes).ok()?;

            let ordinal_index = *self.ordinals.add(self.current_index as usize) as u32;
            let function_rva = *self.functions.add(ordinal_index as usize);
            let address = self.base + function_rva as usize;
            let ordinal = (*self.export_dir).Base + ordinal_index;

            self.current_index += 1;

            Some(ExportEntry {
                name: ExportName::new(name),
                address: ExportAddress(address),
                ordinal: ExportOrdinal(ordinal as u16),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExportAddress(pub usize);

impl ExportAddress {
    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }
}

impl Deref for ExportAddress {
    type Target = usize;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ExportAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExportOrdinal(pub u16);

#[derive(Debug)]
pub struct ExportName {
    bytes: [u8; 256],
    len: usize,
}

impl ExportName {
    pub fn new(s: &str) -> Self {
        let mut bytes = [0u8; 256];
        let bytes_slice = s.as_bytes();
        let len = bytes_slice.len().min(255);
        bytes[..len].copy_from_slice(&bytes_slice[..len]);
        Self { bytes, len }
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.len == 0 {
            return None;
        }
        str::from_utf8(&self.bytes[..self.len]).ok()
    }
}

impl fmt::Display for ExportName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.as_str() {
            write!(f, "{}", s)
        } else {
            write!(f, "")
        }
    }
}

impl PartialEq<str> for ExportName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}

impl PartialEq<&str> for ExportName {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == Some(*other)
    }
}

impl PartialEq for ExportName {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

#[derive(Debug)]
pub struct ExportEntry {
    pub name: ExportName,
    pub address: ExportAddress,
    pub ordinal: ExportOrdinal,
}

impl ExportEntry {
    pub fn name_str(&self) -> Option<&str> {
        self.name.as_str()
    }

    pub fn address_usize(&self) -> usize {
        self.address.0
    }

    pub fn ordinal_u16(&self) -> u16 {
        self.ordinal.0
    }
}