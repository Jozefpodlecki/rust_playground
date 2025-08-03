pub const SETUP_SQL: &str = r#"
    INSTALL sqlite;
    LOAD sqlite;

    CREATE SCHEMA data;
    CREATE SCHEMA enum;
    CREATE SCHEMA jss;
    CREATE SCHEMA lpk;
    CREATE SCHEMA assembly;

    CREATE TABLE assembly.LOSTARK
    (
        Address INT NOT NULL,
        Opcode VARCHAR(3) NOT NULL
    );
"#;

pub const SELECT_TOP_1_TABLE_NAME: &str = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name LIMIT 1";

pub const SELECT_TABLE_NAME: &str = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";

pub const POST_WORK_SQL: &str = r#"

VACUUM;
"#;