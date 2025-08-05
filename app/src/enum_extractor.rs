use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use quick_xml::Reader;
use quick_xml::events::Event;
use anyhow::*;
use once_cell::sync::Lazy;

pub static WORDS_MAP: Lazy<HashSet<&str>> = Lazy::new(|| {
    [
        "object", "timer", "type", "damage", "font", "group",
        "colosseum", "stat", "move", "result", "town", "npc",
        "pool", "paid", "product", "cost", "expire", "achievement",
        "category", "battlefield", "team", "ai", "compare", "position",
        "skill", "area", "affect", "dir", "money", "item", "flag",
        "guild", "resource", "slang", "action", "story", "pass",
        "mission", "status", "kick", "check", "update", "contribution",
        "interaction", "volume", "leave", "stay", "pvp", "vehicle",
        "music", "play", "common", "use", "be", "damaged",
        "zone", "category", "dungeon", "party", "balance", "guestbook", "quest", "visibility",
        "view", "check", "voyage", "liner", "state", "tutorial", "mode", "grade", "effect",
        "heal", "life", "skill", "section", "tier", "point", "subtract", "transaction", "entry",
        "plan", "wound", "event", "account", "condition", "map", "symbol", "content", "socket",
        "amplify", "research", "chase", "waypoint", "card", "ability", "filter", "operation",
        "object", "size", "level", "absolute", "schedule", "selfie", "sort", "detail", "menu",
        "player", "supply", "wave", "spawn", "level", "low", "rank", "stage", "checker", "mood",
        "add", "origin", "coop", "people", "failure", "remove", "identity", "proc", "dialog",
        "room", "return", "service", "timing", "bar", "decal", "cancel", "bind", "open",
        "chat", "member", "grade", "persistent", "trigger", "complete", "rune", "summon",
        "position", "invoker", "assembly", "training", "bind", "mod", "state", "packet",
        "thread", "secret", "dungeon", "shape", "epic", "enable", "market", "gift", "private",
        "prop", "cinematic", "sync", "disable", "dispatch", "condition", "relation", "shuffle",
        "event", "index", "filter", "cli", "func", "apply", "method", "reward", "boost",
        "option", "fx", "flag", "community", "name", "change", "trade", "enter", "rotation",
        "island", "auction", "trophy", "leave", "reason", "collecting", "puzzle", "unlock",
        "period", "team", "match", "lobby", "troop", "wallpaper", "auction", "trophy", "remove",
        "gamepad", "vibration", "skill", "constraint", "voyage", "continent", "quest", "branch",
        "feature", "apply", "target", "coop", "competition", "reward", "member", "monitoring",
        "zone", "item", "durability", "ability", "buddy", "penalty", "excess", "property",
        "ref", "table", "projectile", "trace", "idle", "move", "aossu", "event", "condition",
        "playguide", "pc", "delete", "code", "medal", "rank", "mod", "wave", "colosseum",
        "team", "observer", "mail", "reason", "welcome", "boost", "card", "position", "warboard",
        "kurzan", "active", "stronghold", "mvp", "file", "version", "teleport", "identity",
        "gauge", "update", "part", "shape", "notice", "matching", "adjustable", "ui", "tutorial",
        "next", "step", "summary", "eventshop", "barter", "map", "dungeon", "entry", "cost",
        "action", "exec", "aux", "password", "contents", "transcendence", "special", "npc",
        "spawn", "location", "stat", "change", "interaction", "camera", "trinity", "orb", "param",
        "asset", "view", "material", "weapon", "attribute", "friendship", "additional", "reverse",
        "ruin", "grade", "disassembly", "output", "feature", "system", "option", "direction",
        "floating", "matter", "realm", "ratio", "rotate", "unconditional", "diratk", "priority",
        "warp", "start", "pvp", "weekly", "expire", "patrol", "equip", "hide", "server", "time",
        "disguise", "transparent", "sequence", "command", "ace", "safe", "level", "growth",
        "mission", "chaos", "gate", "summoned", "show", "event", "remove", "auction", "product",
        "conditional", "scale", "bank", "data", "addend", "memory", "pos", "query", "info",
        "ship", "hunting", "chase", "tint", "gloss", "partyboard", "epic", "front", "hud",
        "put", "stage", "remove", "option", "common", "point", "origin", "room", "change",
        "reason", "login", "security", "warning", "dispatch", "troop", "join", "reject", "faction",
        "broadcast", "content", "revive", "method", "support", "emoticon", "learn", "placement",
        "trigger", "grade", "hit", "rising", "update", "mod", "cue", "stage", "step", "transit",
        "open", "match", "enlist", "mod", "signal", "voice", "chat", "channel", "package",
        "toy", "service", "ark", "passive", "history", "stat", "multiplier", "trap", "integrated",
        "amplify", "distance", "attack", "filter", "braveheart", "contents", "barter", "refund",
        "log", "avatar", "equip", "preset", "index", "limit", "acquire", "custom", "area",
        "direction", "collision", "disable", "death", "door", "damage", "sharing", "cooldown",
        "evolution", "condition", "actor", "pickup", "prop", "life", "concert", "offer",
        "return", "pending", "command", "universe", "ignore", "wave", "complete", "reset",
        "option", "update", "duration", "inferno", "lucky", "trap", "combat", "actor", "admin",
        "request", "target", "set", "cash", "game", "commission", "rage", "aos", "species", "fixed",
        "body", "source", "shop", "rep", "token", "clear", "op", "manage", "give", "afk", "raid",
        "affinity", "purpose", "root", "motion", "slide", "dev", "aura", "chef", "kit", "drop", "entity",
        "slot", "retry", "invincible", "pickaxe", "world", "credit", "unpublish", "blade", "burse",
        "anchor", "battle", "season", "input", "ocean", "moment", "fail", "expanded", "bid", "eac", "crew",
        "ultimate", "chat", "blob", "subregion", "random", "box", "creature", "dispose", "id", "astra",
        "unsummon", "jumping", "auto", "register", "abuse", "class", "polymorph", "neutralize", "business",
        "expedition", "instrument", "announce", "instant", "recall", "pet", "newbie", "quick",
        "profession", "invoke", "tooltip", "restrict", "superarmor", "distributed", "serial", "number",
        "spot", "have", "align", "abort", "victory", "crest", "book", "ore", "sign", "mini", "confront",
        "invite", "track", "record", "rule", "mark", "stance", "phase", "banner", "finish", "purchase",
        "install", "attach", "template", "end", "exit", "sub", "visit", "audit", "bounds", "to", "entry",
        "entrance", "collection", "platinum", "selection", "error", "asset", "immune", "abnormal", "building",
        "multi", "stock", "config", "paralyzation", "guide", "mask", "regress", "loot", "owner", "ship", "main",
        "off", "tendency", "price", "calc", "create", "int", "attr", "online", "override", "gold", "pick",
        "lock", "count", "element", "hint", "kill", "range", "title", "strength", "Msg", "bonus", "per", "search",
        "jump", "distribution", "elixir", "present", "milestone", "basic", "offset", "tribe", "border",
        "text", "completion", "acquisition", "carrying", "notify", "msg", "cache", "control", "contient",
        "tag", "player", "transfer", "congestion", "sub", "buff", "enhance", "key", "base", "merdiem", "provider",
        "widget", "disband", "cmd", "task", "donation", "heart", "site", "exchange",
        "accessory", "dictionary", "inherit", "polish", "proximity", "stack", "recovery", "produce"
    ].into_iter().collect()
});

pub fn extract_enum_maps_from_xml(enum_path: &Path) -> Result<HashMap<String, HashMap<u32, String>>> {
    let xml = fs::read_to_string(enum_path)?;

    let mut enums: HashMap<String, HashMap<u32, String>> = HashMap::new();
    let mut reader = Reader::from_str(&xml);

    let mut buf = Vec::new();

    while let std::result::Result::Ok(event) = reader.read_event_into(&mut buf) {
        match event {
            Event::Empty(ref e) if e.name().as_ref() == b"NODE" => {
                let mut enum_type = None;
                let mut name = None;
                let mut index = None;

                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"Type" => enum_type = Some(attr.unescape_value().unwrap().into_owned()),
                        b"Name" => name = Some(attr.unescape_value().unwrap().into_owned()),
                        b"Index" => index = Some(attr.unescape_value().unwrap().parse::<u32>().unwrap()),
                        _ => {}
                    }
                }

                 if let (Some(enum_type), Some(variant_name), Some(index)) = (enum_type, name, index) {
                    let enum_name = to_pascal_case(&enum_type);
                    enums
                        .entry(enum_name)
                        .or_insert_with(HashMap::new)
                        .insert(index, variant_name);
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(enums)
}


pub fn to_pascal_case(input: &str) -> String {
    let mut result = String::new();
    let mut i = 0;

    while i < input.len() {
        let mut found = false;

        for j in (i + 1..=input.len()).rev() {
            let part = &input[i..j];
            if WORDS_MAP.contains(part) {
                result.push_str(&capitalize(part));
                i = j;
                found = true;
                break;
            }
        }

        if !found {
            result.push(input[i..=i].chars().next().unwrap());
            i += 1;
        }
    }

    capitalize(&result)
}

pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}