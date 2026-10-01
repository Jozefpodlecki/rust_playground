use core::mem::size_of;
use core::ptr::read_unaligned;

use crate::types::*;
use super::enums::*;

impl<'a> DmiEntry<'a> {
    pub fn string(&self, table: &'a [u8], index: u8) -> Option<&'a str> {
        strings::get(table, self.strings_offset, index)
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

impl<'a> BiosInformation<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x12 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);
        Some(Self {
            vendor: s(b[0x00]),
            version: s(b[0x04]),
            release_date: s(b[0x08]),
        })
    }
}

impl<'a> SystemInformation<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x08 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);

        let uuid = if b.len() >= 0x14 {
            let mut raw = [0u8; 16];
            raw.copy_from_slice(&b[0x04..0x14]);
            let u = Uuid::from_smbios(&raw);
            if u.is_nil() || u.is_all_ones() { None } else { Some(u) }
        } else {
            None
        };

        Some(Self {
            manufacturer: s(b[0x00]),
            product_name: s(b[0x01]),
            version: s(b[0x02]),
            serial: s(b[0x03]),
            uuid,
            wake_up_type: WakeUpType(if b.len() > 0x18 { b[0x18] } else { 0 }),
            sku: if b.len() > 0x19 { s(b[0x19]) } else { None },
            family: if b.len() > 0x1A { s(b[0x1A]) } else { None },
        })
    }
}

impl<'a> BaseboardInformation<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x08 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);
        Some(Self {
            manufacturer: s(b[0x00]),
            product: s(b[0x01]),
            version: s(b[0x02]),
            serial: s(b[0x03]),
            asset_tag: s(b[0x04]),
        })
    }
}

impl<'a> ChassisInformation<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x09 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);
        Some(Self {
            manufacturer: s(b[0x00]),
            version: s(b[0x02]),
            serial: s(b[0x03]),
            asset_tag: s(b[0x04]),
            kind: ChassisKind(b[0x01]),
        })
    }
}

impl<'a> ProcessorInformation<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x1A {
            return None;
        }
        let s = |i: u8| entry.string(table, i);

        let family = if b[0x02] == 0xFF && b.len() > 0x29 {
            u16::from_le_bytes([b[0x28], b[0x29]])
        } else {
            b[0x02] as u16
        };

        let core_count = if b.len() > 0x27 && b[0x1F] == 0xFF {
            u16::from_le_bytes([b[0x26], b[0x27]])
        } else if b.len() > 0x1F {
            b[0x1F] as u16
        } else {
            0
        };

        let thread_count = if b.len() > 0x2B && b[0x21] == 0xFF {
            u16::from_le_bytes([b[0x2A], b[0x2B]])
        } else if b.len() > 0x21 {
            b[0x21] as u16
        } else {
            0
        };

        Some(Self {
            socket: s(b[0x00]),
            manufacturer: s(b[0x03]),
            version: s(b[0x10]),
            serial: if b.len() > 0x1C { s(b[0x1C]) } else { None },
            asset_tag: if b.len() > 0x1D { s(b[0x1D]) } else { None },
            part_number: if b.len() > 0x1E { s(b[0x1E]) } else { None },
            processor_type: ProcessorType(b[0x01]),
            processor_family: ProcessorFamily(family as u8),
            processor_id: u64::from_le_bytes([
                b[0x07], b[0x08], b[0x09], b[0x0A],
                b[0x0B], b[0x0C], b[0x0D], b[0x0E],
            ]),
            external_clock: u16::from_le_bytes([b[0x0E], b[0x0F]]),
            max_speed: u16::from_le_bytes([b[0x10], b[0x11]]),
            current_speed: u16::from_le_bytes([b[0x12], b[0x13]]),
            core_count,
            thread_count,
        })
    }
}

impl<'a> MemoryDevice<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x15 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);

        let raw_size = u16::from_le_bytes([b[0x08], b[0x09]]);
        let size_mib = if raw_size == 0x7FFF && b.len() >= 0x1F {
            u32::from_le_bytes([b[0x1C], b[0x1D], b[0x1E], b[0x1F]]) & 0x7FFF_FFFF
        } else {
            raw_size as u32
        };

        Some(Self {
            locator: s(b[0x0C]),
            bank_locator: if b.len() > 0x0D { s(b[0x0D]) } else { None },
            manufacturer: if b.len() > 0x13 { s(b[0x13]) } else { None },
            serial: if b.len() > 0x14 { s(b[0x14]) } else { None },
            asset_tag: if b.len() > 0x15 { s(b[0x15]) } else { None },
            part_number: if b.len() > 0x16 { s(b[0x16]) } else { None },
            size_mib,
            memory_type: MemoryDeviceType(if b.len() > 0x0E { b[0x0E] } else { 0 }),
            speed: if b.len() > 0x12 { u16::from_le_bytes([b[0x11], b[0x12]]) } else { 0 },
            configured_speed: if b.len() > 0x1D { u16::from_le_bytes([b[0x1C], b[0x1D]]) } else { 0 },
            form_factor: MemoryFormFactor(if b.len() > 0x0A { b[0x0A] } else { 0 }),
        })
    }
}

impl MemoryArray {
    fn parse(entry: DmiEntry<'_>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x0B {
            return None;
        }

        let cap = u32::from_le_bytes([b[0x03], b[0x04], b[0x05], b[0x06]]);
        let cap = if cap == 0x80000000 && b.len() >= 0x13 {
            u64::from_le_bytes([
                b[0x0B], b[0x0C], b[0x0D], b[0x0E],
                b[0x0F], b[0x10], b[0x11], b[0x12],
            ])
        } else {
            (cap & 0x7FFF_FFFF) as u64
        };

        Some(Self {
            location: MemoryArrayLocation(b[0x00]),
            use_kind: MemoryArrayUse(b[0x01]),
            error_correction: MemoryErrorCorrection(b[0x02]),
            maximum_capacity_kib: cap,
            device_count: u16::from_le_bytes([b[0x09], b[0x0A]]),
        })
    }
}

impl<'a> CacheInformation<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x0F {
            return None;
        }
        let s = |i: u8| entry.string(table, i);

        let config = u16::from_le_bytes([b[0x01], b[0x02]]);
        let installed = u16::from_le_bytes([b[0x05], b[0x06]]);
        let installed2 = if b.len() >= 0x15 {
            u32::from_le_bytes([b[0x11], b[0x12], b[0x13], b[0x14]])
        } else {
            0
        };

        let size_kib = if installed == 0xFFFF && installed2 != 0 {
            installed2
        } else if installed & 0x8000 != 0 {
            ((installed & 0x7FFF) as u32) * 64
        } else {
            (installed & 0x7FFF) as u32
        };

        Some(Self {
            socket_designation: s(b[0x00]),
            level: ((config & 0x7) + 1) as u8,
            kind: CacheKind(b[0x0D]),
            size_kib,
            associativity: CacheAssociativity(b[0x0E]),
        })
    }
}

impl<'a> PortConnector<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x05 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);
        Some(Self {
            internal_designator: s(b[0x00]),
            external_designator: s(b[0x02]),
            port_type: PortType(b[0x04]),
            internal_connector_type: PortType(b[0x01]),
            external_connector_type: PortType(b[0x03]),
        })
    }
}

impl<'a> SystemSlot<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 0x09 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);
        Some(Self {
            designation: s(b[0x00]),
            kind: SlotKind(b[0x01]),
            bus_width: SlotBusWidth(b[0x02]),
            current_usage: SlotUsage(b[0x03]),
            slot_id: u16::from_le_bytes([b[0x06], b[0x07]]),
        })
    }
}

impl<'a> OnboardDevicesExtended<'a> {
    pub const ENTRY_SIZE: usize = 7;

    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < Self::ENTRY_SIZE || b.len() % Self::ENTRY_SIZE != 0 {
            return None;
        }
        let _ = table;
        Some(Self {
            raw: b,
            strings_offset: entry.strings_offset,
            table
        })
    }

    pub fn entries(&self) -> OnboardDeviceIter<'a> {
        OnboardDeviceIter {
            raw: self.raw,
            strings_offset: self.strings_offset,
            table: self.table,
            offset: 0,
        }
    }
}

pub struct OnboardDeviceIter<'a> {
    raw: &'a [u8],
    strings_offset: usize,
    table: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for OnboardDeviceIter<'a> {
    type Item = OnboardDeviceEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + OnboardDevicesExtended::ENTRY_SIZE > self.raw.len() {
            return None;
        }
        let b = &self.raw[self.offset..self.offset + OnboardDevicesExtended::ENTRY_SIZE];
        let entry = OnboardDeviceEntry {
            reference_designation: strings::get(self.table, self.strings_offset, b[0]),
            device_type: b[1],
            device_type_instance: b[2],
            segment_group: u16::from_le_bytes([b[3], b[4]]),
            bus: b[5],
            device_function: b[6],
        };
        self.offset += OnboardDevicesExtended::ENTRY_SIZE;
        Some(entry)
    }
}

impl<'a> TpmDevice<'a> {
    fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Option<Self> {
        let b = entry.body;
        if b.len() < 27 {
            return None;
        }
        let s = |i: u8| entry.string(table, i);

        Some(Self {
            vendor_id: [b[0x00], b[0x01], b[0x02], b[0x03]],
            major_spec_version: b[0x04],
            minor_spec_version: b[0x05],
            firmware_version_1: u32::from_le_bytes([b[0x06], b[0x07], b[0x08], b[0x09]]),
            firmware_version_2: u32::from_le_bytes([b[0x0A], b[0x0B], b[0x0C], b[0x0D]]),
            description: s(b[0x0E]),
            characteristics: u64::from_le_bytes([
                b[0x0F], b[0x10], b[0x11], b[0x12],
                b[0x13], b[0x14], b[0x15], b[0x16],
            ]),
            oem_defined: u32::from_le_bytes([b[0x17], b[0x18], b[0x19], b[0x1A]]),
        })
    }
}

impl<'a> Structure<'a> {
    pub fn parse(table: &'a [u8], entry: DmiEntry<'a>) -> Self {
        let kind = entry.header.kind;
        let dmi_header_kind = DmiHeaderKind::from_byte(kind);
        let length = entry.header.length;

        match dmi_header_kind {
            DmiHeaderKind::Bios => BiosInformation::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::Bios),
            DmiHeaderKind::System => SystemInformation::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::System),
            DmiHeaderKind::Baseboard => BaseboardInformation::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::Baseboard),
            DmiHeaderKind::Chassis => ChassisInformation::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::Chassis),
            DmiHeaderKind::Processor => ProcessorInformation::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::Processor),
            DmiHeaderKind::Cache => CacheInformation::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::Cache),
            DmiHeaderKind::PortConnector => PortConnector::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::PortConnector),
            DmiHeaderKind::SystemSlot => SystemSlot::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::SystemSlot),
            DmiHeaderKind::MemoryArray => MemoryArray::parse(entry)
                .map_or(Structure::Unknown { kind, length }, Structure::MemoryArray),
            DmiHeaderKind::MemoryDevice => MemoryDevice::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::MemoryDevice),
            DmiHeaderKind::EndOfTable => Structure::EndOfTable,
            DmiHeaderKind::Other(_) => Structure::Unknown { kind, length },
            DmiHeaderKind::OnboardDevicesExtended => OnboardDevicesExtended::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::OnboardDevicesExtended),
            DmiHeaderKind::TpmDevice => TpmDevice::parse(table, entry)
                .map_or(Structure::Unknown { kind, length }, Structure::TpmDevice),
        }
    }
}

impl<'a> core::fmt::Display for Structure<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Structure::Bios(b) => {
                writeln!(f, "BIOS Information:")?;
                if let Some(x) = b.vendor { writeln!(f, "  Vendor: {}", x)?; }
                if let Some(x) = b.version { writeln!(f, "  Version: {}", x)?; }
                if let Some(x) = b.release_date { writeln!(f, "  Release Date: {}", x)?; }
                Ok(())
            }
            Structure::System(s) => {
                writeln!(f, "System Information:")?;
                if let Some(x) = s.manufacturer { writeln!(f, "  Manufacturer: {}", x)?; }
                if let Some(x) = s.product_name { writeln!(f, "  Product: {}", x)?; }
                if let Some(x) = s.version { writeln!(f, "  Version: {}", x)?; }
                if let Some(x) = s.serial { writeln!(f, "  Serial: {}", x)?; }
                if let Some(x) = s.sku { writeln!(f, "  SKU: {}", x)?; }
                if let Some(x) = s.family { writeln!(f, "  Family: {}", x)?; }
                if let Some(u) = s.uuid { writeln!(f, "  UUID: {}", u)?; }
                writeln!(f, "  Wake-Up: {}", s.wake_up_type)?;
                Ok(())
            }
            Structure::Baseboard(b) => {
                writeln!(f, "Baseboard:")?;
                if let Some(x) = b.manufacturer { writeln!(f, "  Manufacturer: {}", x)?; }
                if let Some(x) = b.product { writeln!(f, "  Product: {}", x)?; }
                if let Some(x) = b.version { writeln!(f, "  Version: {}", x)?; }
                if let Some(x) = b.serial { writeln!(f, "  Serial: {}", x)?; }
                if let Some(x) = b.asset_tag { writeln!(f, "  Asset Tag: {}", x)?; }
                Ok(())
            }
            Structure::Chassis(c) => {
                writeln!(f, "Chassis: {}", c.kind)?;
                if let Some(x) = c.manufacturer { writeln!(f, "  Manufacturer: {}", x)?; }
                if let Some(x) = c.version { writeln!(f, "  Version: {}", x)?; }
                if let Some(x) = c.serial { writeln!(f, "  Serial: {}", x)?; }
                if let Some(x) = c.asset_tag { writeln!(f, "  Asset Tag: {}", x)?; }
                Ok(())
            }
            Structure::Processor(p) => {
                writeln!(f, "Processor:")?;
                if let Some(x) = p.socket { writeln!(f, "  Socket: {}", x)?; }
                if let Some(x) = p.manufacturer { writeln!(f, "  Manufacturer: {}", x)?; }
                if let Some(x) = p.version { writeln!(f, "  Version: {}", x)?; }
                if let Some(x) = p.part_number { writeln!(f, "  Part Number: {}", x)?; }
                writeln!(f, "  Type: {}", p.processor_type)?;
                writeln!(f, "  Family: {}", p.processor_family)?;
                writeln!(f, "  External Clock: {} MHz", p.external_clock)?;
                writeln!(f, "  Max Speed: {} MHz", p.max_speed)?;
                writeln!(f, "  Current Speed: {} MHz", p.current_speed)?;
                writeln!(f, "  Cores: {}", p.core_count)?;
                writeln!(f, "  Threads: {}", p.thread_count)?;
                Ok(())
            }
            Structure::MemoryArray(m) => {
                writeln!(f, "Memory Array:")?;
                writeln!(f, "  Location: {}", m.location)?;
                writeln!(f, "  Use: {}", m.use_kind)?;
                writeln!(f, "  Error Correction: {}", m.error_correction)?;
                writeln!(f, "  Maximum Capacity: {} MiB", m.maximum_capacity_kib / 1024)?;
                writeln!(f, "  Devices: {}", m.device_count)?;
                Ok(())
            }
            Structure::MemoryDevice(m) => {
                writeln!(f, "Memory Device:")?;
                if let Some(x) = m.locator { writeln!(f, "  Locator: {}", x)?; }
                if let Some(x) = m.bank_locator { writeln!(f, "  Bank: {}", x)?; }
                if let Some(x) = m.manufacturer { writeln!(f, "  Manufacturer: {}", x)?; }
                if let Some(x) = m.part_number { writeln!(f, "  Part Number: {}", x)?; }
                writeln!(f, "  Size: {} MiB", m.size_mib)?;
                writeln!(f, "  Type: {}", m.memory_type)?;
                writeln!(f, "  Form Factor: {}", m.form_factor)?;
                writeln!(f, "  Speed: {} MT/s", m.speed)?;
                if m.configured_speed != 0 {
                    writeln!(f, "  Configured Speed: {} MT/s", m.configured_speed)?;
                }
                Ok(())
            }
            Structure::Cache(c) => {
                writeln!(f, "Cache:")?;
                if let Some(x) = c.socket_designation { writeln!(f, "  Designation: {}", x)?; }
                writeln!(f, "  Level: {}", c.level)?;
                writeln!(f, "  Kind: {}", c.kind)?;
                writeln!(f, "  Size: {} KiB", c.size_kib)?;
                writeln!(f, "  Associativity: {}", c.associativity)?;
                Ok(())
            }
            Structure::PortConnector(p) => {
                writeln!(f, "Port Connector:")?;
                if let Some(x) = p.internal_designator { writeln!(f, "  Internal: {}", x)?; }
                if let Some(x) = p.external_designator { writeln!(f, "  External: {}", x)?; }
                writeln!(f, "  Type: {}", p.port_type)?;
                writeln!(f, "  Internal Connector: {}", p.internal_connector_type)?;
                writeln!(f, "  External Connector: {}", p.external_connector_type)?;
                Ok(())
            }
            Structure::SystemSlot(s) => {
                writeln!(f, "System Slot:")?;
                if let Some(x) = s.designation { writeln!(f, "  Designation: {}", x)?; }
                writeln!(f, "  Kind: {}", s.kind)?;
                writeln!(f, "  Bus Width: {}", s.bus_width)?;
                writeln!(f, "  Usage: {}", s.current_usage)?;
                writeln!(f, "  Slot ID: {}", s.slot_id)?;
                Ok(())
            }
            Structure::OnboardDevicesExtended(o) => {
                writeln!(f, "Onboard Devices Extended:")?;
                for e in o.entries() {
                    if let Some(x) = e.reference_designation {
                        writeln!(f, "  Designation: {}", x)?;
                    }
                    writeln!(f, "  Type: {} instance {}", e.device_type, e.device_type_instance)?;
                    writeln!(
                        f,
                         "  Segment {} Bus {} Device {} Function {}",
                        e.segment_group, e.bus, e.device_type_instance, e.device_function
                    )?;
                }
                Ok(())
            }
            Structure::TpmDevice(t) => {
                writeln!(f, "TPM Device:")?;
                let vendor = core::str::from_utf8(&t.vendor_id).unwrap_or("????");
                writeln!(f, "  Vendor ID: {}", vendor)?;
                writeln!(f, "  Spec Version: {}.{}", t.major_spec_version, t.minor_spec_version)?;
                writeln!(f, "  Firmware: {:#010X} {:#010X}", t.firmware_version_1, t.firmware_version_2)?;
                if let Some(x) = t.description {
                    writeln!(f, "  Description: {}", x)?;
                }
                writeln!(f, "  Characteristics: {:#018X}", t.characteristics)?;
                writeln!(f, "  OEM Defined: {:#010X}", t.oem_defined)?;
                Ok(())
            }
            Structure::EndOfTable => writeln!(f, "End of table"),
            Structure::Unknown { kind, length } => {
                writeln!(f, "Unknown structure kind={} length={}", kind, length)
            }
        }
    }
}

pub fn iter<'a>(table: &'a [u8]) -> impl Iterator<Item = Structure<'a>> + 'a {
    DmiIter::new(table).map(move |e| Structure::parse(table, e))
}