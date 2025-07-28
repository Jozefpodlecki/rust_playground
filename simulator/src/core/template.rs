use rand::{rng, Rng};

use crate::core::{player::Class, types::*};

pub fn template_3dps_1support() -> EncounterTemplate {
    let mut rng = rng();

    let template = EncounterTemplate {
        boss: EncounterTemplateBoss {
            npc_id: 1,
            max_hp: 1e9 as i64,
            hp_bars: 3,
        },
        parties: vec![EncounterTemplateParty {
            id: rng.random(),
            members: vec![
                EncounterTemplatePartyMember {
                    id: rng.random(),
                    name: Class::Sorceress.as_ref().to_string(),
                    class_id: Class::Sorceress,
                },
                EncounterTemplatePartyMember {
                    id: rng.random(),
                    name: Class::Gunslinger.as_ref().to_string(),
                    class_id: Class::Gunslinger,
                },
                EncounterTemplatePartyMember {
                    id: rng.random(),
                    name: Class::Berserk.as_ref().to_string(),
                    class_id: Class::Berserk,
                },
                EncounterTemplatePartyMember {
                    id: rng.random(),
                    name: Class::Bard.as_ref().to_string(),
                    class_id: Class::Bard,
                },
            ],
        }],
    };

    template
}