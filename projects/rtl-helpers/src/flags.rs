use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessParametersFlags(u32);

impl From<u32> for ProcessParametersFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl ProcessParametersFlags {
    pub const NORMALIZED: Self = Self(0x00000001);
    pub const PROFILE_USER: Self = Self(0x00000002);
    pub const PROFILE_KERNEL: Self = Self(0x00000004);
    pub const PROFILE_SERVER: Self = Self(0x00000008);
    pub const RESERVE_1MB: Self = Self(0x00000020);
    pub const RESERVE_16MB: Self = Self(0x00000040);
    pub const CASE_SENSITIVE: Self = Self(0x00000080);
    pub const DISABLE_HEAP_DECOMMIT: Self = Self(0x00000100);
    pub const DLL_REDIRECTION_LOCAL: Self = Self(0x00001000);
    pub const APP_MANIFEST_PRESENT: Self = Self(0x00002000);
    pub const IMAGE_KEY_MISSING: Self = Self(0x00004000);
    pub const DEV_OVERRIDE_ENABLED: Self = Self(0x00008000);
    pub const OPTIN_PROCESS: Self = Self(0x00020000);
    pub const SESSION_OWNER: Self = Self(0x00040000);
    pub const HANDLE_USER_CALLBACK_EXCEPTIONS: Self = Self(0x00080000);
    pub const PROTECTED_PROCESS: Self = Self(0x00400000);
    pub const RESERVE_PLACEHOLDER: Self = Self(0x01000000);
    pub const SECURE_PROCESS: Self = Self(0x80000000);

    pub fn contains(&self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (Self, &'static str)> {
        [
            (Self::NORMALIZED, "RTL_USER_PROC_PARAMS_NORMALIZED"),
            (Self::PROFILE_USER, "RTL_USER_PROC_PROFILE_USER"),
            (Self::PROFILE_KERNEL, "RTL_USER_PROC_PROFILE_KERNEL"),
            (Self::PROFILE_SERVER, "RTL_USER_PROC_PROFILE_SERVER"),
            (Self::RESERVE_1MB, "RTL_USER_PROC_RESERVE_1MB"),
            (Self::RESERVE_16MB, "RTL_USER_PROC_RESERVE_16MB"),
            (Self::CASE_SENSITIVE, "RTL_USER_PROC_CASE_SENSITIVE"),
            (Self::DISABLE_HEAP_DECOMMIT, "RTL_USER_PROC_DISABLE_HEAP_DECOMMIT"),
            (Self::DLL_REDIRECTION_LOCAL, "RTL_USER_PROC_DLL_REDIRECTION_LOCAL"),
            (Self::APP_MANIFEST_PRESENT, "RTL_USER_PROC_APP_MANIFEST_PRESENT"),
            (Self::IMAGE_KEY_MISSING, "RTL_USER_PROC_IMAGE_KEY_MISSING"),
            (Self::DEV_OVERRIDE_ENABLED, "RTL_USER_PROC_DEV_OVERRIDE_ENABLED"),
            (Self::OPTIN_PROCESS, "RTL_USER_PROC_OPTIN_PROCESS"),
            (Self::SESSION_OWNER, "RTL_USER_PROC_SESSION_OWNER"),
            (Self::HANDLE_USER_CALLBACK_EXCEPTIONS, "RTL_USER_PROC_HANDLE_USER_CALLBACK_EXCEPTIONS"),
            (Self::PROTECTED_PROCESS, "RTL_USER_PROC_PROTECTED_PROCESS"),
            (Self::RESERVE_PLACEHOLDER, "RTL_USER_PROC_RESERVE_PLACEHOLDER"),
            (Self::SECURE_PROCESS, "RTL_USER_PROC_SECURE_PROCESS"),
        ]
        .into_iter()
        .filter(move |(flag, _)| self.contains(*flag))
    }
}

impl fmt::Display for ProcessParametersFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "0x00000000 (no flags)");
        }

        let mut first = true;
        for (_, name) in self.iter() {
            if !first {
                write!(f, " | ")?;
            }
            write!(f, "{}", name)?;
            first = false;
        }

        // Also show hex value
        write!(f, " (0x{:08X})", self.0)?;

        Ok(())
    }
}

impl From<ProcessParametersFlags> for u32 {
    fn from(flags: ProcessParametersFlags) -> Self {
        flags.0
    }
}