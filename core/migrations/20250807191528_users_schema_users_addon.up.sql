-- Add up migration script here

CREATE SCHEMA IF NOT EXISTS users;

CREATE TABLE users.users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL,
    hashed_password TEXT NOT NULL,
    role TEXT DEFAULT 'user',
    is_owner BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT now(),
    UNIQUE (email)
);

-- CREATE TABLE users.sessions (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     user_id UUID NOT NULL REFERENCES users.users(id) ON DELETE CASCADE,
--     session_token TEXT UNIQUE NOT NULL,
--     expires_at TIMESTAMP NOT NULL,
--     remember_me BOOLEAN DEFAULT FALSE,
--     created_at TIMESTAMP DEFAULT now()
-- );

-- CREATE INDEX idx_sessions_token ON users.sessions(session_token);
-- CREATE INDEX idx_sessions_user_id ON users.sessions(user_id);
-- CREATE INDEX idx_sessions_expires_at ON users.sessions(expires_at);
