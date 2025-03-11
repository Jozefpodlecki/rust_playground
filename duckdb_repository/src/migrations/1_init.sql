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
    class_id USMALLINT NOT NULL,
    character_id UINTEGER NOT NULL,
    last_gear_score FLOAT NOT NULL,
    CONSTRAINT PK_Player PRIMARY KEY (id),
    CONSTRAINT UQ_Player_Name UNIQUE(name)
);

CREATE TABLE Zone (
    id INTEGER NOT NULL,
    created_on TIMESTAMP NOT NULL,
    name VARCHAR(40) NOT NULL,
    CONSTRAINT PK_Zone PRIMARY KEY (id),
    CONSTRAINT UQ_Zone_Id_Name UNIQUE(id, name)
);

CREATE TABLE Raid (
    id UUID NOT NULL,
    created_on TIMESTAMP(6) NOT NULL,
    name VARCHAR(30) NOT NULL,
    sub_name VARCHAR(30) NULL,
    gate TINYINT NULL,
    zone_ids UINTEGER[] NOT NULL,
    CONSTRAINT PK_Raid PRIMARY KEY (id),
    CONSTRAINT UQ_Raid_Name_Gate UNIQUE(name, sub_name, gate)
);

CREATE TABLE Confrontation (
    id UUID NOT NULL,
    created_on TIMESTAMP NOT NULL,
    raid_id UUID NOT NULL,
    is_cleared BOOLEAN NOT NULL,
    total_damage_dealt INTEGER NOT NULL,
    total_damage_taken INTEGER NOT NULL,
    duration INTERVAL NOT NULL,
    CONSTRAINT PK_Confrontation PRIMARY KEY (id)
);

CREATE TABLE PlayerStats (
    confrontation_id UUID NOT NULL,
    player_id UUID NOT NULL,
    created_on TIMESTAMP NOT NULL,
    total_damage_taken INTEGER NOT NULL,
    total_damage_dealt INTEGER NOT NULL,
    CONSTRAINT PK_PlayerStats PRIMARY KEY (confrontation_id, player_id)
);

CREATE TABLE Npc (
    id UUID NOT NULL,
    created_on TIMESTAMP NOT NULL,
    name VARCHAR(30) NOT NULL,
    npc_id INTEGER NOT NULL,
    npc_type TINYINT NOT NULL,
    raid_id UUID NOT NULL,
    CONSTRAINT PK_Npc PRIMARY KEY (id),
    CONSTRAINT UQ_Npc_Name UNIQUE(name, npc_id, raid_id),
    CONSTRAINT FK_Npc_Raid FOREIGN KEY (raid_id) REFERENCES Raid(id)
);

CREATE TABLE HpSession (
    id UUID NOT NULL,
    npc_id UUID NOT NULL,
    confrontation_id UUID NOT NULL,
    started_on TIMESTAMP NOT NULL,
    ended_on TIMESTAMP,
    CONSTRAINT PK_HpSession PRIMARY KEY (id),
    CONSTRAINT UQ_HpSession_Npc_Raid UNIQUE (npc_id, confrontation_id)
);

CREATE TABLE HpLog (
    session_id UUID NOT NULL,
    recorded_on TIMESTAMP NOT NULL,
    value INTEGER NOT NULL,
    CONSTRAINT PK_HpLog PRIMARY KEY (session_id, recorded_on),
    CONSTRAINT FK_HpLog_Session FOREIGN KEY (session_id) REFERENCES HpSession(id)
);

COMMIT;