CREATE TABLE IF NOT EXISTS users
(
    id          INTEGER PRIMARY KEY NOT NULL,
    name        VARCHAR(20)        NOT NULL,
    email       NVARCHAR(20)        NOT NULL,
    active      BOOLEAN             NOT NULL DEFAULT 0
);