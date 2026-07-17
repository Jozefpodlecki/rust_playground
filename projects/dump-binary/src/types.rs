use core::fmt::{Debug, Display};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use pelite::image::*;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy)]
pub struct Address(pub u64);

impl core::fmt::Display for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{:X}", self.0))
    }
}

impl From<u64> for Address {
    fn from(value: u64) -> Self {
        Address(value)
    }
}

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Address(value as u64)
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Address(value as u64)
    }
}

impl From<Address> for u64 {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl From<Address> for u32 {
    fn from(value: Address) -> Self {
        value.0 as u32
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Binary {
    pub file_header: FileHeader,
    pub optional_header: OptionalHeader,
    pub data_directory: DataDirectory,
    pub sections: Vec<Section>,
    pub rich_header: Option<RichHeader>,
    pub exports: Option<ExportDirectory>,
    pub imports: Option<ImportDirectory>,
    pub exception: Option<ExceptionDirectory>,
    pub relocs: Option<RelocationDirectory>
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDirectory(pub Vec<ExportEntry>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportEntry {
    pub name: String,
    pub ordinal: u16,
    pub rva: Address,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDirectory(pub Vec<ImportModule>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportModule {
    pub dll_name: String,
    pub functions: Vec<ImportFunction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFunction {
    pub name: String,
    pub hint: usize,
    pub ordinal: u16,
    pub rva: Address,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionDirectory(pub Vec<ExceptionEntry>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionEntry {
    pub begin_rva: Address,
    pub end_rva: Address,
    pub unwind_rva: Address,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocationEntry {
    pub rva: Address,
    pub r#type: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocationBlock {
    pub virtual_address: Address,
    pub entries: Vec<RelocationEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocationDirectory(pub Vec<RelocationBlock>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHeader {
    pub machine: Address,
    pub number_of_sections: Address,
    pub time_date_stamp: TimeDateStamp,
    pub pointer_to_symbol_table: Address,
    pub number_of_symbols: Address,
    pub size_of_optional_header: Address,
    pub characteristics: Characteristics,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeDateStamp(u32);

impl From<u32> for TimeDateStamp {
    fn from(value: u32) -> Self {
        TimeDateStamp(value)
    }
}

impl From<TimeDateStamp> for u32 {
    fn from(value: TimeDateStamp) -> Self {
        value.0
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalHeader {
    pub magic: u16,
    pub linker_version: String,
    pub size_of_code: Address,
    pub size_of_initialized_data: Address,
    pub size_of_uninitialized_data: Address,
    pub address_of_entry_point: Address,
    pub base_of_code: u32,
    // pub base_of_data: u32,
    pub image_base: Address,
    pub section_alignment: Address,
    pub file_alignment: Address,
    pub operating_system_version: Version,
    pub image_version: Version,
    pub subsystem_version: Version,
    pub win32_version_value: u32,
    pub size_of_image: Address,
    pub size_of_headers: Address,
    pub check_sum: u32,
    pub subsystem: Subsystem,
    pub dll_characteristics: DllCharacteristics,
    pub size_of_stack_reserve: Address,
    pub size_of_stack_commit: Address,
    pub size_of_heap_reserve: Address,
    pub size_of_heap_commit: Address,
    pub loader_flags: Address,
    pub number_of_rva_and_sizes: Address,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub name: SectionName,
    pub data: SectionData,
    pub virtual_size: Address,
    pub virtual_address: Address,
    pub size_of_raw_data: Address,
    pub pointer_to_raw_data: Address,
    pub pointer_to_relocations: Address,
    pub pointer_to_linenumbers: Address,
    pub number_of_relocations: Address,
    pub number_of_linenumbers: Address,
    pub characteristics: SectionCharacteristics,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionData(
    #[serde(
        serialize_with = "serialize_section_data",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub Vec<u8>
);

fn serialize_section_data<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if data.is_empty() {
        serializer.serialize_none()
    } else {
        let hex: String = data.iter()
            .map(|b| format!("{:02X}", b))
            .collect();
        serializer.serialize_str(&hex)
    }
}

impl SectionData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        SectionData(bytes.to_vec())
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RichHeader {
    pub image_checksum: u32,
    pub xor_key: u32,
    pub records: Vec<RichRecord>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RichRecord {
    pub id: u16,
    pub version: u16,
    pub count: u32
}

#[derive(Copy, Clone, Default)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}.{}", self.major, self.minor))
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl From<(u16, u16)> for Version {
    fn from((major, minor): (u16, u16)) -> Self {
        Version { major, minor }
    }
}

pub struct SectionName {
    pub valid_utf8: bool,
    pub name: [u8; 8],
}

impl SectionName {
    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        let valid_utf8 = core::str::from_utf8(bytes).is_ok();
        
        SectionName {
            valid_utf8,
            name: *bytes,
        }
    }
}

impl Serialize for SectionName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.valid_utf8 {
            let trimmed = self.name.iter()
                .take_while(|&&b| b != 0)
                .copied()
                .collect::<heapless::Vec<u8, 8>>();
            
            let s = core::str::from_utf8(&trimmed).unwrap_or("");
            serializer.serialize_str(s)
        } else {
            let mut hex = [0u8; 16];
            for (i, &b) in self.name.iter().enumerate() {
                let byte = b as usize;
                hex[i * 2] = hex_char((byte >> 4) & 0xF);
                hex[i * 2 + 1] = hex_char(byte & 0xF);
            }

            let hex_str = core::str::from_utf8(&hex).unwrap_or("????????");
            serializer.serialize_str(hex_str)
        }
    }
}

#[inline]
const fn hex_char(value: usize) -> u8 {
    match value {
        0..=9 => b'0' + value as u8,
        10..=15 => b'a' + (value - 10) as u8,
        _ => b'?',
    }
}

impl core::fmt::Display for SectionName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.valid_utf8 {
            let trimmed = self.name.iter()
                .take_while(|&&b| b != 0)
                .copied()
                .collect::<heapless::Vec<u8, 8>>();
            let s = core::str::from_utf8(&trimmed).unwrap_or("");
            write!(f, "{}", s)
        } else {
            for &b in &self.name {
                write!(f, "{:02X}", b)?;
            }
            Ok(())
        }
    }
}

pub struct SerializedBinaryOptions {
    pub include_rich_header: bool,
    pub include_relocations: bool,
    pub include_debug: bool,
    pub include_section_data: bool,
    pub include_data_directories: bool,
    pub include_exports: bool,
    pub include_imports: bool,
    pub include_exception: bool,
}

impl SerializedBinaryOptions {
    pub const fn all() -> Self {
        Self {
            include_section_data: true,
            include_rich_header: true,
            include_relocations: true,
            include_debug: true,
            include_data_directories: true,
            include_exports: true,
            include_imports: true,
            include_exception: true,
        }
    }
}

impl Default for SerializedBinaryOptions {
    fn default() -> Self {
        Self {
            include_section_data: false,
            include_rich_header: true,
            include_relocations: true,
            include_debug: true,
            include_data_directories: true,
            include_exports: true,
            include_imports: true,
            include_exception: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileCharacteristic {
    RelocsStripped,
    ExecutableImage,
    LineNumsStripped,
    LocalSymsStripped,
    AggressiveWsTrim,
    LargeAddressAware,
    BytesReversedLo,
    Bit32Machine,
    DebugStripped,
    RemovableRunFromSwap,
    NetRunFromSwap,
    System,
    Dll,
    UpSystemOnly,
    BytesReversedHi,
}

#[derive(Debug, Clone, Serialize)]
pub struct Characteristics(pub Vec<FileCharacteristic>);

impl From<u16> for Characteristics {
    fn from(value: u16) -> Self {
        let mut flags = Vec::new();
        if value & IMAGE_FILE_RELOCS_STRIPPED != 0 { flags.push(FileCharacteristic::RelocsStripped); }
        if value & IMAGE_FILE_EXECUTABLE_IMAGE != 0 { flags.push(FileCharacteristic::ExecutableImage); }
        if value & IMAGE_FILE_LINE_NUMS_STRIPPED != 0 { flags.push(FileCharacteristic::LineNumsStripped); }
        if value & IMAGE_FILE_LOCAL_SYMS_STRIPPED != 0 { flags.push(FileCharacteristic::LocalSymsStripped); }
        if value & IMAGE_FILE_AGGRESIVE_WS_TRIM != 0 { flags.push(FileCharacteristic::AggressiveWsTrim); }
        if value & IMAGE_FILE_LARGE_ADDRESS_AWARE != 0 { flags.push(FileCharacteristic::LargeAddressAware); }
        if value & IMAGE_FILE_BYTES_REVERSED_LO != 0 { flags.push(FileCharacteristic::BytesReversedLo); }
        if value & IMAGE_FILE_32BIT_MACHINE != 0 { flags.push(FileCharacteristic::Bit32Machine); }
        if value & IMAGE_FILE_DEBUG_STRIPPED != 0 { flags.push(FileCharacteristic::DebugStripped); }
        if value & IMAGE_FILE_REMOVABLE_RUN_FROM_SWAP != 0 { flags.push(FileCharacteristic::RemovableRunFromSwap); }
        if value & IMAGE_FILE_NET_RUN_FROM_SWAP != 0 { flags.push(FileCharacteristic::NetRunFromSwap); }
        if value & IMAGE_FILE_SYSTEM != 0 { flags.push(FileCharacteristic::System); }
        if value & IMAGE_FILE_DLL != 0 { flags.push(FileCharacteristic::Dll); }
        if value & IMAGE_FILE_UP_SYSTEM_ONLY != 0 { flags.push(FileCharacteristic::UpSystemOnly); }
        if value & IMAGE_FILE_BYTES_REVERSED_HI != 0 { flags.push(FileCharacteristic::BytesReversedHi); }
        Characteristics(flags)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SectionCharacteristic {
    TypeNoPad,
    ContainsCode,
    ContainsInitializedData,
    ContainsUninitializedData,
    LinkerOther,
    LinkerInfo,
    LinkerRemove,
    LinkerComdat,
    GpRel,
    MemPurgeable,
    MemLocked,
    MemPreload,
    Align1Bytes,
    Align2Bytes,
    Align4Bytes,
    Align8Bytes,
    Align16Bytes,
    Align32Bytes,
    Align64Bytes,
    Align128Bytes,
    Align256Bytes,
    Align512Bytes,
    Align1024Bytes,
    Align2048Bytes,
    Align4096Bytes,
    Align8192Bytes,
    LinkerNrelocOvfl,
    MemDiscardable,
    MemNotCached,
    MemNotPaged,
    MemShared,
    MemExecute,
    MemRead,
    MemWrite,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionCharacteristics(pub Vec<SectionCharacteristic>);

impl From<u32> for SectionCharacteristics {
    fn from(value: u32) -> Self {
        let mut flags = Vec::new();
        if value & IMAGE_SCN_TYPE_NO_PAD != 0 { flags.push(SectionCharacteristic::TypeNoPad); }
        if value & IMAGE_SCN_CNT_CODE != 0 { flags.push(SectionCharacteristic::ContainsCode); }
        if value & IMAGE_SCN_CNT_INITIALIZED_DATA != 0 { flags.push(SectionCharacteristic::ContainsInitializedData); }
        if value & IMAGE_SCN_CNT_UNINITIALIZED_DATA != 0 { flags.push(SectionCharacteristic::ContainsUninitializedData); }
        if value & IMAGE_SCN_LNK_OTHER != 0 { flags.push(SectionCharacteristic::LinkerOther); }
        if value & IMAGE_SCN_LNK_INFO != 0 { flags.push(SectionCharacteristic::LinkerInfo); }
        if value & IMAGE_SCN_LNK_REMOVE != 0 { flags.push(SectionCharacteristic::LinkerRemove); }
        if value & IMAGE_SCN_LNK_COMDAT != 0 { flags.push(SectionCharacteristic::LinkerComdat); }
        if value & IMAGE_SCN_GPREL != 0 { flags.push(SectionCharacteristic::GpRel); }
        if value & IMAGE_SCN_MEM_PURGEABLE != 0 { flags.push(SectionCharacteristic::MemPurgeable); }
        if value & IMAGE_SCN_MEM_LOCKED != 0 { flags.push(SectionCharacteristic::MemLocked); }
        if value & IMAGE_SCN_MEM_PRELOAD != 0 { flags.push(SectionCharacteristic::MemPreload); }
        if value & IMAGE_SCN_ALIGN_1BYTES != 0 { flags.push(SectionCharacteristic::Align1Bytes); }
        if value & IMAGE_SCN_ALIGN_2BYTES != 0 { flags.push(SectionCharacteristic::Align2Bytes); }
        if value & IMAGE_SCN_ALIGN_4BYTES != 0 { flags.push(SectionCharacteristic::Align4Bytes); }
        if value & IMAGE_SCN_ALIGN_8BYTES != 0 { flags.push(SectionCharacteristic::Align8Bytes); }
        if value & IMAGE_SCN_ALIGN_16BYTES != 0 { flags.push(SectionCharacteristic::Align16Bytes); }
        if value & IMAGE_SCN_ALIGN_32BYTES != 0 { flags.push(SectionCharacteristic::Align32Bytes); }
        if value & IMAGE_SCN_ALIGN_64BYTES != 0 { flags.push(SectionCharacteristic::Align64Bytes); }
        if value & IMAGE_SCN_ALIGN_128BYTES != 0 { flags.push(SectionCharacteristic::Align128Bytes); }
        if value & IMAGE_SCN_ALIGN_256BYTES != 0 { flags.push(SectionCharacteristic::Align256Bytes); }
        if value & IMAGE_SCN_ALIGN_512BYTES != 0 { flags.push(SectionCharacteristic::Align512Bytes); }
        if value & IMAGE_SCN_ALIGN_1024BYTES != 0 { flags.push(SectionCharacteristic::Align1024Bytes); }
        if value & IMAGE_SCN_ALIGN_2048BYTES != 0 { flags.push(SectionCharacteristic::Align2048Bytes); }
        if value & IMAGE_SCN_ALIGN_4096BYTES != 0 { flags.push(SectionCharacteristic::Align4096Bytes); }
        if value & IMAGE_SCN_ALIGN_8192BYTES != 0 { flags.push(SectionCharacteristic::Align8192Bytes); }
        if value & IMAGE_SCN_LNK_NRELOC_OVFL != 0 { flags.push(SectionCharacteristic::LinkerNrelocOvfl); }
        if value & IMAGE_SCN_MEM_DISCARDABLE != 0 { flags.push(SectionCharacteristic::MemDiscardable); }
        if value & IMAGE_SCN_MEM_NOT_CACHED != 0 { flags.push(SectionCharacteristic::MemNotCached); }
        if value & IMAGE_SCN_MEM_NOT_PAGED != 0 { flags.push(SectionCharacteristic::MemNotPaged); }
        if value & IMAGE_SCN_MEM_SHARED != 0 { flags.push(SectionCharacteristic::MemShared); }
        if value & IMAGE_SCN_MEM_EXECUTE != 0 { flags.push(SectionCharacteristic::MemExecute); }
        if value & IMAGE_SCN_MEM_READ != 0 { flags.push(SectionCharacteristic::MemRead); }
        if value & IMAGE_SCN_MEM_WRITE != 0 { flags.push(SectionCharacteristic::MemWrite); }
        SectionCharacteristics(flags)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DllCharacteristic {
    HighEntropyVa,
    DynamicBase,
    ForceIntegrity,
    NxCompat,
    NoIsolation,
    NoSeh,
    NoBind,
    Appcontainer,
    WdmDriver,
    GuardCf,
    TerminalServerAware,
}

#[derive(Debug, Clone, Serialize)]
pub struct DllCharacteristics(pub Vec<DllCharacteristic>);

impl From<u16> for DllCharacteristics {
    fn from(value: u16) -> Self {
        let mut flags = Vec::new();
        if value & IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA != 0 { flags.push(DllCharacteristic::HighEntropyVa); }
        if value & IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE != 0 { flags.push(DllCharacteristic::DynamicBase); }
        if value & IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY != 0 { flags.push(DllCharacteristic::ForceIntegrity); }
        if value & IMAGE_DLLCHARACTERISTICS_NX_COMPAT != 0 { flags.push(DllCharacteristic::NxCompat); }
        if value & IMAGE_DLLCHARACTERISTICS_NO_ISOLATION != 0 { flags.push(DllCharacteristic::NoIsolation); }
        if value & IMAGE_DLLCHARACTERISTICS_NO_SEH != 0 { flags.push(DllCharacteristic::NoSeh); }
        if value & IMAGE_DLLCHARACTERISTICS_NO_BIND != 0 { flags.push(DllCharacteristic::NoBind); }
        if value & IMAGE_DLLCHARACTERISTICS_APPCONTAINER != 0 { flags.push(DllCharacteristic::Appcontainer); }
        if value & IMAGE_DLLCHARACTERISTICS_WDM_DRIVER != 0 { flags.push(DllCharacteristic::WdmDriver); }
        if value & IMAGE_DLLCHARACTERISTICS_GUARD_CF != 0 { flags.push(DllCharacteristic::GuardCf); }
        if value & IMAGE_DLLCHARACTERISTICS_TERMINAL_SERVER_AWARE != 0 { flags.push(DllCharacteristic::TerminalServerAware); }
        DllCharacteristics(flags)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Subsystem {
    Unknown,
    Native,
    WindowsGui,
    WindowsCui,
    Os2Cui,
    PosixCui,
    NativeWindows,
    WindowsCeGui,
    EfiApplication,
    EfiBootServiceDriver,
    EfiRuntimeDriver,
    EfiRom,
    Xbox,
    WindowsBootApplication,
    Other(u16),
}

impl From<u16> for Subsystem {
    fn from(value: u16) -> Self {
        match value {
            IMAGE_SUBSYSTEM_UNKNOWN => Subsystem::Unknown,
            IMAGE_SUBSYSTEM_NATIVE => Subsystem::Native,
            IMAGE_SUBSYSTEM_WINDOWS_GUI => Subsystem::WindowsGui,
            IMAGE_SUBSYSTEM_WINDOWS_CUI => Subsystem::WindowsCui,
            IMAGE_SUBSYSTEM_OS2_CUI => Subsystem::Os2Cui,
            IMAGE_SUBSYSTEM_POSIX_CUI => Subsystem::PosixCui,
            IMAGE_SUBSYSTEM_NATIVE_WINDOWS => Subsystem::NativeWindows,
            IMAGE_SUBSYSTEM_WINDOWS_CE_GUI => Subsystem::WindowsCeGui,
            IMAGE_SUBSYSTEM_EFI_APPLICATION => Subsystem::EfiApplication,
            IMAGE_SUBSYSTEM_EFI_BOOT_SERVICE_DRIVER => Subsystem::EfiBootServiceDriver,
            IMAGE_SUBSYSTEM_EFI_RUNTIME_DRIVER => Subsystem::EfiRuntimeDriver,
            IMAGE_SUBSYSTEM_EFI_ROM => Subsystem::EfiRom,
            IMAGE_SUBSYSTEM_XBOX => Subsystem::Xbox,
            IMAGE_SUBSYSTEM_WINDOWS_BOOT_APPLICATION => Subsystem::WindowsBootApplication,
            _ => Subsystem::Other(value),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialOrd, Ord, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DataDirectoryType {
    Export,
    Import,
    Resource,
    Exception,
    Security,
    Basereloc,
    Debug,
    Architecture,
    Globalptr,
    Tls,
    LoadConfig,
    BoundImport,
    Iat,
    DelayImport,
    ComDescriptor,
    Other(usize),
}

impl From<usize> for DataDirectoryType {
    fn from(index: usize) -> Self {
        match index {
            0 => DataDirectoryType::Export,
            1 => DataDirectoryType::Import,
            2 => DataDirectoryType::Resource,
            3 => DataDirectoryType::Exception,
            4 => DataDirectoryType::Security,
            5 => DataDirectoryType::Basereloc,
            6 => DataDirectoryType::Debug,
            7 => DataDirectoryType::Architecture,
            8 => DataDirectoryType::Globalptr,
            9 => DataDirectoryType::Tls,
            10 => DataDirectoryType::LoadConfig,
            11 => DataDirectoryType::BoundImport,
            12 => DataDirectoryType::Iat,
            13 => DataDirectoryType::DelayImport,
            14 => DataDirectoryType::ComDescriptor,
            _ => DataDirectoryType::Other(index),
        }
    }
}

impl DataDirectoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataDirectoryType::Export => "export",
            DataDirectoryType::Import => "import",
            DataDirectoryType::Resource => "resource",
            DataDirectoryType::Exception => "exception",
            DataDirectoryType::Security => "security",
            DataDirectoryType::Basereloc => "basereloc",
            DataDirectoryType::Debug => "debug",
            DataDirectoryType::Architecture => "architecture",
            DataDirectoryType::Globalptr => "globalptr",
            DataDirectoryType::Tls => "tls",
            DataDirectoryType::LoadConfig => "load_config",
            DataDirectoryType::BoundImport => "bound_import",
            DataDirectoryType::Iat => "iat",
            DataDirectoryType::DelayImport => "delay_import",
            DataDirectoryType::ComDescriptor => "com_descriptor",
            DataDirectoryType::Other(_) => "other",
        }
    }
}

impl core::fmt::Display for DataDirectoryType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataDirectoryEntry {
    #[serde(skip)]
    pub directory_type: DataDirectoryType,
    pub virtual_address: Address,
    pub size: Address,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataDirectory(pub BTreeMap<DataDirectoryType, DataDirectoryEntry>);