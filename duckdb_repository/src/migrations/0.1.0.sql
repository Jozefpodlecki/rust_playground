CREATE TABLE Config(
    version TEXT PRIMARY KEY
)

CREATE TABLE Player(
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    created_on TIMESTAMP NOT NULL
)

INSERT INTO Config
(version)
VALUES ("0.1.0")