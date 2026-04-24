-- Secure School ERP - Database Schema (Supabase)

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    phone TEXT,
    avatar_url TEXT,
    totp_secret TEXT,
    totp_enabled BOOLEAN DEFAULT false,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'inactive')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Permissions table
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- User-Role mapping
CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    school_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, role_id, school_id)
);

-- User-specific scopes
CREATE TABLE user_scopes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    scope_type TEXT NOT NULL,
    scope_value TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sessions table (for refresh token rotation)
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL,
    device_info TEXT,
    ip_address TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit log table (append-only)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    entity_type TEXT,
    entity_id UUID,
    old_value JSONB,
    new_value JSONB,
    ip_address TEXT,
    user_agent TEXT,
    trace_id UUID DEFAULT uuid_generate_v4(),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_user_roles_user ON user_roles(user_id);
CREATE INDEX idx_user_roles_role ON user_roles(role_id);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);
CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_refresh ON sessions(refresh_token_hash);

-- Insert default roles
INSERT INTO roles (name, description) VALUES
    ('admin', 'Full system administrator'),
    ('yayasan', 'Foundation management'),
    ('finance', 'Finance department'),
    ('hr', 'Human resources'),
    ('teacher', 'Teacher'),
    ('student', 'Student'),
    ('parent', 'Parent/Guardian'),
    ('donor', 'Donor'),
    ('public', 'Public access') ON CONFLICT (name) DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (name, description) VALUES
    ('auth:manage', 'Manage authentication'),
    ('user:read', 'Read users'),
    ('user:write', 'Create/update users'),
    ('user:delete', 'Delete users'),
    ('role:manage', 'Manage roles'),
    ('payment:read', 'Read payments'),
    ('payment:write', 'Create payments'),
    ('audit:read', 'Read audit logs'),
    ('report:generate', 'Generate reports'),
    ('ai:query', 'Query AI assistant'),
    ('file:upload', 'Upload files'),
    ('file:read', 'Read files'),
    ('student:manage', 'Manage students'),
    ('attendance:manage', 'Manage attendance') ON CONFLICT (name) DO NOTHING;