INSERT INTO Raid
(
    id,
    created_on,
    name,
    sub_name,
    gate
)
VALUES
(UUID(), CURRENT_TIMESTAMP, 'Kazeros Raid', 'Echidna', 1)

INSERT INTO Npc
(
    id,
    created_on,
    name,
    npc_type_id,
    raid_id
)
VALUES
(UUID(), CURRENT_TIMESTAMP, )
