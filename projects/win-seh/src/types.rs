
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct UnwindCode {
    pub code_offset: u8,
    pub unwind_op: u8,
    pub op_info: u8,
    pub frame_offset: u16,
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy)]
pub struct VersionFlags(u8);

impl VersionFlags {
    pub const fn new(version: u8, flags: u8) -> Self {
        Self((version & 0x7) | ((flags & 0x1F) << 3))
    }

    pub const fn version(&self) -> u8 {
        self.0 & 0x7
    }

    pub const fn flags(&self) -> u8 {
        (self.0 >> 3) & 0x1F
    }

    pub const fn with_flags(mut self, flags: u8) -> Self {
        self.0 = (self.0 & 0x7) | ((flags & 0x1F) << 3);
        self
    }

    pub const fn with_version(mut self, version: u8) -> Self {
        self.0 = (self.0 & !0x7) | (version & 0x7);
        self
    }

    pub const EHANDLER: Self = Self::new(1, 0x01);
    pub const UHANDLER: Self = Self::new(1, 0x02);
    pub const CHAININFO: Self = Self::new(1, 0x04);
}

impl From<u8> for VersionFlags {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<VersionFlags> for u8 {
    fn from(value: VersionFlags) -> Self {
        value.0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union UnwindInfoFooter {
    pub exception_handler: u32,
    pub function_entry: u32,
}


#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct UnwindInfo {
    pub version_flags: VersionFlags,
    pub size_of_prolog: u8,
    pub count_of_codes: u8,
    pub frame_register_offset: u8,
    pub unwind_code: [UnwindCode; 1],
    pub footer: UnwindInfoFooter,
}

impl UnwindInfo {
    pub fn new() -> Self {
        Self {
            version_flags: VersionFlags::EHANDLER,
            size_of_prolog: 0,
            count_of_codes: 0,
            frame_register_offset: 0,
            unwind_code: [UnwindCode::default()],
            footer: UnwindInfoFooter { exception_handler: 0 },
        }
    }

    pub fn set_exception_handler(&mut self, rva: u32) {
        self.footer.exception_handler = rva;
    }

    pub fn build(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0] = self.version_flags.0;
        bytes[1] = self.size_of_prolog;
        bytes[2] = self.count_of_codes;
        bytes[3] = self.frame_register_offset;

        unsafe {
            let handler = self.footer.exception_handler.to_le_bytes();
            bytes[4..8].copy_from_slice(&handler);
        }

        bytes
    }
}