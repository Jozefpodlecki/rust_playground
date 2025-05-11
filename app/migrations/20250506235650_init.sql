CREATE TABLE exercise (
    id BLOB PRIMARY KEY,
    name NVARCHAR(20) NOT NULL,
    markdown NVARCHAR(20) NOT NULL,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE exercise_session (
    id BLOB PRIMARY KEY,
    exercise_id BLOB NOT NULL,
    folder_path NVARCHAR(20) NULL,
    started_on DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_on DATETIME NULL,
    FOREIGN KEY (exercise_id) REFERENCES exercise(id)
);

INSERT INTO exercise (id, name, markdown)
VALUES
(X'550e8400e29b41d4a716446655440001', 'Welcome', '1_welcome.md'),
(X'550e8400e29b41d4a716446655440002', 'Background Worker', '2_background_worker.md'),
(X'550e8400e29b41d4a716446655440003', 'Tcp Client', '3_tcp_client.md'),
(X'550e8400e29b41d4a716446655440004', 'REST Countries', '4_rest_countries.md'),
(X'550e8400e29b41d4a716446655440005', 'Tcp Client', '5_tcp_client.md'),
(X'550e8400e29b41d4a716446655440006', 'Tcp Client', '6_tcp_client.md'),
(X'550e8400e29b41d4a716446655440007', 'Tcp Client', '7_tcp_client.md')