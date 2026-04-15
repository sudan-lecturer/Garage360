-- Garage360 Control Database Schema
-- This schema is initialized when the control-db container starts

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Super Admin Users
CREATE TABLE IF NOT EXISTS super_admin_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tenants Registry
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    database_host TEXT NOT NULL DEFAULT 'localhost',
    database_port INTEGER NOT NULL DEFAULT 5432,
    database_name TEXT NOT NULL,
    database_username TEXT NOT NULL DEFAULT 'postgres',
    database_password TEXT NOT NULL DEFAULT 'postgres',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tenants_slug ON tenants(slug);
CREATE INDEX idx_tenants_is_active ON tenants(is_active);

-- Feature Flags (Global Defaults)
CREATE TABLE IF NOT EXISTS feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL UNIQUE,
    description TEXT,
    default_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tenant Feature Flag Overrides
CREATE TABLE IF NOT EXISTS tenant_feature_flag_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    feature_flag_id UUID NOT NULL REFERENCES feature_flags(id) ON DELETE CASCADE,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, feature_flag_id)
);

CREATE INDEX idx_tenant_overrides_tenant ON tenant_feature_flag_overrides(tenant_id);

-- Control Audit Logs
CREATE TABLE IF NOT EXISTS control_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    performed_by TEXT,
    performed_by_role TEXT,
    metadata JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_control_audit_entity ON control_audit_logs(entity_type, entity_id);
CREATE INDEX idx_control_audit_action ON control_audit_logs(action);
CREATE INDEX idx_control_audit_created ON control_audit_logs(created_at DESC);

-- Seed default feature flags
INSERT INTO feature_flags (key, description, default_enabled) VALUES
    ('module.dvi', 'Digital Vehicle Inspection', true),
    ('module.purchases', 'Purchase Orders + GRN + QA', true),
    ('module.reports', 'Report builder', true),
    ('module.hr', 'HR records + payroll', false),
    ('module.assets', 'Asset management', false),
    ('module.customer_portal', 'Customer self-service portal', false),
    ('module.loyalty', 'Loyalty points', false),
    ('jobs.intake_inspection', 'Intake checklist', true),
    ('jobs.intake_signature', 'Customer digital signature', true),
    ('jobs.bay_management', 'Service bay tracking', true),
    ('jobs.approval_workflow', 'Quote approval step', true),
    ('jobs.mid_service_approval', 'Change request approvals', true),
    ('jobs.dvi_required', 'Block billing until QA complete', false),
    ('jobs.notification_email', 'Email customer notifications', true),
    ('jobs.notification_sms', 'SMS customer notifications', false),
    ('inventory.low_stock_alerts', 'Low stock push notifications', true),
    ('purchases.approval_required', 'PO approval before sending', true),
    ('purchases.qa_required', 'QA before stock-in', true),
    ('billing.vat', 'VAT/tax line on invoices', false),
    ('billing.multi_currency', 'Multi-currency', false),
    ('hr.payroll', 'Payroll (requires module.hr)', false),
    ('hr.leave_management', 'Leave management', false),
    ('hr.attendance', 'Clock-in/out', false),
    ('assets.daily_inspection', 'Daily inspection checklist', true),
    ('export.excel', 'Excel export on all lists', true),
    ('import.excel', 'Excel import for core entities', true)
ON CONFLICT (key) DO NOTHING;

-- Create default super admin (password: admin123 - CHANGE IN PRODUCTION)
INSERT INTO super_admin_users (email, password_hash, name)
VALUES (
    'admin@garage360.io',
    '$argon2id$v=19$m=19456,t=2,p=1$QzFHN1JuV3R5ZVN0cXFMTQ$6pYmTsI4K7nLwXhR5bK0K7mQ4kJ3mN5oP6qR7sT8uV',
    'System Admin'
)
ON CONFLICT (email) DO NOTHING;
