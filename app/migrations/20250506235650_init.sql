CREATE TABLE exercise (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    difficulty TEXT CHECK (difficulty IN ('easy', 'medium', 'hard')),
    category TEXT,
    created_on DATE DEFAULT CURRENT_DATE
);