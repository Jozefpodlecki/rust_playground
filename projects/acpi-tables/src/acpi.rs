#![allow(non_snake_case)]
use ntapi::ntexapi::{
    SYSTEM_FIRMWARE_TABLE_INFORMATION,
    SystemFirmwareTableEnumerate,
    SystemFirmwareTableInformation,
};
use toolkit::syscalls::NtQuerySystemInformation;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AcpiHeader {
    pub Signature: [u8; 4],
    pub Length: u32,
    pub Revision: u8,
    pub Checksum: u8,
    pub OemId: [u8; 6],
    pub OemTableId: [u8; 8],
    pub OemRevision: u32,
    pub CreatorId: [u8; 4],
    pub CreatorRevision: u32,
}

impl AcpiHeader {
    pub const SIZE: usize = core::mem::size_of::<Self>();
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Dmar {
    pub Header: AcpiHeader,
    pub HostAddressWidth: u8,
    pub Flags: u8,
    pub Reserved: [u8; 10],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct DmarStructureHeader {
    pub Type: u16,
    pub Length: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct DmarDrhd {
    pub Header: DmarStructureHeader,
    pub Flags: u8,
    pub Size: u8,
    pub SegmentNumber: u16,
    pub RegisterBaseAddress: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct DmarDeviceScope {
    pub Type: u8,
    pub Length: u8,
    pub Reserved: u16,
    pub EnumerationId: u8,
    pub StartBusNumber: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct DmarPciPath {
    pub Device: u8,
    pub Function: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Mcfg {
    pub Header: AcpiHeader,
    pub Reserved: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct McfgEntry {
    pub BaseAddress: u64,
    pub PciSegmentGroupNumber: u16,
    pub StartBusNumber: u8,
    pub EndBusNumber: u8,
    pub Reserved: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Facp {
    pub Header: AcpiHeader,
    pub FirmwareCtrl: u32,
    pub Dsdt: u32,
    pub Reserved: u8,
    pub PreferredPmProfile: u8,
    pub SciInt: u16,
    pub SmiCmd: u32,
    pub AcpiEnable: u8,
    pub AcpiDisable: u8,
    pub S4BiosReq: u8,
    pub PstateCnt: u8,
    pub Pm1aEvtBlk: u32,
    pub Pm1bEvtBlk: u32,
    pub Pm1aCntBlk: u32,
    pub Pm1bCntBlk: u32,
    pub Pm2CntBlk: u32,
    pub PmTmrBlk: u32,
    pub Gpe0Blk: u32,
    pub Gpe1Blk: u32,
    pub Pm1EvtLen: u8,
    pub Pm1CntLen: u8,
    pub Pm2CntLen: u8,
    pub PmTmrLen: u8,
    pub Gpe0BlkLen: u8,
    pub Gpe1BlkLen: u8,
    pub Gpe1Base: u8,
    pub CstCnt: u8,
    pub PLvl2Lat: u16,
    pub PLvl3Lat: u16,
    pub FlushSize: u16,
    pub FlushStride: u16,
    pub DutyOffset: u8,
    pub DutyWidth: u8,
    pub DayAlrm: u8,
    pub MonAlrm: u8,
    pub Century: u8,
    pub IaPcBootArch: u16,
    pub Reserved2: u8,
    pub Flags: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Madt {
    pub Header: AcpiHeader,
    pub LocalApicAddress: u32,
    pub Flags: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtEntryHeader {
    pub Type: u8,
    pub Length: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtLocalApic {
    pub Header: MadtEntryHeader,
    pub AcpiProcessorId: u8,
    pub ApicId: u8,
    pub Flags: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MadtIoApic {
    pub Header: MadtEntryHeader,
    pub IoApicId: u8,
    pub Reserved: u8,
    pub IoApicAddress: u32,
    pub GlobalSystemInterruptBase: u32,
}

use core::slice;
use toolkit::println;

pub fn walk_acpi_blob(blob: &[u8]) {
    let mut offset = 0;

    while offset + AcpiHeader::SIZE <= blob.len() {
        let header = unsafe {
            &*(blob.as_ptr().add(offset) as *const AcpiHeader)
        };

        let sig = core::str::from_utf8(&header.Signature).unwrap_or("????");
        let length = header.Length as usize;

        if length < AcpiHeader::SIZE || offset + length > blob.len() {
            break;
        }

        let table = &blob[offset..offset + length];

        match sig {
            "DMAR" => parse_dmar(table),
            "MCFG" => parse_mcfg(table),
            "FACP" => parse_facp(table),
            "APIC" => parse_madt(table),
            _ => {
                println!("ACPI: {} ({} bytes)", sig, length);
            }
        }

        offset += length;
        offset = (offset + 3) & !3;
    }
}

fn parse_dmar(table: &[u8]) {
    if table.len() < core::mem::size_of::<Dmar>() {
        return;
    }

    let dmar = unsafe { &*(table.as_ptr() as *const Dmar) };
    let flags = dmar.Flags;
    let haw = dmar.HostAddressWidth;

    println!("DMAR: HostAddressWidth={} Flags=0x{:02X}", haw, flags);

    let mut offset = core::mem::size_of::<Dmar>();
    while offset + core::mem::size_of::<DmarStructureHeader>() <= table.len() {
        let sh = unsafe {
            &*(table.as_ptr().add(offset) as *const DmarStructureHeader)
        };
        let stype = sh.Type;
        let slen = sh.Length as usize;

        if slen < core::mem::size_of::<DmarStructureHeader>()
            || offset + slen > table.len()
        {
            break;
        }

        match stype {
            0x0000 => parse_drhd(&table[offset..offset + slen]),
            _ => {
                println!("  DMAR struct type {} ({} bytes)", stype, slen);
            }
        }

        offset += slen;
        offset = (offset + 3) & !3;
    }
}

fn parse_drhd(sub: &[u8]) {
    if sub.len() < core::mem::size_of::<DmarDrhd>() {
        return;
    }

    let drhd = unsafe { &*(sub.as_ptr() as *const DmarDrhd) };
    let flags = drhd.Flags;
    let segment = drhd.SegmentNumber;
    let base = drhd.RegisterBaseAddress;

    println!("  DRHD: Segment={} Base=0x{:016X} Flags=0x{:02X}",
        segment, base, flags);

    let mut offset = core::mem::size_of::<DmarDrhd>();
    while offset + core::mem::size_of::<DmarDeviceScope>() <= sub.len() {
        let ds = unsafe {
            &*(sub.as_ptr().add(offset) as *const DmarDeviceScope)
        };
        let dtype = ds.Type;
        let dlen = ds.Length as usize;

        if dlen < core::mem::size_of::<DmarDeviceScope>()
            || offset + dlen > sub.len()
        {
            break;
        }

        println!("    Scope: type={} bus={} enum={}",
            dtype, ds.StartBusNumber, ds.EnumerationId);

        offset += dlen;
    }
}

fn parse_mcfg(table: &[u8]) {
    if table.len() < core::mem::size_of::<Mcfg>() {
        return;
    }

    let mcfg = unsafe { &*(table.as_ptr() as *const Mcfg) };
    let _ = mcfg;

    let mut offset = core::mem::size_of::<Mcfg>();
    while offset + core::mem::size_of::<McfgEntry>() <= table.len() {
        let e = unsafe {
            &*(table.as_ptr().add(offset) as *const McfgEntry)
        };
        let base = e.BaseAddress;
        let seg = e.PciSegmentGroupNumber;
        let start = e.StartBusNumber;
        let end = e.EndBusNumber;

        println!("MCFG: Segment={} Bus[{}-{}] Base=0x{:016X}",
            seg, start, end, base);

        offset += core::mem::size_of::<McfgEntry>();
    }
}

fn parse_facp(table: &[u8]) {
    if table.len() < core::mem::size_of::<Facp>() {
        return;
    }

    let f = unsafe { &*(table.as_ptr() as *const Facp) };
    let sci = f.SciInt;
    let profile = f.PreferredPmProfile;

    println!("FACP: SCI_INT={} PreferredProfile={}", sci, profile);
}

fn parse_madt(table: &[u8]) {
    if table.len() < core::mem::size_of::<Madt>() {
        return;
    }

    let m = unsafe { &*(table.as_ptr() as *const Madt) };
    let lapic = m.LocalApicAddress;

    println!("MADT: LocalApicAddress=0x{:08X}", lapic);

    let mut offset = core::mem::size_of::<Madt>();
    while offset + core::mem::size_of::<MadtEntryHeader>() <= table.len() {
        let h = unsafe {
            &*(table.as_ptr().add(offset) as *const MadtEntryHeader)
        };
        let etype = h.Type;
        let elen = h.Length as usize;

        if elen < core::mem::size_of::<MadtEntryHeader>()
            || offset + elen > table.len()
        {
            break;
        }

        match etype {
            0 => {
                if elen >= core::mem::size_of::<MadtLocalApic>() {
                    let e = unsafe {
                        &*(table.as_ptr().add(offset) as *const MadtLocalApic)
                    };
                    println!("  LAPIC: id={} acpi_id={} flags=0x{:08X}",
                        e.ApicId, e.AcpiProcessorId, unsafe { e.Flags });
                }
            }
            1 => {
                if elen >= core::mem::size_of::<MadtIoApic>() {
                    let e = unsafe {
                        &*(table.as_ptr().add(offset) as *const MadtIoApic)
                    };
                    println!("  IOAPIC: id={} addr=0x{:08X} gsi_base={}",
                        e.IoApicId, unsafe { e.IoApicAddress }, unsafe { e.GlobalSystemInterruptBase });
                }
            }
            _ => {
                println!("  MADT entry type {} ({} bytes)", etype, elen);
            }
        }

        offset += elen;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AcpiSignature(pub [u8; 4]);

impl AcpiSignature {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap_or("????")
    }

    pub fn as_u32(&self) -> u32 {
        u32::from_le_bytes(self.0)
    }
}

impl core::fmt::Display for AcpiSignature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct AcpiEnumIter {
    data: [u8; 4096],
    len: usize,
    offset: usize,
}

impl AcpiEnumIter {
    pub fn new() -> Option<Self> {
        let mut iter = Self {
            data: [0u8; 4096],
            len: 0,
            offset: 0,
        };

        let info_ptr = iter.data.as_mut_ptr() as *mut SYSTEM_FIRMWARE_TABLE_INFORMATION;
        let info = unsafe { &mut *info_ptr };
        info.Action = SystemFirmwareTableEnumerate;
        info.ProviderSignature = u32::from_be_bytes(*b"ACPI");

        let mut expected = 0;
        let status = unsafe {
            NtQuerySystemInformation(
                SystemFirmwareTableInformation,
                iter.data.as_mut_ptr() as *mut _,
                iter.data.len() as u32,
                &mut expected,
            )
        };

        if status != 0 {
            return None;
        }

        iter.len = info.TableBufferLength as usize;
        Some(iter)
    }
}

impl Iterator for AcpiEnumIter {
    type Item = AcpiSignature;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + 4 > self.len {
            return None;
        }

        let info_ptr = self.data.as_ptr() as *const SYSTEM_FIRMWARE_TABLE_INFORMATION;
        let info = unsafe { &*info_ptr };
        let base = unsafe { info.TableBuffer.as_ptr() };

        let mut sig = [0u8; 4];
        unsafe {
            core::ptr::copy_nonoverlapping(base.add(self.offset), sig.as_mut_ptr(), 4);
        }

        self.offset += 4;
        Some(AcpiSignature(sig))
    }
}