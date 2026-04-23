# Garage360 — Phase Development Plan

## Overview

This document tracks the phase-wise development progress. It complements MASTER-PLAN.md with detailed UI build steps based on the Step-by-Step UI Build Plan.

### Stack
- React 19 + TypeScript + Vite 6 + Tailwind CSS v4
- TanStack Query v5 + Zustand + shadcn/ui + Radix UI + Lucide React
- Design System: Dark industrial theme — Salt (#FFFFFF), Pepper (#2B2B2B), Amber (#F59E0B) accent

---

## Phase 0 — Infrastructure & Setup
**Status:** ✅ Complete

- [x] Set up repository structure (`api/`, `web/`, `docker/`)
- [x] Write `docker-compose.yml` and all Dockerfiles
- [x] Bare Axum app with health check endpoint
- [x] Bare Vite + React + TypeScript + Tailwind setup
- [x] Control DB schema + tenant provisioning endpoint
- [x] JWT auth middleware skeleton
- [x] **Color migration**: Salt & Pepper palette applied to UI

**Artifacts:**
- `web/src/styles/globals.css` - Salt & Pepper color tokens
- `web/src/layouts/MainLayout.tsx` - Color updates
- `web/index.html` - theme-color meta tag

---

## Phase 1 — Auth & Foundation
**Status:** ✅ Complete

- [x] Auth module: login, refresh, logout (Rust)
- [x] Tenant router middleware (Rust)
- [x] Feature flag resolution middleware (Rust)
- [x] Login screen + auth state (React)
- [x] App shell with sidebar
- [x] Login → Dashboard redirect

**Credentials:**
- Email: `admin@garage360.io`
- Password: `password`

**Known Issues:**
- `api/src/modules/auth/routes.rs` has DEV BYPASS - remove before production

---

## Phase 2 — Shared Component Library & API Hooks
**Status:** 🛠️ Next

### Step 1 — Shared Component Library (`web/src/components/shared/`)
- [ ] DataTable — TanStack Table wrapper: pagination, sorting, filtering, multi-select, row actions, responsive
- [ ] StatusBadge — Semantic color badges (INTAKE=amber, AUDIT=blue, QUOTE=yellow, APPROVAL=purple, IN_SERVICE=cyan, QA=indigo, BILLING=green, COMPLETED=success, CANCELLED=destructive)
- [ ] EmptyState — Icon + title + description + CTA button
- [ ] LoadingSpinner / Skeleton — Full-page and inline variants
- [ ] PageHeader — Title + description + action buttons + breadcrumb
- [ ] FormField — React Hook Form + Zod + Label + Input/Select/Textarea combo
- [ ] ConfirmDialog — Radix Dialog for delete/status-change confirmations
- [ ] Avatar — Customer/employee initials avatar with fallback
- [ ] SearchInput — Debounced search with useState + useEffect

### Step 2 — Extend UI Primitives (`web/src/components/ui/`)
- [ ] select.tsx — Radix Select
- [ ] textarea.tsx — Radix Textarea
- [ ] dialog.tsx — Radix Dialog (for modals)
- [ ] dropdown-menu.tsx — Radix Dropdown Menu (row actions)
- [ ] card.tsx — Surface card with variants
- [ ] badge.tsx — Status badges
- [ ] tabs.tsx — Radix Tabs (page sub-sections)
- [ ] table.tsx — TanStack Table base + header/body/cell primitives
- [ ] pagination.tsx — Table pagination controls
- [ ] checkbox.tsx — Radix Checkbox (bulk select)
- [ ] popover.tsx — Radix Popover (filters)
- [ ] separator.tsx — Radix Separator
- [ ] tooltip.tsx — Radix Tooltip
- [ ] switch.tsx — Radix Switch (feature flags)
- [ ] scroll-area.tsx — Radix Scroll Area (long lists)

### Step 3 — API Hook Layer (`web/src/api/hooks/`)
- [ ] useCustomers.ts — useQuery for list, useMutation for CRUD
- [ ] useVehicles.ts — same pattern
- [ ] useJobs.ts — list, detail, transition mutation, activity feed
- [ ] useInventory.ts
- [ ] usePurchases.ts
- [ ] useBilling.ts
- [ ] useAssets.ts
- [ ] useHR.ts
- [ ] useReports.ts
- [ ] useSettings.ts

Each hook file: useQuery + useMutation + queryClient.invalidateQueries on success

---

## Phase 3 — Dashboard & Core Navigation
**Status:** 📋 Planned

### Step 4 — Dashboard Page (`modules/dashboard/`)
- [ ] KPI cards row: Open Jobs, Stock Alerts, Bays Occupied, Goods In Transit, Recent Activity
- [ ] Recharts for mini sparklines in KPI cards
- [ ] Bay Board widget: grid of bays with status (FREE/OCCUPIED/RESERVED) + job number
- [ ] Recent activity feed from job_card_activities
- [ ] Quick action buttons: New Job, Add Customer, Add Inventory
- [ ] Responsive: stack cards on mobile 375px

### Step 5 — Bay Board (`modules/dashboard/BayBoard.tsx`)
- [ ] Visual grid of service bays with real-time status
- [ ] Click bay → opens job detail or assign modal
- [ ] Color-coded: green=FREE, amber=OCCUPIED, gray=RESERVED, red=MAINTENANCE

---

## Phase 4 — CRM (Customers & Vehicles)
**Status:** 📋 Planned

### Step 6 — Customer List (`modules/customers/pages/CustomerList.tsx`)
- [ ] TanStack Table: Name, Type (Individual/Org), Phone, Email, Vehicle Count, Total Spent, Actions
- [ ] Search bar (debounced), filter by type
- [ ] Export button (feature-flag aware)
- [ ] Responsive: horizontal scroll on mobile

### Step 7 — Customer Create/Edit (`modules/customers/pages/CustomerForm.tsx`)
- [ ] React Hook Form + Zod schema
- [ ] Toggle: Individual (name, phone, email) vs Organisation (org name, contact person, phone, email)
- [ ] Address fields (optional)
- [ ] Submit mutation with loading state

### Step 8 — Customer Detail (`modules/customers/pages/CustomerDetail.tsx`)
- [ ] Tabs: Profile | Vehicles | Job History | Invoices
- [ ] Profile: avatar + info grid
- [ ] Vehicles: mini table linked to vehicle module
- [ ] Job History: recent jobs with status badges
- [ ] Invoices: recent invoices with amounts

### Step 9 — Vehicle List (`modules/vehicles/pages/VehicleList.tsx`)
- [ ] Table: License Plate, Make, Model, Year, Customer, Last Service, Actions
- [ ] Search by license plate (primary search use case)
- [ ] Filter by make/model

### Step 10 — Vehicle Detail (`modules/vehicles/pages/VehicleDetail.tsx`)
- [ ] Profile info + job history timeline

---

## Phase 5 — Job Card System
**Status:** 📋 Planned

### Step 11 — Job List (`modules/jobs/pages/JobList.tsx`)
- [ ] Two views: Kanban board (by status column) + Table view toggle
- [ ] Kanban: draggable cards between columns
- [ ] Table: Job #, Customer, Vehicle, Status, Mechanic, Bay, Created, Actions
- [ ] Filters: status, mechanic, bay, date range
- [ ] Search: by job number or customer name

### Step 12 — Job Detail (`modules/jobs/pages/JobDetail.tsx`)
- [ ] Tabs: Overview | Items | Timeline | Intake | QA | Invoice
- [ ] Overview tab: job info, customer/vehicle links, status badge, assigned mechanic/bay, complaint, diagnosis
- [ ] Timeline tab: job_card_activities feed with timestamps, user, action
- [ ] Items tab: parts + labour line items with edit capability
- [ ] Intake tab: checklist responses, photos (WebP thumbnails), signature
- [ ] QA tab: intake reference panel + QA checklist + pass/fail result
- [ ] Invoice tab: link to invoice or "Generate Invoice" button

### Step 13 — Intake Flow (`modules/jobs/pages/IntakeFlow.tsx`)
- [ ] Multi-step wizard: Step 1 Checklist → Step 2 Photos → Step 3 Signature → Step 4 Confirm
- [ ] Step 1: Dynamic checklist from intake_checklist_templates, checkboxes per item
- [ ] Step 2: react-webcam capture + upload, display thumbnails, delete option (max 10 photos)
- [ ] Step 3: react-signature-canvas for customer signature
- [ ] Step 4: Review all + confirm → creates job card with INTAKE status
- [ ] Mobile-optimized: large touch targets (min 44px), sticky bottom action bar

### Step 14 — Audit Form (`modules/jobs/pages/AuditForm.tsx`)
- [ ] Complaint textarea, diagnosis textarea
- [ ] Mechanic assignment: dropdown (list of mechanics from API)
- [ ] Bay selection: dropdown of free bays
- [ ] Preferred contact channel: email/SMS/phone
- [ ] Submit → transitions job to AUDIT status

### Step 15 — Quote Builder (`modules/jobs/pages/QuoteBuilder.tsx`)
- [ ] Parts section: search inventory (inline stock status), add row, quantity, unit price
- [ ] Labour section: description, hours, rate
- [ ] Auto-calculates: subtotal, tax (if billing.vat flag on), total
- [ ] Add/remove line items dynamically
- [ ] Submit → transitions to QUOTE status

### Step 16 — Approval Flow (`modules/jobs/components/ApprovalSection.tsx`)
- [ ] Display quote summary
- [ ] "Notify Customer" button → sends email/SMS with portal token link
- [ ] Record approval: "Customer Approved" button (staff records approval)
- [ ] If jobs.approval_workflow = false, skip this step

### Step 17 — Change Request (`modules/jobs/components/ChangeRequestModal.tsx`)
- [ ] Mechanic submits: add item (part/labour), reason
- [ ] Notification sent to customer
- [ ] Manager/Account Manager approves or declines
- [ ] Approved items auto-added to job card items

### Step 18 — QA Screen (`modules/jobs/pages/QAScreen.tsx`)
- [ ] Left panel: Intake checklist (read-only reference)
- [ ] Right panel: QA checklist (all complaint items + change request items checked)
- [ ] PASS → transitions to BILLING
- [ ] FAIL → returns to IN_SERVICE with defect notes, increments qa_cycles

---

## Phase 6 — Inventory & Purchasing
**Status:** 📋 Planned

### Step 19 — Inventory List (`modules/inventory/pages/InventoryList.tsx`)
- [ ] Table: Item Name, SKU, Category, Current Stock, Min Stock, Status (OK/Low/Out), Actions
- [ ] Low stock filter toggle
- [ ] Category filter
- [ ] "Add Item" button → opens create form

### Step 20 — Inventory Create/Edit (`modules/inventory/pages/InventoryForm.tsx`)
- [ ] Fields: name, SKU (auto-generated option), category, description, unit, cost price, selling price, min stock level
- [ ] Barcode field + scan button (react-webcam or manual input)
- [ ] Submit → creates/updates inventory item

### Step 21 — Stock Adjustment Modal (`modules/inventory/components/StockAdjustmentModal.tsx`)
- [ ] Radix Dialog: adjustment type (add/remove), quantity, reason
- [ ] Submit → calls POST /api/v1/inventory/:id/adjust

### Step 22 — Purchase Order List (`modules/purchases/pages/POList.tsx`)
- [ ] Table: PO #, Supplier, Status (DRAFT→SUBMITTED→APPROVED→SENT→IN_TRANSIT→RECEIVED), Items Count, Total, Actions
- [ ] Filter by status, supplier
- [ ] Timeline view available

### Step 23 — PO Create/Edit (`modules/purchases/pages/POForm.tsx`)
- [ ] Supplier dropdown, items table (add/remove rows), notes
- [ ] DRAFT only editable; beyond that, read-only with timeline

### Step 24 — GRN Entry (`modules/purchases/pages/GRNForm.tsx`)
- [ ] Display PO items with expected vs received quantity inputs
- [ ] Submit → creates GRN, stocks inventory

### Step 25 — QA Inspection (Purchases) (`modules/purchases/pages/QAInspectionForm.tsx`)
- [ ] Per GRN item: pass/fail, photo capture for defects, notes
- [ ] If purchases.qa_required flag on

---

## Phase 7 — Billing & Invoicing
**Status:** 📋 Planned

### Step 26 — Invoice List (`modules/billing/pages/InvoiceList.tsx`)
- [ ] Table: Invoice #, Customer, Job #, Amount, Status (DRAFT/ISSUED/PAID/VOID), Date, Actions
- [ ] Filter by status, customer, date range

### Step 27 — Invoice Detail (`modules/billing/pages/InvoiceDetail.tsx`)
- [ ] Invoice header: invoice #, customer, job link, dates
- [ ] Line items table: description, quantity, unit price, total
- [ ] Tax line (if billing.vat on), grand total
- [ ] "Record Payment" button → opens payment modal
- [ ] "Download PDF" button → opens /api/v1/invoices/:id/pdf in new tab
- [ ] "Void Invoice" button → confirmation dialog

### Step 28 — Payment Modal (`modules/billing/components/PaymentModal.tsx`)
- [ ] Amount paid, payment method (cash/card/bank transfer), notes
- [ ] Submit → records payment, updates invoice status

---

## Phase 8 — DVI (Digital Vehicle Inspection)
**Status:** 📋 Planned

### Step 29 — DVI Template List (`modules/dvi/pages/DVITemplateList.tsx`)
- [ ] Table: Template Name, Location, Created, Actions
- [ ] "Create Template" → opens editor

### Step 30 — DVI Template Editor (`modules/dvi/pages/DVITemplateEditor.tsx`)
- [ ] Configurable checklist builder: add/remove/reorder inspection items
- [ ] Categories: Engine, Brakes, Suspension, etc.
- [ ] Save template

### Step 31 — DVI Result Entry (`modules/dvi/pages/DVIResultEntry.tsx`)
- [ ] Mobile-optimized: large checkboxes, photo capture per item
- [ ] Select template → fill inspection items → submit
- [ ] Mechanic view: simple, touch-friendly

---

## Phase 9 — Assets (Flag-Gated)
**Status:** 📋 Planned

### Step 32 — Asset List (`modules/assets/pages/AssetList.tsx`)
- [ ] Table: Asset Name, Category, Location, Status, Last Inspection, Actions
- [ ] Filter by category, status, location

### Step 33 — Daily Inspection (`modules/assets/pages/DailyInspection.tsx`)
- [ ] Mobile-first: per-asset checklist, photo capture for defects
- [ ] "Submit Inspection" → creates asset_inspections record

### Step 34 — Defect Report (`modules/assets/components/DefectReportForm.tsx`)
- [ ] Photo capture, severity picker (low/medium/high), description
- [ ] Submit → creates asset_defects record

---

## Phase 10 — HR & Payroll (Flag-Gated)
**Status:** 📋 Planned

### Step 35 — Employee List (`modules/hr/pages/EmployeeList.tsx`)
- [ ] Table: Name, Employee ID, Role, Employment Type, Status, Actions

### Step 36 — Payroll Periods (`modules/hr/pages/PayrollPeriodList.tsx`)
- [ ] Table: Period, Status (DRAFT/RUN/APPROVED), Total Payroll, Actions
- [ ] "Run Payroll" → calculates per employee with deductions
- [ ] "Approve & Export" → exports Excel

### Step 37 — Leave Requests (`modules/hr/pages/LeaveRequestList.tsx`)
- [ ] Manager view: pending requests with approve/reject
- [ ] Employee view: submit leave request form

### Step 38 — Attendance (`modules/hr/pages/AttendanceView.tsx`)
- [ ] Calendar view + table
- [ ] Clock-in/out buttons (for employees)

---

## Phase 11 — Reports & Settings
**Status:** 📋 Planned

### Step 39 — Report Builder (`modules/reports/pages/ReportBuilder.tsx`)
- [ ] Select report type → configure filters (date range, customer, vehicle, etc.) → preview → export
- [ ] Saved reports list with re-run option

### Step 40 — Settings Pages (`modules/settings/`)
- [ ] Workshop Profile: name, address, phone, email, logo upload
- [ ] Locations: CRUD table
- [ ] Users & Roles: CRUD table with role assignment
- [ ] Bay Management: CRUD table with status toggle
- [ ] Feature Flags: tenant-level overrides (Owner/Admin only), toggle switches
- [ ] Notification Preferences: email SMTP config, SMS config, test buttons
- [ ] Intake Template Editor: checklist builder
- [ ] DVI Template Management: link to DVI module
- [ ] Leave Types: CRUD for HR leave types

---

## Phase 12 — Performance, Accessibility & Polish
**Status:** 📋 Planned

### Step 41 — Route-level code splitting
- [ ] const DashboardPage = lazy(() => import('@/modules/dashboard/DashboardPage'));
- [ ] Wrap in `<Suspense>` with `<LoadingSpinner />`

### Step 42 — PWA setup (vite-plugin-pwa)
- [ ] Configure vite.config.ts with PWA plugin
- [ ] Service worker strategy: stale-while-revalidate for API, cache-first for static
- [ ] Manifest: name, short_name, icons, theme_color = #0C0D0E, display=standalone
- [ ] Offline read: cache recent job/customer/vehicle lists

### Step 43 — Accessibility audit
- [ ] Run axe-core or manual: all interactive elements have cursor-pointer
- [ ] All images have alt text
- [ ] Form inputs have `<label>`
- [ ] Color contrast 4.5:1 minimum
- [ ] prefers-reduced-motion respected
- [ ] Focus indicators visible

### Step 44 — Responsive pass at 375px
- [ ] Test every page at 375px width
- [ ] Sticky bottom action bars for mobile forms
- [ ] Tables: horizontal scroll (overflow-x-auto) not card layout
- [ ] Touch targets minimum 44px

### Step 45 — Tests
- [ ] Each page: test loading state, empty state, error state, happy path
- [ ] Each form: test validation errors, submit success
- [ ] msw handlers for all API endpoints
- [ ] Run: npm test, npm run lint, npm run typecheck, npm run build

---

## Execution Order (Priority)

| Priority | Steps | Notes |
|----------|-------|-------|
| P0 | 1–3, 6–10 | Foundation + CRM |
| P0 | 11–18 | Job Card (core business logic) |
| P1 | 19–25 | Inventory + Purchasing |
| P1 | 26–28 | Billing |
| P2 | 29–31 | DVI |
| P2 | 32–34 | Assets (flag-gated) |
| P2 | 35–38 | HR (flag-gated) |
| P3 | 39–40 | Reports + Settings |
| P3 | 41–45 | Polish, PWA, tests |

---

## Quick Reference

### Phase Summary
| Phase | Name | Status |
|-------|------|--------|
| 0 | Infrastructure | ✅ Done |
| 1 | Auth & Foundation | ✅ Done |
| 2 | Shared Components & Hooks | 🛠️ Next |
| 3 | Dashboard & Navigation | 📋 |
| 4 | CRM (Customers & Vehicles) | 📋 |
| 5 | Job Card System | 📋 |
| 6 | Inventory & Purchasing | 📋 |
| 7 | Billing & Invoicing | 📋 |
| 8 | DVI | 📋 |
| 9 | Assets | 📋 |
| 10 | HR & Payroll | 📋 |
| 11 | Reports & Settings | 📋 |
| 12 | Performance & Polish | 📋 |

### Key Technical Decisions
- No framer-motion unless added to package.json
- TanStack Table v8 for all data tables with overflow-x-auto wrapper for mobile
- Radix Dialog for all modals (accessible by default)
- React Hook Form + Zod for all forms with FormField shared component
- TanStack Query for all server state with extracted hooks
- Zustand only for auth + UI state, not server data
- Recharts for dashboard charts and KPI sparklines
- react-webcam for photo capture, react-signature-canvas for signatures
- Feature flag guards: conditionally render routes/modules

### Docker Commands
```bash
make up          # Start containers
make down        # Stop containers
make restart     # Restart containers
make test-api    # Run API tests
make test-web    # Run web tests
make lint        # Run lint
make typecheck   # Run typecheck
```

### Git Convention
Commit messages follow Conventional Commits:
- `type(scope): summary`
- Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`