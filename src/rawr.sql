-- SPDX-License-Identifier: Apache-2.0

-- Table: Upstream Codebase
-- Describes a codebase of interest
-- CREATE TABLE IF NOT EXISTS upstream
-- (
--     id            INT  NOT NULL PRIMARY KEY,
--     name          TEXT NOT NULL UNIQUE,
--     relative_path TEXT NOT NULL,
--     notes         TEXT
-- ) STRICT;

-- Table: Upstream Items of Interest
-- Stores all items of interest at all revisions.
-- No primary key due to high chance of duplicates.
CREATE TABLE IF NOT EXISTS upstream
(
    -- ID of upstream codebase.
    -- Use ID later.
    upstream      TEXT    NOT NULL,

    -- Treeish of revision, expecting SHA1, may be anything.
    revision      TEXT    NOT NULL,

    -- Relative path to containing file.
    path          TEXT    NOT NULL,

    -- Name of the tree sitter grammar
    lang          TEXT    NOT NULL,

    -- Kind of matched item, extracted from matcher
    kind          TEXT    NOT NULL,

    -- Name of matched item
    identifier    TEXT    NOT NULL,

    -- Hash of matched bytes, stored as uppercase hex without leading 0x.
    -- Switch to BLOB for efficiency. Consider first 64 bits of SHA 512?
    hash          BLOB    NOT NULL,

    -- Hash of matched bytes, after attempting to convert to valid UTF-8
    -- and stripping Unicode whitespace. This should aid in matching after
    -- reformatting.
    hash_stripped BLOB,

    -- Range information
    start_byte    INTEGER NOT NULL,
    end_byte      INTEGER NOT NULL,

    start_line    INTEGER NOT NULL,
    start_column  INTEGER NOT NULL,
    end_line      INTEGER NOT NULL,
    end_column    INTEGER NOT NULL,

    -- Optional notes regarding item.
    notes         TEXT,


    -- Physical ordering and uniqueness
    CONSTRAINT PK_upstream PRIMARY KEY (upstream, revision, path, lang, kind, identifier, hash, start_byte)

-- Skip FK for now, to simplify build.
--     CONSTRAINT FK_upstream_codebase FOREIGN KEY (codebase)
--         REFERENCES codebase (name) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

-- Index: Item History
-- Track an item across many revisions
CREATE INDEX IF NOT EXISTS IX_upstream_item_history ON upstream (upstream, lang, kind, identifier, path, revision);

-- Index: Lookup by Hash
-- Get item info based on hash and other attributes
CREATE INDEX IF NOT EXISTS IX_upstream_hash ON upstream (hash, kind, identifier, lang, revision, upstream);
