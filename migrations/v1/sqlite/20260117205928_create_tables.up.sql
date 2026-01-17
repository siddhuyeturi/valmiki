-- Lets start with accounts table for storing accounts
CREATE TABLE IF NOT EXISTS accounts(
    id INTEGER PRIMARY KEY,
    -- timestamps
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    -- account flags
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    role TEXT NOT NULL,
    -- account creds
    username TEXT UNIQUE NOT NULL,
    pw_hash TEXT NOT NULL,
    -- account bio
    birth_date TEXT NOT NULL,
    email_id TEXT UNIQUE,
    phone_number TEXT UNIQUE,
    -- verification
    email_verified BOOLEAN,
    phone_number_verified BOOLEAN
);

CREATE TABLE IF NOT EXISTS profile(
    account_id INTEGER PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    first_name TEXT NOT NULL,
    middle_name TEXT,
    last_name TEXT NOT NULL,
    flat_house_number TEXT,
    address TEXT,
    pin_code TEXT,
    city TEXT,
    state TEXT,
    country TEXT,
    socials TEXT,
    bio TEXT,
    preferences TEXT
);

-- create table for holding sessions
CREATE TABLE IF NOT EXISTS session(
    id INTEGER PRIMARY KEY, -- session id or jti
    owner INTEGER NOT NULL REFERENCES accounts(id), -- session owner
    -- timestamps
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    -- state flags
    is_expired BOOLEAN NOT NULL,
    force_logout BOOLEAN NOT NULL,
    -- session info
    opeque_token_hash TEXT NOT NULL,
    ip TEXT NOT NULL,
    user_agent TEXT NOT NULL
);

-- A table for holding login attempts
CREATE TABLE IF NOT EXISTS login_attempts(
    id INTEGER PRIMARY KEY,
    created_at INTEGER NOT NULL,
    creds TEXT NOT NULL,
    is_success BOOLEAN NOT NULL,
    reason TEXT,
    account_id INTEGER REFERENCES accounts(id),
    ip TEXT,
    user_agent TEXT
);

CREATE TABLE IF NOT EXISTS visibility_group(
    id INTEGER PRIMARY KEY,
    group_name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    owner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS visibility_group_members(
    group_id INTEGER NOT NULL REFERENCES visibility_group(id),
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS collection(
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULl REFERENCES accounts(id) on delete cascade,
    is_public BOOLEAN NOT NULL DEFAULT 0,
    visibility_group INTEGER NOT NULL REFERENCES visibility_group(id),
    kind TEXT NOT NULL
);

-- A table for storing media uploads
CREATE TABLE IF NOT EXISTS document(
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    collection_id INTEGER NOT NULL REFERENCES collection(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_links TEXT NOT NULL DEFAULT '{}'
);
