use core::fmt;
use core::mem::size_of;
use core::ptr::read_unaligned;

use crate::enums::*;

impl DmiHeader {
    pub const SIZE: usize = size_of::<Self>();
}

#[derive(Clone, Debug)]
pub struct DmiEntry<'a> {
    pub header: DmiHeader,
    pub body: &'a [u8],
    pub strings_offset: usize,
}

pub mod strings {
    pub fn get<'a>(table: &'a [u8], offset: usize, index: u8) -> Option<&'a str> {
        if index == 0 || offset >= table.len() {
            return None;
        }

        let mut pos = offset;
        let mut current = 1u8;

        while pos < table.len() && table[pos] != 0 {
            let start = pos;
            while pos < table.len() && table[pos] != 0 {
                pos += 1;
            }

            if current == index {
                return core::str::from_utf8(&table[start..pos]).ok();
            }

            current += 1;
            pos += 1;
        }

        None
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DmiIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> DmiIter<'a> {
    pub fn new(table: &'a [u8]) -> Self {
        Self { data: table, offset: 0 }
    }
}

impl<'a> Iterator for DmiIter<'a> {
    type Item = DmiEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + DmiHeader::SIZE > self.data.len() {
            return None;
        }

        let header = unsafe {
            read_unaligned(self.data.as_ptr().add(self.offset) as *const DmiHeader)
        };

        if header.length < DmiHeader::SIZE as u8 {
            return None;
        }

        let start = self.offset + DmiHeader::SIZE;
        let end = self.offset + header.length as usize;

        if end > self.data.len() || start > end {
            return None;
        }

        let body = &self.data[start..end];
        let strings_offset = end;
        let mut next = end;

        while next + 1 < self.data.len() {
            if self.data[next] == 0 && self.data[next + 1] == 0 {
                next += 2;
                break;
            }
            next += 1;
        }

        if next > self.data.len() {
            next = self.data.len();
        }

        self.offset = next;

        Some(DmiEntry { header, body, strings_offset })
    }
}


#[derive(Clone, Copy, Debug)]
pub struct BiosInformation<'a> {
    pub vendor: Option<&'a str>,
    pub version: Option<&'a str>,
    pub release_date: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
pub struct SystemInformation<'a> {
    pub manufacturer: Option<&'a str>,
    pub product_name: Option<&'a str>,
    pub version: Option<&'a str>,
    pub serial: Option<&'a str>,
    pub uuid: Option<Uuid>,
    pub wake_up_type: WakeUpType,
    pub sku: Option<&'a str>,
    pub family: Option<&'a str>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uuid(pub [u8; 16]);

impl Uuid {
    pub fn from_smbios(raw: &[u8; 16]) -> Self {
        let mut u = [0u8; 16];
        u[0] = raw[3]; u[1] = raw[2]; u[2] = raw[1]; u[3] = raw[0];
        u[4] = raw[5]; u[5] = raw[4];
        u[6] = raw[7]; u[7] = raw[6];
        u[8..16].copy_from_slice(&raw[8..16]);
        Self(u)
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn is_nil(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    pub fn is_all_ones(&self) -> bool {
        self.0.iter().all(|&b| b == 0xFF)
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let u = &self.0;
        write!(
            f,
            "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            u[0], u[1], u[2], u[3],
            u[4], u[5],
            u[6], u[7],
            u[8], u[9],
            u[10], u[11], u[12], u[13], u[14], u[15],
        )
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BaseboardInformation<'a> {
    pub manufacturer: Option<&'a str>,
    pub product: Option<&'a str>,
    pub version: Option<&'a str>,
    pub serial: Option<&'a str>,
    pub asset_tag: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
pub struct ChassisInformation<'a> {
    pub manufacturer: Option<&'a str>,
    pub version: Option<&'a str>,
    pub serial: Option<&'a str>,
    pub asset_tag: Option<&'a str>,
    pub kind: ChassisKind,
}

#[derive(Clone, Copy, Debug)]
pub struct ProcessorInformation<'a> {
    pub socket: Option<&'a str>,
    pub manufacturer: Option<&'a str>,
    pub version: Option<&'a str>,
    pub serial: Option<&'a str>,
    pub asset_tag: Option<&'a str>,
    pub part_number: Option<&'a str>,
    pub processor_type: ProcessorType,
    pub processor_family: ProcessorFamily,
    pub processor_id: u64,
    pub external_clock: u16,
    pub max_speed: u16,
    pub current_speed: u16,
    pub core_count: u16,
    pub thread_count: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryArray {
    pub location: MemoryArrayLocation,
    pub use_kind: MemoryArrayUse,
    pub error_correction: MemoryErrorCorrection,
    pub maximum_capacity_kib: u64,
    pub device_count: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct CacheInformation<'a> {
    pub socket_designation: Option<&'a str>,
    pub level: u8,
    pub kind: CacheKind,
    pub size_kib: u32,
    pub associativity: CacheAssociativity,
}

#[derive(Clone, Copy, Debug)]
pub struct PortConnector<'a> {
    pub internal_designator: Option<&'a str>,
    pub external_designator: Option<&'a str>,
    pub port_type: PortType,
    pub internal_connector_type: PortType,
    pub external_connector_type: PortType,
}

#[derive(Clone, Copy, Debug)]
pub struct SystemSlot<'a> {
    pub designation: Option<&'a str>,
    pub kind: SlotKind,
    pub bus_width: SlotBusWidth,
    pub current_usage: SlotUsage,
    pub slot_id: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct OnboardDeviceEntry<'a> {
    pub reference_designation: Option<&'a str>,
    pub device_type: u8,
    pub device_type_instance: u8,
    pub segment_group: u16,
    pub bus: u8,
    pub device_function: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct OnboardDevicesExtended<'a> {
    pub raw: &'a [u8],
    pub strings_offset: usize,
    pub table: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct TpmDevice<'a> {
    pub vendor_id: [u8; 4],
    pub major_spec_version: u8,
    pub minor_spec_version: u8,
    pub firmware_version_1: u32,
    pub firmware_version_2: u32,
    pub description: Option<&'a str>,
    pub characteristics: u64,
    pub oem_defined: u32,
}


#[derive(Clone, Copy, Debug)]
pub struct EndOfTable;

#[derive(Clone, Debug)]
pub enum Structure<'a> {
    Bios(BiosInformation<'a>),
    System(SystemInformation<'a>),
    Baseboard(BaseboardInformation<'a>),
    Chassis(ChassisInformation<'a>),
    Processor(ProcessorInformation<'a>),
    MemoryDevice(MemoryDevice<'a>),
    MemoryArray(MemoryArray),
    Cache(CacheInformation<'a>),
    PortConnector(PortConnector<'a>),
    SystemSlot(SystemSlot<'a>),
    OnboardDevicesExtended(OnboardDevicesExtended<'a>),
    TpmDevice(TpmDevice<'a>),
    EndOfTable,
    Unknown { kind: u8, length: u8 },
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryDevice<'a> {
    pub locator: Option<&'a str>,
    pub bank_locator: Option<&'a str>,
    pub manufacturer: Option<&'a str>,
    pub serial: Option<&'a str>,
    pub asset_tag: Option<&'a str>,
    pub part_number: Option<&'a str>,
    pub size_mib: u32,
    pub memory_type: MemoryDeviceType,
    pub speed: u16,
    pub configured_speed: u16,
    pub form_factor: MemoryFormFactor,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DmiHeader {
    pub kind: u8,
    pub length: u8,
    pub handle: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmiHeaderKind {
    Bios,
    System,
    Baseboard,
    Chassis,
    Processor,
    Cache,
    PortConnector,
    SystemSlot,
    MemoryArray,
    MemoryDevice,
    OnboardDevicesExtended,
    TpmDevice,
    EndOfTable,
    Other(u8),
}

impl DmiHeaderKind {
    pub fn from_byte(b: u8) -> Self {
        match b {
            0 => Self::Bios,
            1 => Self::System,
            2 => Self::Baseboard,
            3 => Self::Chassis,
            4 => Self::Processor,
            7 => Self::Cache,
            8 => Self::PortConnector,
            9 => Self::SystemSlot,
            16 => Self::MemoryArray,
            17 => Self::MemoryDevice,
            41 => Self::OnboardDevicesExtended,
            43 => Self::TpmDevice,
            127 => Self::EndOfTable,
            other => Self::Other(other),
        }
    }

    pub fn raw(&self) -> u8 {
        match self {
            Self::Bios => 0,
            Self::System => 1,
            Self::Baseboard => 2,
            Self::Chassis => 3,
            Self::Processor => 4,
            Self::Cache => 7,
            Self::PortConnector => 8,
            Self::SystemSlot => 9,
            Self::MemoryArray => 16,
            Self::MemoryDevice => 17,
            Self::OnboardDevicesExtended => 41,
            Self::TpmDevice => 43,
            Self::EndOfTable => 127,
            Self::Other(b) => *b,
        }
    }
}
