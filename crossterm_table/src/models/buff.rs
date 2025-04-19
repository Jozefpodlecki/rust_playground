use chrono::{DateTime, Duration, Utc};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum BuffCategory {
    #[default]
    Buff,
    Debuff
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum BuffTarget {
    #[default]
    TargetSelf,
    Party
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum BuffType {
    #[default]
    Brand,
    AttackPowerBuff,
    Identity,
    DamageAmplification,
    HyperAwakeningTechnique,
    HyperAwakeningTechniqueOutgoingDamage,
    Shield,
    DamageReduction,
    Other
}


#[derive(Clone, PartialEq)]
pub struct Buff {
    pub instance_id: u32,
    pub target: BuffTarget,
    pub kind: BuffType,
    pub expires_on: DateTime<Utc>,
    pub value: f32
}

#[derive(Default, Debug, Clone)]
pub struct BuffTemplate {
    pub category: BuffCategory,
    pub target: BuffTarget,
    pub kind: BuffType,
    pub duration: Duration,
    pub value: f32
}