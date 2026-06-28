use core::fmt;


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MitigationPolicies(pub u8);

impl fmt::Display for MitigationPolicies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mitigation Policies:")?;
        writeln!(f, "  NX Support Policy: {}", self.nx_support_policy())?;
        writeln!(f, "  SEH Validation Policy: {}", self.seh_validation_policy())?;
        writeln!(f, "  CurDir Devices Skipped For DLLs: {}", self.cur_dir_devices_skipped_for_dlls())?;
        writeln!(f, "  Reserved: {}", self.reserved())?;
        Ok(())
    }
}

impl MitigationPolicies {
    pub fn nx_support_policy(&self) -> u8 {
        (self.0 >> 0) & 0x3
    }

    pub fn seh_validation_policy(&self) -> u8 {
        (self.0 >> 2) & 0x3
    }

    pub fn cur_dir_devices_skipped_for_dlls(&self) -> u8 {
        (self.0 >> 4) & 0x3
    }

    pub fn reserved(&self) -> u8 {
        (self.0 >> 6) & 0x3
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedDataFlags(pub u32);

impl fmt::Display for SharedDataFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        
        if self.dbg_error_port_present() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgErrorPortPresent")?;
            first = false;
        }
        if self.dbg_elevation_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgElevationEnabled")?;
            first = false;
        }
        if self.dbg_virt_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgVirtEnabled")?;
            first = false;
        }
        if self.dbg_installer_detect_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgInstallerDetectEnabled")?;
            first = false;
        }
        if self.dbg_lkg_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgLkgEnabled")?;
            first = false;
        }
        if self.dbg_dyn_processor_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgDynProcessorEnabled")?;
            first = false;
        }
        if self.dbg_console_broker_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgConsoleBrokerEnabled")?;
            first = false;
        }
        if self.dbg_secure_boot_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgSecureBootEnabled")?;
            first = false;
        }
        if self.dbg_multi_session_sku() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgMultiSessionSku")?;
            first = false;
        }
        if self.dbg_multi_users_in_session_sku() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgMultiUsersInSessionSku")?;
            first = false;
        }
        if self.dbg_state_separation_enabled() {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "DbgStateSeparationEnabled")?;
            first = false;
        }
        
        let spare = self.spare_bits();
        if spare != 0 {
            if !first { writeln!(f)?; write!(f, "  ")?; }
            write!(f, "SpareBits: {:#x}", spare)?;
            first = false;
        }
        
        if first {
            write!(f, "None")?;
        }
        
        Ok(())
    }
}

impl SharedDataFlags {
    pub fn dbg_error_port_present(&self) -> bool { (self.0 >> 0) & 1 != 0 }
    pub fn dbg_elevation_enabled(&self) -> bool { (self.0 >> 1) & 1 != 0 }
    pub fn dbg_virt_enabled(&self) -> bool { (self.0 >> 2) & 1 != 0 }
    pub fn dbg_installer_detect_enabled(&self) -> bool { (self.0 >> 3) & 1 != 0 }
    pub fn dbg_lkg_enabled(&self) -> bool { (self.0 >> 4) & 1 != 0 }
    pub fn dbg_dyn_processor_enabled(&self) -> bool { (self.0 >> 5) & 1 != 0 }
    pub fn dbg_console_broker_enabled(&self) -> bool { (self.0 >> 6) & 1 != 0 }
    pub fn dbg_secure_boot_enabled(&self) -> bool { (self.0 >> 7) & 1 != 0 }
    pub fn dbg_multi_session_sku(&self) -> bool { (self.0 >> 8) & 1 != 0 }
    pub fn dbg_multi_users_in_session_sku(&self) -> bool { (self.0 >> 9) & 1 != 0 }
    pub fn dbg_state_separation_enabled(&self) -> bool { (self.0 >> 10) & 1 != 0 }
    pub fn spare_bits(&self) -> u32 { (self.0 >> 11) & 0x1FFFFF }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QpcData(pub u8);

impl QpcData {
    pub fn qpc_bypass_enabled(&self) -> bool { (self.0 >> 0) & 1 != 0 }
    pub fn qpc_shift(&self) -> u8 { (self.0 >> 1) & 0x7F }
}


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuiteMask(pub u32);

impl fmt::Display for SuiteMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        
        if self.contains(Suite::SmallBusiness) {
            if !first { write!(f, ", ")?; }
            write!(f, "SmallBusiness")?;
            first = false;
        }
        if self.contains(Suite::Enterprise) {
            if !first { write!(f, ", ")?; }
            write!(f, "Enterprise")?;
            first = false;
        }
        if self.contains(Suite::BackOffice) {
            if !first { write!(f, ", ")?; }
            write!(f, "BackOffice")?;
            first = false;
        }
        if self.contains(Suite::Communications) {
            if !first { write!(f, ", ")?; }
            write!(f, "Communications")?;
            first = false;
        }
        if self.contains(Suite::Terminal) {
            if !first { write!(f, ", ")?; }
            write!(f, "Terminal")?;
            first = false;
        }
        if self.contains(Suite::SmallBusinessRestricted) {
            if !first { write!(f, ", ")?; }
            write!(f, "SmallBusinessRestricted")?;
            first = false;
        }
        if self.contains(Suite::Embedded) {
            if !first { write!(f, ", ")?; }
            write!(f, "Embedded")?;
            first = false;
        }
        if self.contains(Suite::Datacenter) {
            if !first { write!(f, ", ")?; }
            write!(f, "Datacenter")?;
            first = false;
        }
        if self.contains(Suite::SingleUserTS) {
            if !first { write!(f, ", ")?; }
            write!(f, "SingleUserTS")?;
            first = false;
        }
        if self.contains(Suite::Personal) {
            if !first { write!(f, ", ")?; }
            write!(f, "Personal")?;
            first = false;
        }
        if self.contains(Suite::ServerAppliance) {
            if !first { write!(f, ", ")?; }
            write!(f, "ServerAppliance")?;
            first = false;
        }
        if self.contains(Suite::StorageServer) {
            if !first { write!(f, ", ")?; }
            write!(f, "StorageServer")?;
            first = false;
        }
        if self.contains(Suite::ComputeServer) {
            if !first { write!(f, ", ")?; }
            write!(f, "ComputeServer")?;
            first = false;
        }
        if self.contains(Suite::WHServer) {
            if !first { write!(f, ", ")?; }
            write!(f, "WHServer")?;
            first = false;
        }
        
        if first {
            write!(f, "None")?;
        }
        
        Ok(())
    }
}

impl SuiteMask {
    pub fn contains(&self, suite: Suite) -> bool {
        self.0 & suite as u32 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    SmallBusiness = 0x00000001,
    Enterprise = 0x00000002,
    BackOffice = 0x00000004,
    Communications = 0x00000008,
    Terminal = 0x00000010,
    SmallBusinessRestricted = 0x00000020,
    Embedded = 0x00000040,
    Datacenter = 0x00000080,
    SingleUserTS = 0x00000100,
    Personal = 0x00000200,
    ServerAppliance = 0x00000400,
    StorageServer = 0x00002000,
    ComputeServer = 0x00004000,
    WHServer = 0x00008000,
}
