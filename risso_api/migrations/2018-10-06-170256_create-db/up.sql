
CREATE TABLE IF NOT EXISTS comments (
    tid REFERENCES threads(id),
    id INTEGER PRIMARY KEY,
    parent INTEGER,
    created FLOAT NOT NULL,
    modified FLOAT,
    mode INTEGER,
    remote_addr VARCHAR,
    text VARCHAR,
    author VARCHAR,
    email VARCHAR,
    website VARCHAR,
    likes INTEGER DEFAULT 0,
    dislikes INTEGER DEFAULT 0,
    voters BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS preferences (
   key VARCHAR PRIMARY KEY, value VARCHAR
);

CREATE TABLE IF NOT EXISTS threads (
    id INTEGER PRIMARY KEY, uri VARCHAR(256) UNIQUE, title VARCHAR(256)
);
