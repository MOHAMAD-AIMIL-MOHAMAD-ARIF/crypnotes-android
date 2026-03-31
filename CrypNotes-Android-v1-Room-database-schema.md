\-- 1) Notes
CREATE TABLE notes (
id TEXT PRIMARY KEY NOT NULL,                    -- UUIDv7
title TEXT,                                      -- optional explicit title
payload\_envelope BLOB NOT NULL,                  -- encrypted canonical JSON document
payload\_schema\_version INTEGER NOT NULL,
doc\_schema\_version INTEGER NOT NULL,
encryption\_container\_version INTEGER NOT NULL,
char\_count INTEGER NOT NULL CHECK(char\_count <= 20000),
background\_color\_argb INTEGER NOT NULL DEFAULT 0,
is\_pinned INTEGER NOT NULL DEFAULT 0,
state TEXT NOT NULL CHECK(state IN ('ACTIVE','ARCHIVED','TRASHED')) DEFAULT 'ACTIVE',
created\_at\_epoch\_ms INTEGER NOT NULL,
updated\_at\_epoch\_ms INTEGER NOT NULL,
archived\_at\_epoch\_ms INTEGER,
trashed\_at\_epoch\_ms INTEGER
);



CREATE INDEX idx\_notes\_state ON notes(state);
CREATE INDEX idx\_notes\_pin\_updated ON notes(is\_pinned, updated\_at\_epoch\_ms DESC);
CREATE INDEX idx\_notes\_trashed\_at ON notes(trashed\_at\_epoch\_ms);



\-- 2) Manual/custom ordering
CREATE TABLE note\_order (
note\_id TEXT PRIMARY KEY NOT NULL,
position REAL NOT NULL,                          -- fractional ordering key
updated\_at\_epoch\_ms INTEGER NOT NULL,
FOREIGN KEY(note\_id) REFERENCES notes(id) ON DELETE CASCADE
);



CREATE INDEX idx\_note\_order\_position ON note\_order(position);



\-- 3) Labels
CREATE TABLE labels (
id TEXT PRIMARY KEY NOT NULL,                    -- UUIDv7
name TEXT NOT NULL COLLATE NOCASE,
color\_argb INTEGER,
created\_at\_epoch\_ms INTEGER NOT NULL,
updated\_at\_epoch\_ms INTEGER NOT NULL
);



CREATE UNIQUE INDEX idx\_labels\_name\_unique ON labels(name);



\-- 4) Note-Label join (many-to-many)
CREATE TABLE note\_labels (
note\_id TEXT NOT NULL,
label\_id TEXT NOT NULL,
created\_at\_epoch\_ms INTEGER NOT NULL,
PRIMARY KEY(note\_id, label\_id),
FOREIGN KEY(note\_id) REFERENCES notes(id) ON DELETE CASCADE,
FOREIGN KEY(label\_id) REFERENCES labels(id) ON DELETE CASCADE
);



CREATE INDEX idx\_note\_labels\_label\_id ON note\_labels(label\_id);



\-- 5) Attachments
CREATE TABLE attachments (
id TEXT PRIMARY KEY NOT NULL,                    -- UUIDv7
note\_id TEXT NOT NULL,
type TEXT NOT NULL CHECK(type IN ('IMAGE','AUDIO')),
mime\_type TEXT NOT NULL,
size\_bytes INTEGER NOT NULL,
duration\_ms INTEGER,                             -- audio only; max 180000
width\_px INTEGER,                                -- image only
height\_px INTEGER,                               -- image only
encrypted\_original\_ref TEXT NOT NULL,            -- blob store key/path
encrypted\_thumbnail\_ref TEXT,                    -- image thumbnail key/path (256px target)
created\_at\_epoch\_ms INTEGER NOT NULL,
updated\_at\_epoch\_ms INTEGER NOT NULL,
FOREIGN KEY(note\_id) REFERENCES notes(id) ON DELETE CASCADE
);



CREATE INDEX idx\_attachments\_note\_id ON attachments(note\_id);
CREATE INDEX idx\_attachments\_type ON attachments(type);



\-- 6) Reminders (first-class)
CREATE TABLE reminders (
id TEXT PRIMARY KEY NOT NULL,                    -- UUIDv7
note\_id TEXT NOT NULL,
is\_enabled INTEGER NOT NULL DEFAULT 1,
recurrence TEXT NOT NULL CHECK(recurrence IN ('NONE','DAILY','WEEKLY','MONTHLY','YEARLY')),
anchor\_local\_datetime TEXT NOT NULL,             -- e.g. 2026-03-31T09:30:00
timezone\_id TEXT NOT NULL,                       -- e.g. Asia/Kuala\_Lumpur
next\_trigger\_at\_epoch\_ms INTEGER,
last\_triggered\_at\_epoch\_ms INTEGER,
created\_at\_epoch\_ms INTEGER NOT NULL,
updated\_at\_epoch\_ms INTEGER NOT NULL,
FOREIGN KEY(note\_id) REFERENCES notes(id) ON DELETE CASCADE
);



CREATE INDEX idx\_reminders\_next\_trigger ON reminders(next\_trigger\_at\_epoch\_ms);
CREATE INDEX idx\_reminders\_enabled ON reminders(is\_enabled);



\-- 7) Vault/security/settings metadata (single row)
CREATE TABLE vault\_meta (
id INTEGER PRIMARY KEY NOT NULL CHECK(id = 1),
kdf\_salt BLOB NOT NULL,
argon2\_memory\_kib INTEGER NOT NULL DEFAULT 65536,
argon2\_iterations INTEGER NOT NULL DEFAULT 3,
argon2\_parallelism INTEGER NOT NULL DEFAULT 1,
vault\_key\_blob BLOB NOT NULL,                    -- Encrypt(KEK, DEK || vault\_metadata)
local\_wrapped\_dek BLOB,                          -- Encrypt(DK, DEK)
quick\_unlock\_enabled INTEGER NOT NULL DEFAULT 0,
biometric\_fallback\_enabled INTEGER NOT NULL DEFAULT 0,
auto\_lock\_timeout\_sec INTEGER NOT NULL CHECK(auto\_lock\_timeout\_sec IN (0,30,60,120,300)),
secure\_screen\_enabled INTEGER NOT NULL DEFAULT 1,
created\_at\_epoch\_ms INTEGER NOT NULL,
updated\_at\_epoch\_ms INTEGER NOT NULL
);



\-- 8) Local plaintext index (local-only search model)
CREATE TABLE note\_index (
note\_id TEXT PRIMARY KEY NOT NULL,
display\_title TEXT NOT NULL,
search\_text TEXT NOT NULL,                       -- decrypted tokenizable text
updated\_at\_epoch\_ms INTEGER NOT NULL,
FOREIGN KEY(note\_id) REFERENCES notes(id) ON DELETE CASCADE
);



\-- 9) FTS virtual table for fast search
CREATE VIRTUAL TABLE note\_search\_fts USING fts5(
display\_title,
search\_text,
content='note\_index',
content\_rowid='rowid'
);



\-- 10) Triggers to keep FTS in sync with note\_index



\-- Keep FTS in sync when a row is inserted into note\_index
CREATE TRIGGER note\_index\_ai AFTER INSERT ON note\_index BEGIN
INSERT INTO note\_search\_fts(rowid, display\_title, search\_text)
VALUES (new.rowid, new.display\_title, new.search\_text);
END;



\-- Keep FTS in sync when a row is deleted from note\_index
CREATE TRIGGER note\_index\_ad AFTER DELETE ON note\_index BEGIN
INSERT INTO note\_search\_fts(note\_search\_fts, rowid, display\_title, search\_text)
VALUES ('delete', old.rowid, old.display\_title, old.search\_text);
END;



\-- Keep FTS in sync when a row is updated in note\_index
CREATE TRIGGER note\_index\_au AFTER UPDATE ON note\_index BEGIN
INSERT INTO note\_search\_fts(note\_search\_fts, rowid, display\_title, search\_text)
VALUES ('delete', old.rowid, old.display\_title, old.search\_text);

INSERT INTO note\_search\_fts(rowid, display\_title, search\_text)
VALUES (new.rowid, new.display\_title, new.search\_text);
END;

