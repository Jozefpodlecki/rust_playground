INSERT INTO Raid
(
    id,
    created_on,
    name,
    sub_name,
    gate,
    zone_ids
)
VALUES
(UUID(), CURRENT_TIMESTAMP, 'Kazeros Raid', 'Echidna', 1, [37502, 37821]);

SET VARIABLE raid_id = (SELECT id FROM Raid WHERE Name = 'Kazeros Raid');

INSERT INTO Npc
(
    id,
    created_on,
    name,
    npc_id,
    npc_type,
    raid_id
)
VALUES
(UUID(), CURRENT_TIMESTAMP, 'Covetous Master Echidna', 307015, 1, getvariable('raid_id'))
