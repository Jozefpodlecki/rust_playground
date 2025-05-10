CREATE TABLE exercise (
    id NVARCHAR(40) PRIMARY KEY,
    name NVARCHAR(20) NOT NULL,
    markdown NVARCHAR(20) NOT NULL,
    created_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE exercise_session (
    id NVARCHAR(40) PRIMARY KEY,
    exercise_id NVARCHAR(40) NOT NULL,
    folder_path NVARCHAR(20) NOT NULL,
    started_on DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_on DATETIME NULL,
    FOREIGN KEY (exercise_id) REFERENCES exercise(id)
);

INSERT INTO exercise (id, name, markdown)
VALUES
('550e8400-e29b-41d4-a716-446655440000', 'Background Worker', '1_background_worker.md');