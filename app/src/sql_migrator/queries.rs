pub const SELECT_TOP_1_TABLE_NAME: &str = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name LIMIT 1";

pub const SELECT_TABLE_NAME: &str = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
