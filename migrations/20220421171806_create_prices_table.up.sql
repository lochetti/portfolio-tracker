CREATE TABLE IF NOT EXISTS prices (
            id      INTEGER PRIMARY KEY,
            ticker  TEXT NOT NULL,
            date    TEXT NOT NULL,
            price   TEXT NOT NULL,
            UNIQUE (ticker, date)
);