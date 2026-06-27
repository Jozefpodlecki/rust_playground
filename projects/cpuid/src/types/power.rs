use core::arch::x86_64::CpuidResult;

#[derive(Debug, Clone, Copy)]
pub struct CpuidPowerInfo {
    pub digital_thermal_sensor: bool,
    pub intel_thermal_monitor: bool,
    pub performance_bias: bool,
    pub hardware_coordination: bool,
    pub energy_performance_preference: bool,
    pub turbo_boost_max_3: bool,
    pub hardware_p_states: bool,
    pub hardware_p_states_guaranteed: bool,
}

impl From<CpuidResult> for CpuidPowerInfo {
    fn from(leaf: CpuidResult) -> Self {
        CpuidPowerInfo {
            digital_thermal_sensor: (leaf.eax & 0x1) != 0,
            intel_thermal_monitor: (leaf.eax & 0x2) != 0,
            performance_bias: (leaf.eax & 0x8) != 0,
            hardware_coordination: (leaf.ecx & 0x1) != 0,
            energy_performance_preference: (leaf.ecx & 0x2) != 0,
            turbo_boost_max_3: (leaf.ecx & 0x4) != 0,
            hardware_p_states: (leaf.ecx & 0x8) != 0,
            hardware_p_states_guaranteed: (leaf.ecx & 0x10) != 0,
        }
    }
}