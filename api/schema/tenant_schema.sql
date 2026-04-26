-- Garage360 Tenant Database Schema
-- This schema is run once when a new tenant is provisioned

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- =====================================================
-- CORE: Users, Locations, Settings
-- =====================================================

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('OWNER', 'ADMIN', 'MANAGER', 'ACCOUNT_MGR', 'MECHANIC', 'CASHIER', 'HR_OFFICER')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    address TEXT,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tenant_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    is_encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- CRM: Customers & Vehicles
-- =====================================================

CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_type TEXT NOT NULL CHECK (customer_type IN ('INDIVIDUAL', 'ORGANIZATION')),
    first_name TEXT,
    last_name TEXT,
    company_name TEXT,
    company_reg_no TEXT,
    vat_no TEXT,
    email TEXT,
    phone TEXT NOT NULL,
    phone_alternate TEXT,
    address TEXT,
    location_id UUID REFERENCES locations(id),
    notes TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_customers_phone ON customers(phone);
CREATE INDEX idx_customers_email ON customers(email);
CREATE INDEX idx_customers_name ON customers(LOWER(first_name), LOWER(last_name));
CREATE INDEX idx_customers_company ON customers(LOWER(company_name));
CREATE INDEX idx_customers_active ON customers(is_active);

-- Full-text search index for customers
CREATE INDEX idx_customers_fts ON customers USING gin(
    to_tsvector('english', 
        COALESCE(first_name, '') || ' ' || 
        COALESCE(last_name, '') || ' ' || 
        COALESCE(company_name, '') || ' ' || 
        COALESCE(email, '') || ' ' || 
        COALESCE(phone, '')
    )
);

CREATE TABLE IF NOT EXISTS vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
    registration_no TEXT NOT NULL,
    vin TEXT,
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    year INTEGER,
    engine_no TEXT,
    chassis_no TEXT,
    color TEXT,
    transmission TEXT CHECK (transmission IN ('MANUAL', 'AUTOMATIC', 'CVT')),
    fuel_type TEXT CHECK (fuel_type IN ('PETROL', 'DIESEL', 'ELECTRIC', 'HYBRID', 'CNG', 'LPG')),
    odometer_reading INTEGER,
    odometer_unit TEXT DEFAULT 'km',
    last_service_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_vehicles_registration ON vehicles(registration_no);
CREATE INDEX idx_vehicles_customer ON vehicles(customer_id);
CREATE INDEX idx_vehicles_vin ON vehicles(vin);
CREATE INDEX idx_vehicles_chassis ON vehicles(chassis_no);
CREATE INDEX idx_vehicles_active ON vehicles(is_active);

CREATE TABLE IF NOT EXISTS vehicle_photos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    file_deleted_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vehicle_photos_vehicle ON vehicle_photos(vehicle_id);
CREATE INDEX idx_vehicle_photos_active ON vehicle_photos(file_deleted_at);

-- Full-text search for vehicles (registration, vin, chassis)
CREATE INDEX idx_vehicles_fts ON vehicles USING gin(
    to_tsvector('english', 
        COALESCE(registration_no, '') || ' ' || 
        COALESCE(vin, '') || ' ' || 
        COALESCE(chassis_no, '')
    )
);

-- =====================================================
-- SERVICE BAYS
-- =====================================================

CREATE TABLE IF NOT EXISTS service_bays (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    capacity INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- JOB CARDS: Core
-- =====================================================

CREATE TYPE job_status AS ENUM (
    'INTAKE', 'AUDIT', 'QUOTE', 'APPROVAL', 'IN_SERVICE', 'QA', 'BILLING', 'COMPLETED', 'CANCELLED'
);

CREATE TABLE IF NOT EXISTS job_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_no SERIAL,
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
    bay_id UUID REFERENCES service_bays(id),
    status job_status NOT NULL DEFAULT 'INTAKE',
    complaint TEXT,
    diagnosis TEXT,
    odometer_in INTEGER,
    odometer_out INTEGER,
    estimated_completion TIMESTAMPTZ,
    mechanic_id UUID REFERENCES users(id),
    account_manager_id UUID REFERENCES users(id),
    qa_by UUID REFERENCES users(id),
    qa_cycles INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_cards_vehicle ON job_cards(vehicle_id);
CREATE INDEX idx_job_cards_customer ON job_cards(customer_id);
CREATE INDEX idx_job_cards_status ON job_cards(status);
CREATE INDEX idx_job_cards_bay ON job_cards(bay_id);
CREATE INDEX idx_job_cards_mechanic ON job_cards(mechanic_id);
CREATE INDEX idx_job_cards_active ON job_cards(is_active);
CREATE INDEX idx_job_cards_job_no ON job_cards(job_no);

CREATE TABLE IF NOT EXISTS job_card_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID NOT NULL REFERENCES job_cards(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL CHECK (item_type IN ('PART', 'LABOUR')),
    description TEXT NOT NULL,
    quantity NUMERIC(10,3) NOT NULL DEFAULT 1,
    unit_price NUMERIC(10,2) NOT NULL,
    discount_pct NUMERIC(5,2) NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_card_items_job ON job_card_items(job_card_id);

CREATE TABLE IF NOT EXISTS job_card_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID NOT NULL REFERENCES job_cards(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    description TEXT,
    metadata JSONB,
    performed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_job_card_activities_job ON job_card_activities(job_card_id);
CREATE INDEX idx_job_card_activities_created ON job_card_activities(created_at DESC);

CREATE TABLE IF NOT EXISTS job_card_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID NOT NULL REFERENCES job_cards(id) ON DELETE CASCADE,
    approved_by UUID REFERENCES users(id),
    approval_type TEXT NOT NULL,
    channel TEXT,
    notes TEXT,
    portal_token TEXT UNIQUE,
    portal_token_expires_at TIMESTAMPTZ,
    portal_token_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- INTAKE: Checklist, Photos, Signatures
-- =====================================================

CREATE TABLE IF NOT EXISTS intake_checklist_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    items JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS intake_checklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID UNIQUE REFERENCES job_cards(id) ON DELETE CASCADE,
    template_id UUID REFERENCES intake_checklist_templates(id),
    data JSONB NOT NULL,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS intake_photos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID NOT NULL REFERENCES job_cards(id) ON DELETE CASCADE,
    photo_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,
    file_deleted_at TIMESTAMPTZ,
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS customer_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID UNIQUE REFERENCES job_cards(id) ON DELETE CASCADE,
    signature_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_deleted_at TIMESTAMPTZ,
    signed_by TEXT,
    signed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- CHANGE REQUESTS
-- =====================================================

CREATE TABLE IF NOT EXISTS job_change_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID NOT NULL REFERENCES job_cards(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'APPROVED', 'DECLINED')),
    requested_by UUID REFERENCES users(id),
    approved_by UUID REFERENCES users(id),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS job_change_request_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    change_request_id UUID NOT NULL REFERENCES job_change_requests(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity NUMERIC(10,3) NOT NULL DEFAULT 1,
    unit_price NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- INVENTORY
-- =====================================================

CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT,
    unit TEXT NOT NULL DEFAULT 'pc',
    cost_price NUMERIC(10,2) NOT NULL DEFAULT 0,
    sell_price NUMERIC(10,2) NOT NULL DEFAULT 0,
    min_stock_level INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_inventory_sku ON inventory_items(sku);
CREATE INDEX idx_inventory_category ON inventory_items(category);
CREATE INDEX idx_inventory_active ON inventory_items(is_active);

CREATE TABLE IF NOT EXISTS stock_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inventory_item_id UUID NOT NULL REFERENCES inventory_items(id) ON DELETE CASCADE,
    alert_type TEXT NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS stock_adjustments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inventory_item_id UUID NOT NULL REFERENCES inventory_items(id) ON DELETE CASCADE,
    adjustment_type TEXT NOT NULL CHECK (adjustment_type IN ('ADD', 'REMOVE', 'SET', 'COUNT')),
    quantity NUMERIC(10,3) NOT NULL,
    previous_quantity NUMERIC(10,3),
    new_quantity NUMERIC(10,3),
    reason TEXT,
    performed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_stock_adjustments_item ON stock_adjustments(inventory_item_id);

-- =====================================================
-- PURCHASING
-- =====================================================

CREATE TABLE IF NOT EXISTS suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    contact_person TEXT,
    email TEXT,
    phone TEXT,
    address TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_suppliers_name ON suppliers(name);

CREATE TYPE po_status AS ENUM ('DRAFT', 'SENT', 'RECEIVED', 'APPROVED', 'REJECTED', 'GRN_COMPLETED', 'COMPLETED', 'CANCELLED');

CREATE TABLE IF NOT EXISTS purchase_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    po_no SERIAL,
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    status po_status NOT NULL DEFAULT 'DRAFT',
    expected_delivery TIMESTAMPTZ,
    subtotal NUMERIC(10,2) NOT NULL DEFAULT 0,
    discount_pct NUMERIC(5,2) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(10,2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(10,2) NOT NULL DEFAULT 0,
    notes TEXT,
    currency TEXT DEFAULT 'NPR',
    exchange_rate NUMERIC(10,6),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_purchase_orders_supplier ON purchase_orders(supplier_id);
CREATE INDEX idx_purchase_orders_status ON purchase_orders(status);

CREATE TABLE IF NOT EXISTS purchase_order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    purchase_order_id UUID NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
    inventory_item_id UUID REFERENCES inventory_items(id),
    description TEXT NOT NULL,
    quantity NUMERIC(10,3) NOT NULL,
    unit_price NUMERIC(10,2) NOT NULL,
    received_qty NUMERIC(10,3) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS po_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    purchase_order_id UUID NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
    approved_by UUID REFERENCES users(id),
    is_approved BOOLEAN NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS po_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    purchase_order_id UUID NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
    from_status TEXT,
    to_status TEXT NOT NULL,
    notes TEXT,
    changed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE grn_status AS ENUM ('PENDING', 'QA_PASSED', 'QA_FAILED', 'COMPLETED');

CREATE TABLE IF NOT EXISTS goods_receipt_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    grn_no SERIAL,
    purchase_order_id UUID REFERENCES purchase_orders(id),
    status grn_status NOT NULL DEFAULT 'PENDING',
    received_by UUID REFERENCES users(id),
    received_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS grn_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    grn_id UUID NOT NULL REFERENCES goods_receipt_notes(id) ON DELETE CASCADE,
    po_item_id UUID REFERENCES purchase_order_items(id),
    received_qty NUMERIC(10,3) NOT NULL,
    accepted_qty NUMERIC(10,3),
    rejected_qty NUMERIC(10,3),
    unit_cost NUMERIC(10,2),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS qa_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    grn_id UUID NOT NULL REFERENCES goods_receipt_notes(id) ON DELETE CASCADE,
    inspected_by UUID REFERENCES users(id),
    status TEXT NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- BILLING
-- =====================================================

CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_no SERIAL,
    job_card_id UUID REFERENCES job_cards(id),
    customer_id UUID NOT NULL REFERENCES customers(id),
    subtotal NUMERIC(10,2) NOT NULL DEFAULT 0,
    discount_pct NUMERIC(5,2) NOT NULL DEFAULT 0,
    discount_amount NUMERIC(10,2) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(10,2) NOT NULL DEFAULT 0,
    total_amount NUMERIC(10,2) NOT NULL DEFAULT 0,
    amount_paid NUMERIC(10,2) NOT NULL DEFAULT 0,
    payment_method TEXT,
    payment_ref TEXT,
    paid_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'PARTIAL', 'PAID', 'VOID')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoices_job ON invoices(job_card_id);
CREATE INDEX idx_invoices_customer ON invoices(customer_id);
CREATE INDEX idx_invoices_status ON invoices(status);

CREATE TABLE IF NOT EXISTS invoice_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity NUMERIC(10,3) NOT NULL DEFAULT 1,
    unit_price NUMERIC(10,2) NOT NULL,
    discount_pct NUMERIC(5,2) NOT NULL DEFAULT 0,
    line_total NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- DVI (Digital Vehicle Inspection)
-- =====================================================

CREATE TABLE IF NOT EXISTS dvi_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    sections JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dvi_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_card_id UUID NOT NULL REFERENCES job_cards(id) ON DELETE CASCADE,
    template_id UUID REFERENCES dvi_templates(id),
    data JSONB NOT NULL,
    submitted_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- ASSETS
-- =====================================================

CREATE TABLE IF NOT EXISTS assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_tag TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    category TEXT,
    location_id UUID REFERENCES locations(id),
    purchase_date DATE,
    purchase_cost NUMERIC(10,2),
   Useful_life_years INTEGER,
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_assets_tag ON assets(asset_tag);
CREATE INDEX idx_assets_category ON assets(category);

CREATE TABLE IF NOT EXISTS asset_inspection_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    items JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS asset_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    template_id UUID REFERENCES asset_inspection_templates(id),
    data JSONB NOT NULL,
    submitted_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS asset_defects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'REPORTED' CHECK (status IN ('REPORTED', 'IN_PROGRESS', 'RESOLVED')),
    reported_by UUID REFERENCES users(id),
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- HR
-- =====================================================

CREATE TYPE employment_type AS ENUM ('FULL_TIME', 'PART_TIME', 'CONTRACT', 'INTERN');

CREATE TABLE IF NOT EXISTS employees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_no TEXT NOT NULL UNIQUE,
    user_id UUID REFERENCES users(id),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT,
    phone TEXT NOT NULL,
    employment_type employment_type NOT NULL,
    department TEXT,
    designation TEXT,
    join_date DATE,
    salary NUMERIC(10,2),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_employees_no ON employees(employee_no);
CREATE INDEX idx_employees_active ON employees(is_active);

CREATE TABLE IF NOT EXISTS payroll_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'CLOSED')),
    processed_by UUID REFERENCES users(id),
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payroll_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_id UUID NOT NULL REFERENCES payroll_periods(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id),
    basic_salary NUMERIC(10,2) NOT NULL,
    allowances JSONB,
    deductions JSONB,
    gross_salary NUMERIC(10,2) NOT NULL,
    net_salary NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE leave_type AS ENUM ('ANNUAL', 'SICK', 'UNPAID', 'OTHER');

CREATE TABLE IF NOT EXISTS leave_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    leave_type leave_type NOT NULL,
    days_per_year INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS leave_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id),
    leave_type_id UUID NOT NULL REFERENCES leave_types(id),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    days_count NUMERIC(5,1) NOT NULL,
    reason TEXT,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED')),
    approved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id),
    date DATE NOT NULL,
    clock_in TIMESTAMPTZ,
    clock_out TIMESTAMPTZ,
    hours_worked NUMERIC(5,2),
    status TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attendance_employee ON attendance_records(employee_id, date);

-- =====================================================
-- REPORTS
-- =====================================================

CREATE TABLE IF NOT EXISTS saved_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    report_type TEXT NOT NULL,
    config JSONB NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- AUDIT
-- =====================================================

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID,
    user_id UUID,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    metadata JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_action ON audit_logs(action);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);

-- =====================================================
-- SEED DATA
-- =====================================================

-- Default tenant settings
INSERT INTO tenant_settings (key, value) VALUES 
    ('schema_version', '1.0.0'),
    ('timezone', 'Asia/Kathmandu'),
    ('currency_symbol', 'Rs.'),
    ('currency_code', 'NPR'),
    ('date_format', 'YYYY-MM-DD')
ON CONFLICT (key) DO NOTHING;

-- Default service bays
INSERT INTO service_bays (code, name, capacity) VALUES 
    ('BAY-1', 'Service Bay 1', 1),
    ('BAY-2', 'Service Bay 2', 1),
    ('BAY-3', 'Service Bay 3', 1),
    ('BAY-4', 'Service Bay 4', 1),
    ('BAY-5', 'Service Bay 5', 1)
ON CONFLICT (code) DO NOTHING;

-- Default intake checklist template
INSERT INTO intake_checklist_templates (name, items) VALUES 
    ('Standard Intake', 
     '[
        {"key": "keys", "label": "Keys received", "required": true},
        {"key": "tyres", "label": "Tyres condition", "required": true},
        {"key": "engine", "label": "Engine noise", "required": true},
        {"key": "odometer", "label": "Odometer reading", "required": true},
        {"key": "lights", "label": "Lights functional", "required": true},
        {"key": "belongings", "label": "Personal belongings noted", "required": false},
        {"key": "damage", "label": "Existing damage noted", "required": false}
     ]')
ON CONFLICT (name) DO NOTHING;

-- Default DVI template
INSERT INTO dvi_templates (name, sections) VALUES 
    ('Standard DVI',
     '[
        {"section": "Exterior", "items": [
            {"key": "body Paint", "label": "Body Paint", "type": "condition"},
            {"key": "tyres", "label": "Tyres", "type": "condition"},
            {"key": "lights", "label": "Lights", "type": "condition"},
            {"key": "mirrors", "label": "Mirrors", "type": "condition"},
            {"key": "windshield", "label": "Windshield", "type": "condition"}
        ]},
        {"section": "Under Bonnet", "items": [
            {"key": "engine", "label": "Engine Oil", "type": "level"},
            {"key": "coolant", "label": "Coolant", "type": "level"},
            {"key": "brake_fluid", "label": "Brake Fluid", "type": "level"},
            {"key": "battery", "label": "Battery", "type": "condition"}
        ]},
{"section": "Interior", "items": [
            {"key": "dashboard", "label": "Dashboard Lights", "type": "check"},
            {"key": "ac", "label": "A/C Function", "type": "check"},
            {"key": "brakes", "label": "Brakes", "type": "test"}
        ]}
    ]')
ON CONFLICT (name) DO NOTHING;

-- =====================================================
-- SEED DATA: Sample customers
-- =====================================================
INSERT INTO customers (customer_type, first_name, last_name, email, phone, address)
VALUES 
    ('INDIVIDUAL', 'Ram', 'Poudel', 'ram.poudel@email.com', '+977 9841234567', 'Teku, Kathmandu'),
    ('INDIVIDUAL', 'Shyam', 'Khatri', 'shyam.khatri@email.com', '+977 9842345678', 'Baneshwor, Kathmandu'),
    ('INDIVIDUAL', 'Hari', 'Sharma', 'hari.sharma@email.com', '+977 9843456789', 'Kalanki, Kathmandu'),
    ('INDIVIDUAL', 'Geeta', 'Thapa', 'geeta.thapa@email.com', '+977 9844567890', 'Koteshwor, Kathmandu'),
    ('INDIVIDUAL', 'Maya', 'Joshi', 'maya.joshi@email.com', '+977 9845678901', 'Satungal, Kathmandu'),
    ('ORGANIZATION', NULL, NULL, 'info@npc.com.np', '+977 011234567', 'Tripureshwor, Kathmandu'),
    ('ORGANIZATION', NULL, NULL, 'finance@spml.com.np', '+977 011345678', 'Pushpark, Kathmandu')
ON CONFLICT DO NOTHING;

-- =====================================================
-- SEED DATA: Sample vehicles  
-- =====================================================
INSERT INTO vehicles (customer_id, registration_no, vin, make, model, year, color, fuel_type, odometer_reading)
VALUES 
    ((SELECT id FROM customers WHERE phone = '+977 9841234567'), 'BA 1 PA 1111', 'VIN1111111111111111', 'Toyota', 'Camry', 2023, 'White', 'PETROL', 15000),
    ((SELECT id FROM customers WHERE phone = '+977 9842345678'), 'BA 1 PA 2222', 'VIN2222222222222222', 'Honda', 'Civic', 2022, 'Silver', 'PETROL', 25000),
    ((SELECT id FROM customers WHERE phone = '+977 9843456789'), 'BA 1 PA 3333', 'VIN3333333333333333', 'Hyundai', 'Creta', 2023, 'Black', 'PETROL', 12000),
    ((SELECT id FROM customers WHERE phone = '+977 9844567890'), 'BA 1 PA 4444', 'VIN4444444444444444', 'Tata', 'Nexon', 2021, 'Blue', 'DIESEL', 35000),
    ((SELECT id FROM customers WHERE phone = '+977 9845678901'), 'BA 1 PA 5555', 'VIN5555555555555555', 'Mahindra', 'Scorpio', 2022, 'Grey', 'DIESEL', 42000),
    ((SELECT id FROM customers WHERE phone = '+977 011234567'), 'BA 1 P 1234', 'VIN6666666666666666', 'Toyota', 'Hiace', 2022, 'White', 'DIESEL', 35000),
    ((SELECT id FROM customers WHERE phone = '+977 011345678'), 'BA 1 P 5678', 'VIN7777777777777777', 'Toyota', 'Hiace', 2023, 'White', 'DIESEL', 15000)
ON CONFLICT (registration_no) DO NOTHING;

-- =====================================================
-- SEED DATA: Sample users for tenant
-- =====================================================
INSERT INTO users (email, password_hash, name, role)
VALUES 
    ('mechanic@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$aKGSWB7Y5TYfM4ODwUyoxw$KdrDzSEUicsPhQvpP/IchopvAvTuwiOlOnZ8slRfEEw', 'Ram Kumar', 'MECHANIC'),
    ('manager@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$aKGSWB7Y5TYfM4ODwUyoxw$KdrDzSEUicsPhQvpP/IchopvAvTuwiOlOnZ8slRfEEw', 'Shyam Manager', 'MANAGER'),
    ('account@demo.com', '$argon2id$v=19$m=19456,t=2,p=1$aKGSWB7Y5TYfM4ODwUyoxw$KdrDzSEUicsPhQvpP/IchopvAvTuwiOlOnZ8slRfEEw', 'Hari Account', 'ACCOUNT_MGR')
ON CONFLICT (email) DO NOTHING;

-- =====================================================
-- SEED DATA: Sample inventory items
-- =====================================================
INSERT INTO inventory_items (sku, name, description, category, unit, cost_price, sell_price, min_stock_level)
VALUES 
    ('Oil-5W30', 'Engine Oil 5W-30', 'Synthetic engine oil 5W-30', 'Oils', 'litre', 450.00, 550.00, 20),
    ('Oil-10W40', 'Engine Oil 10W-40', 'Mineral engine oil 10W-40', 'Oils', 'litre', 350.00, 420.00, 20),
    ('Filter-Oil', 'Oil Filter', 'Standard oil filter', 'Filters', 'pc', 180.00, 250.00, 10),
    ('Filter-Air', 'Air Filter', 'Standard air filter', 'Filters', 'pc', 220.00, 320.00, 10),
    ('Filter-Cabin', 'Cabin Air Filter', 'Cabin air filter', 'Filters', 'pc', 350.00, 450.00, 5),
    ('Brake-Pad-F', 'Brake Pads Front', 'Front brake pads', 'Brakes', 'set', 1200.00, 1800.00, 5),
    ('Brake-Pad-R', 'Brake Pads Rear', 'Rear brake pads', 'Brakes', 'set', 900.00, 1400.00, 5),
    ('Spark-Plug', 'Spark Plug', 'Iridium spark plug', 'Ignition', 'pc', 180.00, 280.00, 20),
    ('Battery-12V', 'Car Battery 12V', '12V 70AH car battery', 'Electrical', 'pc', 4500.00, 5500.00, 3),
    ('Tyre-185-65', 'Tyre 185/65 R15', 'Standard passenger tyre 185/65 R15', 'Tyres', 'pc', 4500.00, 5500.00, 8)
ON CONFLICT (sku) DO NOTHING;

-- =====================================================
-- SEED DATA: Sample job cards (service jobs)
-- =====================================================
INSERT INTO job_cards (vehicle_id, customer_id, bay_id, status, complaint, odometer_in, estimated_completion)
VALUES 
    ((SELECT id FROM vehicles WHERE registration_no = 'BA 1 PA 1111'), 
     (SELECT customer_id FROM vehicles WHERE registration_no = 'BA 1 PA 1111'), 
     (SELECT id FROM service_bays WHERE code = 'BAY-1'),
     'INTAKE', 'Oil change and general service', 15000, NOW() + INTERVAL '3 days'),
    ((SELECT id FROM vehicles WHERE registration_no = 'BA 1 PA 2222'), 
     (SELECT customer_id FROM vehicles WHERE registration_no = 'BA 1 PA 2222'), 
     (SELECT id FROM service_bays WHERE code = 'BAY-2'),
     'QUOTE', 'Brake noise while braking', 25000, NOW() + INTERVAL '2 days'),
    ((SELECT id FROM vehicles WHERE registration_no = 'BA 1 PA 3333'), 
     (SELECT customer_id FROM vehicles WHERE registration_no = 'BA 1 PA 3333'), 
     (SELECT id FROM service_bays WHERE code = 'BAY-3'),
     'IN_SERVICE', 'Engine check light on - diagnostic required', 12000, NOW() + INTERVAL '1 day'),
    ((SELECT id FROM vehicles WHERE registration_no = 'BA 1 PA 4444'), 
     (SELECT customer_id FROM vehicles WHERE registration_no = 'BA 1 PA 4444'), 
     (SELECT id FROM service_bays WHERE code = 'BAY-1'),
     'QA', 'AC not cooling properly', 35000, NOW() + INTERVAL '1 day'),
    ((SELECT id FROM vehicles WHERE registration_no = 'BA 1 PA 5555'), 
     (SELECT customer_id FROM vehicles WHERE registration_no = 'BA 1 PA 5555'), 
     (SELECT id FROM service_bays WHERE code = 'BAY-2'),
     'COMPLETED', 'Scheduled service completed', 42000, NOW() - INTERVAL '1 day')
ON CONFLICT DO NOTHING;

-- =====================================================
-- SEED DATA: Sample job card items
-- =====================================================
INSERT INTO job_card_items (job_card_id, item_type, description, quantity, unit_price, discount_pct)
SELECT 
    jc.id, 
    'PART', 
    item_name, 
    item_qty, 
    item_price, 
    item_discount
FROM job_cards jc
CROSS JOIN LATERAL (
    VALUES 
        (1, 'Oil-5W30', 4, 550.00, 0),
        (2, 'Filter-Oil', 1, 250.00, 0),
        (3, 'Filter-Air', 1, 320.00, 0)
) AS t(ord, item_name, item_qty, item_price, item_discount)
WHERE jc.status = 'INTAKE'
ORDER BY jc.id, ord
ON CONFLICT DO NOTHING;

INSERT INTO job_card_items (job_card_id, item_type, description, quantity, unit_price, discount_pct)
SELECT 
    jc.id, 
    'PART', 
    item_name, 
    item_qty, 
    item_price, 
    item_discount
FROM job_cards jc
CROSS JOIN LATERAL (
    VALUES 
        (1, 'Brake-Pad-F', 1, 1800.00, 10),
        (2, 'Brake-Pad-R', 1, 1400.00, 10)
) AS t(ord, item_name, item_qty, item_price, item_discount)
WHERE jc.complaint LIKE '%Brake%'
ORDER BY jc.id, ord
ON CONFLICT DO NOTHING;

-- =====================================================
-- SEED DATA: Job card activities
-- =====================================================
INSERT INTO job_card_activities (job_card_id, action, description)
SELECT id, 'CREATED', 'Job card created' FROM job_cards WHERE status IN ('INTAKE', 'QUOTE', 'IN_SERVICE')
ON CONFLICT DO NOTHING;

INSERT INTO job_card_activities (job_card_id, action, description)
SELECT id, 'STATUS_CHANGE', 'Status changed to IN_SERVICE' FROM job_cards WHERE status = 'IN_SERVICE'
ON CONFLICT DO NOTHING;
