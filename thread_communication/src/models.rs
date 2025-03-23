use chrono::{DateTime, Duration, Utc};


#[derive(Debug, Copy, Clone)]
pub enum Message {
    Damage {
        source_id: i64,
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SkillType {
    Normal,
    HyperAwakeningTechnique,
    Awakening,
    HyperAwakening
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SkillEffectType {
    Damage,
    SelfBuff,
    PartyBuff,
    AddStatusEffect
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SkillBuffType {
    DamageAmplification(f32)
}

#[derive(Debug, Clone)]
pub struct SkillBuff {
    pub id: u32,
    pub kind: SkillBuffType,
    pub duration: Duration
}

#[derive(Debug, Clone)]
pub struct SkillEffect {
    pub id: u32,
    pub ratio: i32,
    pub kind: SkillEffectType,
    pub buff: Option<SkillBuff>
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub id: u32,
    pub kind: SkillType,
    pub name: &'static str,
    pub effects: Vec<SkillEffect>,
    pub cast_time: Duration,
    pub cooldown: Duration
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub skills: Vec<Skill>,
    pub cooldown_reduction: f32,
    pub attack_power: i64,
    pub crit_rate: f32,
}

#[derive(Debug, Clone)]
pub struct SkillInstance<'a> {
    pub skill: &'a Skill,
    pub effective_cooldown: Duration,
    pub has_quick_recharge: bool,
    pub priority: u8,
}

pub struct SkillCooldown {
    pub expires_on: DateTime<Utc>,
    pub duration: Duration,
}

impl<'a> SkillInstance<'a> {
    pub fn can_use(&self, elapsed: Duration, has_active_self_buff: bool) -> bool {
        if self.skill.name == "Ragna Deathblade" {
            return elapsed >= Duration::minutes(3) && has_active_self_buff;
        }

        true
    }
}