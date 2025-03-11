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
('848f3db0-4951-4c5a-b67a-b20f703e9528', CURRENT_TIMESTAMP, 'Kazeros Raid', 'Echidna', 1, []),
('333eef23-bd9a-4ba7-9b1e-7ad99f2ea310', CURRENT_TIMESTAMP, 'Kazeros Raid', 'Echidna', 2, [37502, 37821]);

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
('333eef23-bd9a-4ba7-9b1e-7ad99f2ea310', CURRENT_TIMESTAMP, 'Covetous Master Echidna', 307015, 0, '333eef23-bd9a-4ba7-9b1e-7ad99f2ea310')
