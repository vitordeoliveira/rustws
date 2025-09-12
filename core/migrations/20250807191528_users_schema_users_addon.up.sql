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

-- Insert default admin user
INSERT INTO users.users (id, email, hashed_password, role, is_owner)
VALUES (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'admin@admin.com',
    '$argon2id$v=19$m=19456,t=2,p=1$H1L6B6aqx61Dq3uvCLE38A$TAb7g38eLzUNFOloY8APx3nQk18rBW6tQCX2/od2R3E',
    'admin',
    true
);
