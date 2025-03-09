BEGIN;

CREATE TABLE Config (
    version VARCHAR(20) NOT NULL,
    last_migration VARCHAR(30) NOT NULL,
    updated_on TIMESTAMP NOT NULL,
    CONSTRAINT PK_Config PRIMARY KEY (version)
);

CREATE TABLE Player (
    id UUID NOT NULL,
    created_on TIMESTAMP NOT NULL,
    updated_on TIMESTAMP NOT NULL,
    name VARCHAR(30) NOT NULL,
    class_id TINYINT NOT NULL,
    character_id TINYINT NOT NULL,
    last_gear_score FLOAT NOT NULL,
    CONSTRAINT PK_Player PRIMARY KEY (id),
    CONSTRAINT UQ_Player_name UNIQUE(name)
);

CREATE TABLE Raid (
    id UUID NOT NULL,
    name VARCHAR(30) NOT NULL,
    sub_name VARCHAR(30) NULL,
    gate TINYINT NULL,
    created_on TIMESTAMP NOT NULL,
    CONSTRAINT PK_Raid PRIMARY KEY (id),
    CONSTRAINT UQ_Raid_name UNIQUE(name, sub_name)
);

CREATE TABLE Confrontation (
    id UUID NOT NULL,
    created_on TIMESTAMP NOT NULL,
    raid_id UUID NOT NULL,
    is_cleared BOOLEAN NOT NULL,
    total_damage_dealt INTEGER NOT NULL,
    total_damage_taken INTEGER NOT NULL,
    duration VARCHAR(5) NOT NULL,
    CONSTRAINT PK_Confrontation PRIMARY KEY (id)
)

CREATE TABLE PlayerStats (
    confrontation_id UUID NOT NULL,
    player_id UUID NOT NULL,
    created_on TIMESTAMP NOT NULL,
    total_damage_taken INTEGER NOT NULL,
    total_damage_dealt INTEGER NOT NULL,
    CONSTRAINT PK_PlayerStats PRIMARY KEY (confrontation_id, player_id)
)

CREATE TABLE Npc (
    id UUID NOT NULL,
    created_on TIMESTAMP NOT NULL,
    name VARCHAR(30) NOT NULL,
    npc_type_id INTEGER NOT NULL,
    raid_id UUID NOT NULL,
    CONSTRAINT PK_Npc PRIMARY KEY (id),
    CONSTRAINT UQ_Npc_Name UNIQUE(name),
    CONSTRAINT FK_Npc_Raid FOREIGN KEY (session_id) REFERENCES Raid(id)
);

CREATE TABLE HpSession (
    id UUID PRIMARY KEY,
    npc_id UUID NOT NULL,
    raid_id UUID NOT NULL,
    started_on TIMESTAMP NOT NULL,
    ended_on TIMESTAMP,
    CONSTRAINT PK_HpSession PRIMARY KEY (id),
    CONSTRAINT UQ_HpSession_Npc_Raid UNIQUE (npc_id, raid_id)
);

CREATE TABLE HpLog (
    session_id UUID NOT NULL,
    recorded_on TIMESTAMP NOT NULL,
    value INTEGER NOT NULL,
    CONSTRAINT PK_HpLog PRIMARY KEY (session_id, recorded_on),
    CONSTRAINT FK_HpLog_Session FOREIGN KEY (session_id) REFERENCES HpSession(id)
);

COMMIT;