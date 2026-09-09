use core::mem::zeroed;
use alloc::vec;
use alloc::{format, string::String, vec::{Vec}};
use ntapi::{ntexapi::{KUSER_SHARED_DATA, NtQueryLicenseValue, NtQuerySystemInformationEx, SystemExtendedProcessInformation}, ntpebteb::PEB};
use toolkit::{KUserSharedData, ProcessEnvironmentBlock, println};
use widestring::{U16CStr, u16cstr};
use winapi::{shared::{ntdef::UNICODE_STRING, ntstatus::STATUS_UNSUCCESSFUL}, um::winnt::RTL_OSVERSIONINFOEXW};

const SUITE_NAMES: &[(u32, &str)] = &[
    (0x0001, "Small Business"),
    (0x0002, "Enterprise"),
    (0x0004, "BackOffice"),
    (0x0008, "Communications"),
    (0x0010, "Terminal Server"),
    (0x0020, "Small Business Restricted"),
    (0x0040, "Embedded NT"),
    (0x0080, "Datacenter"),
    (0x0100, "Single User TS"),
    (0x0200, "Personal"),
    (0x0400, "Blade"),
    (0x0800, "Embedded Restricted"),
    (0x1000, "Security Appliance"),
    (0x2000, "Storage Server"),
    (0x4000, "Compute Server"),
    (0x8000, "WH Server"),
];

const APPSERVER_MODE_LICENSE: &U16CStr = u16cstr!("TerminalServices-RemoteConnectionManager-AllowAppServerMode");

#[derive(Clone, Copy)]
pub struct VersionNumbers {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl VersionNumbers {
    pub fn from_peb(peb: &PEB) -> Self {
        Self {
            major: peb.OSMajorVersion,
            minor: peb.OSMinorVersion,
            build: peb.OSBuildNumber as u32,
        }
    }

    pub const fn new() -> Self {
        Self {
            major: KUserSharedData::major_version(),
            minor: 0,
            build: KUserSharedData::build_number(),
        }
    }

    pub fn to_windows_string(&self) -> String {
        match (self.major, self.minor, self.build) {
            (10, 0, build) if build >= 22000 => format!("Windows 11 (10.0.{})", build),
            (10, 0, build) if build >= 10240 => format!("Windows 10 (10.0.{})", build),
            (6, 3, build) => format!("Windows 8.1 (6.3.{})", build),
            (6, 2, build) => format!("Windows 8 (6.2.{})", build),
            (6, 1, build) => format!("Windows 7 (6.1.{})", build),
            (6, 0, build) => format!("Windows Vista (6.0.{})", build),
            (5, 2, build) if build >= 3790 => format!("Windows Server 2003 (5.2.{})", build),
            (5, 2, build) => format!("Windows XP x64 (5.2.{})", build),
            (5, 1, build) => format!("Windows XP (5.1.{})", build),
            (5, 0, build) => format!("Windows 2000 (5.0.{})", build),
            _ => format!("Windows {}.{}.{}", self.major, self.minor, self.build),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProductInfo {
    pub product_type: u32,
    pub suite_mask: u32,
    pub appserver_mode: bool,
}

impl ProductInfo {

    pub const fn new() -> Self {
        Self {
            product_type: KUserSharedData::product_type(),
            suite_mask: KUserSharedData::suite_mask(),
            appserver_mode: false,
        }
    }

    pub const fn product_name(&self) -> &'static str {
        match self.product_type {
            1 => "Workstation",
            2 => "Domain Controller",
            3 => "Server",
            _ => "Unknown",
        }
    }

    pub fn suite_names(&self) -> Vec<&'static str> {
        let mut suites: Vec<&'static str> = SUITE_NAMES
            .iter()
            .filter_map(|(bit, name)| {
                if self.suite_mask & bit != 0 {
                    Some(*name)
                } else {
                    None
                }
            })
            .collect();

        if self.appserver_mode {
            suites.push("AppServer Mode");
        }

        suites
    }
}

pub fn check_license_appserver_mode() -> Result<bool, &'static str> {
    unsafe {
        let mut unicode_name: UNICODE_STRING = zeroed();
        unicode_name.Buffer = APPSERVER_MODE_LICENSE.as_ptr() as *mut u16;
        unicode_name.Length = ((APPSERVER_MODE_LICENSE.len() - 1) * 2) as u16;
        unicode_name.MaximumLength = unicode_name.Length;

        let mut license_type: u32 = 0;
        let mut data: u32 = 0;
        let mut result_size: u32 = 0;

        let status = NtQueryLicenseValue(
            &mut unicode_name ,
            &mut license_type as *mut _,
            &mut data as *mut _ as *mut _,
            core::mem::size_of::<u32>() as u32,
            &mut result_size as *mut _,
        );

        if status == 0 && result_size >= 4 {
            Ok(data == 1)
        } else {
            Ok(false)
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SystemVersionInformation {
    pub revision: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub edition_string: [u8; 16],
    pub padding1: [u8; 112],
    pub branch: [u8; 11],
    pub padding2: [u8; 117],
    pub build_lab_short: [u8; 29],
    pub padding3: [u8; 99],
    pub build_lab: [u8; 40],
    pub padding4: [u8; 88],
    pub timestamp: [u8; 12],
    pub padding5: [u8; 14],
    pub architecture: [u8; 6],
    pub padding6: [u8; 12],
    pub extra_dword: u32,
}

impl SystemVersionInformation {
    pub fn new() -> Self {
        unsafe {
            let mut info: SystemVersionInformation = zeroed();
            let mut return_length = 0u32;
            let mut input_value: u32 = 0;

            NtQuerySystemInformationEx(
                0xDE,
                &mut input_value as *mut u32 as *mut winapi::ctypes::c_void,
                4,
                &mut info as *mut _ as *mut winapi::ctypes::c_void,
                0x244,
                &mut return_length as *mut u32,
            );

            info
        }
    }

    pub const fn branch(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.branch) }
    }

    pub const fn edition_string(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.edition_string) }
    }

    pub const fn build_lab(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.build_lab) }
    }

    pub const fn build_lab_short(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.build_lab_short) }
    }

    pub const fn architecture(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.architecture) }
    }

    pub const fn timestamp(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.timestamp) }
    }
}

impl core::fmt::Display for SystemVersionInformation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "SystemVersionInformation")?;
        writeln!(f, "Revision: {}", self.revision)?;
        writeln!(f, "Branch: {}", self.branch())?;
        writeln!(f, "Edition: {}", self.edition_string())?;
        writeln!(f, "Architecture: {}", self.architecture())?;
        writeln!(f, "Build Lab: {}", self.build_lab())?;
        writeln!(f, "Build Lab Short: {}", self.build_lab_short())?;
        writeln!(f, "Timestamp: {}", self.timestamp())?;
        write!(
            f,
            "Version: {}.{}.{}",
            self.major_version, self.minor_version, self.build_number
        )
    }
}

pub struct VersionInfo {
    pub numbers: VersionNumbers,
    pub product: ProductInfo,
    pub csd_version: String
}

impl VersionInfo {

    pub fn new() -> Self {
        let peb = ProcessEnvironmentBlock::current_process();
        let csd_version = {
            let slice = unsafe { core::slice::from_raw_parts(
                peb.CSDVersion.Buffer,
                peb.CSDVersion.Length as usize / 2) };

            String::from_utf16_lossy(slice)
        };
                
        Self {
            numbers: VersionNumbers::new(),
            product: ProductInfo::new(),
            csd_version
        }
    }

    pub fn to_display_string(&self) -> String {
        let version = self.numbers.to_windows_string();
        let product = self.product.product_name();
        let suites = self.product.suite_names();
        let csd = if self.csd_version.is_empty() {
            String::new()
        } else {
            format!(" {}", self.csd_version)
        };

        if suites.is_empty() {
            format!("{}{} [{}]", version, csd, product)
        } else {
            format!("{}{} [{} + {}]", version, csd, product, suites.join(", "))
        }
    }
}

impl core::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.numbers.major, self.numbers.minor, self.numbers.build
        )?;

        if !self.csd_version.is_empty() {
            write!(f, " {}", self.csd_version)?;
        }

        let suites = self.product.suite_names();
        if !suites.is_empty() {
            write!(
                f,
                " [{}: {}]",
                self.product.product_name(),
                suites.join(", ")
            )?;
        } else {
            write!(f, " [{}]", self.product.product_name())?;
        }

        Ok(())
    }
}