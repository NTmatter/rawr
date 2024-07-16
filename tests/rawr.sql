-- SPDX-License-Identifier: Apache-2.0

-- Table: Upstream Codebase
--
CREATE TABLE IF NOT EXISTS codebase
(
    name          TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    notes         TEXT,
    CONSTRAINT PK_codebase PRIMARY KEY (name)
) STRICT;

-- Table: Upstream Items of Interest
-- Stores all items of interest at all revisions.
-- No primary key due to high chance of duplicates.
CREATE TABLE IF NOT EXISTS upstream
(
    -- Friendly name of upstream codebase.
    codebase       TEXT    NOT NULL,

    -- Treeish of revision, expecting SHA1, may be anything.
    revision       TEXT    NOT NULL,

    -- Relative path to containing file.
    path           TEXT    NOT NULL,

    -- Byte Offset of item within file.
    start_byte     INTEGER NOT NULL,

    -- Length of matched item
    length         INTEGER NOT NULL,

    -- Name of matched item
    identifier     TEXT    NOT NULL,

    -- Kind of matched item, extracted from matcher
    kind           TEXT    NOT NULL,

    -- Hash of matched bytes, selected algorithm. Likely SHA512.
    hash_algorithm TEXT    NOT NULL,

    -- Optional salt for hash. This will be problematic for lookups.
    -- Store as a u64.
    salt           INT,

    -- Hash of matched bytes, stored as uppercase hex without leading 0x.
    -- Switch to BLOB for efficiency. Consider first 64 bits of SHA 512?
    hash           TEXT    NOT NULL,

    -- Optional notes regarding item.
    notes          TEXT

-- Skip FK for now, to simplify build.
--     CONSTRAINT FK_upstream_codebase FOREIGN KEY (codebase)
--         REFERENCES codebase (name) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

-- Index: Upstream primary query
-- Expect intensive lookups across an item's history.
CREATE INDEX IF NOT EXISTS IX_upstream ON upstream (codebase, path, identifier, kind, revision);

-- Index: Hash Lookup
-- Expect for lookups by hash, looking for duplicates by type, varying identifier,
-- and across codebases.
CREATE INDEX IF NOT EXISTS IX_upstream_hash ON upstream (hash, kind, identifier, codebase)
    WHERE hash IS NOT NULL;
