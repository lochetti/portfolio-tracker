CREATE TABLE IF NOT EXISTS trades (
            id      INTEGER PRIMARY KEY,
            ticker  TEXT NOT NULL,
            date    TEXT NOT NULL,
            type    TEXT NOT NULL,
            amount  INTEGER NOT NULL,
            price   TEXT NOT NULL
);