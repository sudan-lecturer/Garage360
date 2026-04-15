---
name: project-manager
description: >
  Generates project management artifacts for Garage360: user stories, acceptance criteria, task
  breakdowns, sprint plans, Definition of Done, and GitHub Issues. Use this skill whenever
  planning work, breaking down a feature, writing user stories for a module, creating a sprint
  plan, prioritising backlog items, or generating GitHub Issue templates. Triggers on: "write
  user stories for X", "break down the job card feature", "create a sprint plan", "what tasks
  are needed for inventory", "generate GitHub issues", "plan the next sprint", "create a backlog".
  Follows INVEST criteria for stories, three-layer acceptance criteria, and story-point sizing.
---

# Project Manager

Generates all project management artifacts for Garage360 development following structured,
engineering-actionable practices. Every story is independently deliverable, testable, and sized.

---

## User Story Format

```
As a [role],
I want to [action/goal],
So that [business value].

Acceptance Criteria:
  Given [context]
  When [action]
  Then [expected outcome]

  Given [context]
  When [action]
  Then [expected outcome]

Definition of Done:
  - [ ] Unit tests written and passing
  - [ ] Integration tests written and passing
  - [ ] API endpoint documented
  - [ ] Feature flag gate applied (if applicable)
  - [ ] Mobile view tested at 375px
  - [ ] Code reviewed and approved
  - [ ] Merged to main branch

Story Points: [1 | 2 | 3 | 5 | 8 | 13]
Priority: [P0 | P1 | P2 | P3]
Module: [module name]
```

Story point scale:
- 1 — Trivial (config change, label update, tiny bug fix)
- 2 — Small (single endpoint, single form field, simple component)
- 3 — Medium (full CRUD endpoint + basic UI)
- 5 — Large (complex flow, multi-step UI, state machine step)
- 8 — Very large (full module, complex integrations, multi-service)
- 13 — Epic (break this down before starting)

---

## Complete User Story Backlog — Garage360

### Epic 1 — Auth & Access (Phase 1)

**US-001** — Login
```
As a workshop staff member,
I want to log in with my email and password,
So that I can access the system securely.

AC:
  Given valid credentials and an active account
  When I submit the login form
  Then I receive a JWT access token and am redirected to the dashboard

  Given invalid credentials
  When I submit the login form
  Then I see "Invalid email or password" and remain on the login page

  Given 5 consecutive failed attempts
  When I try again within 15 minutes
  Then I see "Too many attempts, please wait"

Story Points: 3 | Priority: P0 | Module: auth
```

**US-002** — Token refresh
```
As a logged-in staff member,
I want my session to silently refresh,
So that I don't get logged out mid-task.

AC:
  Given a valid refresh token
  When my access token expires (15 min)
  Then a new access token is issued transparently

  Given an invalid or expired refresh token
  When the app tries to refresh
  Then I am redirected to the login screen

Story Points: 2 | Priority: P0 | Module: auth
```

**US-003** — Role-based sidebar
```
As a system user,
I want to see only the modules I have access to,
So that the interface is not cluttered with irrelevant features.

AC:
  Given I am logged in as a MECHANIC
  When I view the sidebar
  Then I see only: Dashboard, Jobs, Assets, DVI

  Given a feature flag is disabled by admin
  When I view the sidebar
  Then the corresponding module is completely absent (not greyed out)

Story Points: 3 | Priority: P0 | Module: auth, settings
```

---

### Epic 2 — Customer Management (Phase 2)

**US-010** — Create individual customer
```
As a service manager,
I want to create an individual customer profile,
So that I can attach vehicles and jobs to them.

AC:
  Given I fill in full_name and phone
  When I submit the create customer form
  Then a new customer is created and I am redirected to their profile page

  Given I submit without full_name
  When the form validates
  Then I see "Full name is required" inline on the field

  Given a phone number already exists on another customer
  When I submit
  Then I see "Phone number already registered to [customer name]"

Story Points: 3 | Priority: P0 | Module: customers
```

**US-011** — Create organisation customer
```
As a service manager,
I want to create an organisation customer with a primary contact,
So that I can manage corporate fleet accounts.

AC:
  Given I select "Organisation" customer type
  When the form renders
  Then I see company_name, tax_id, billing_address, primary_contact_name, primary_contact_phone

  Given company_name is filled but primary_contact_name is empty
  When I submit
  Then I see "Primary contact name is required"

Story Points: 3 | Priority: P1 | Module: customers
```

**US-012** — Customer search
```
As a service manager,
I want to search customers by name, phone, or license plate,
So that I can quickly find a customer when they arrive.

AC:
  Given a customer exists with license plate "BA 1 PA 1234"
  When I type "BA 1 PA" in the search bar
  Then the customer appears in results within 500ms

  Given I search by customer phone number
  When results load
  Then the matching customer is shown with their vehicle count

Story Points: 3 | Priority: P0 | Module: customers
```

---

### Epic 3 — Vehicle Management (Phase 2)

**US-020** — Register vehicle with license plate
```
As a service manager,
I want to register a vehicle using its license plate,
So that I can create job cards linked to that vehicle.

AC:
  Given license_plate "BA 1 PA 1234" and a valid customer_id
  When I submit the create vehicle form
  Then the vehicle is created and linked to the customer

  Given I try to register a license plate that already exists
  When I submit
  Then I see "License plate already registered"

  Given no VIN is provided
  When I submit
  Then the vehicle is created without VIN (VIN is optional)

Story Points: 2 | Priority: P0 | Module: vehicles
```

---

### Epic 4 — Job Card Lifecycle (Phase 3)

**US-030** — Create job card at intake
```
As a service manager,
I want to create a new job card when a customer arrives,
So that the workshop can begin processing the vehicle.

AC:
  Given a valid customer and vehicle
  When I create a new job card
  Then a job card is created with status INTAKE and a unique job number (JC-YYYY-NNNNN)

  Given the same vehicle already has an open job
  When I try to create another job for it
  Then I see "Vehicle already has an open job card [JC-2024-00045]"

Story Points: 3 | Priority: P0 | Module: jobs
```

**US-031** — Complete intake checklist
```
As a service manager,
I want to complete a vehicle intake checklist with the customer present,
So that the vehicle's condition is documented before work begins.

AC:
  Given a job card at INTAKE status
  When I complete all mandatory checklist items and submit
  Then the intake checklist is saved and I can proceed to capture signature

  Given a mandatory item (e.g. odometer reading) is left empty
  When I try to submit the checklist
  Then I see an inline error on that field

  Given I am on mobile (375px)
  When I complete the checklist
  Then each item is accessible without horizontal scrolling

Story Points: 5 | Priority: P0 | Module: jobs/intake
```

**US-032** — Capture customer signature
```
As a service manager,
I want to capture the customer's digital signature on the intake form,
So that there is legal proof of the vehicle's recorded condition at drop-off.

AC:
  Given the intake checklist is complete
  When the customer draws their signature on the signature pad
  Then the signature is saved and the job advances to AUDIT

  Given the signature is captured
  When I request the intake PDF
  Then a PDF is generated showing checklist results, vehicle photos, and the signature

  Given I am on a mobile device
  When the signature pad renders
  Then it fills the screen width and has a visible "Clear" and "Confirm" button

Story Points: 5 | Priority: P0 | Module: jobs/intake
```

**US-033** — Assign mechanic and bay
```
As a service manager,
I want to assign a mechanic and a service bay to a job,
So that the workshop floor knows who is working where.

AC:
  Given a job at AUDIT status
  When I select an available mechanic and an available bay
  Then both are saved and the job is ready to move to QUOTE

  Given a bay is already OCCUPIED
  When I try to assign it to this job
  Then it does not appear in the available bays list

  Given two managers assign the same bay simultaneously
  When the second assignment completes
  Then it receives a 409 response and the bay board refreshes

Story Points: 3 | Priority: P0 | Module: jobs, bays
```

**US-034** — Bay board live view
```
As a service manager,
I want to see all service bays and their current status on one screen,
So that I can manage floor capacity at a glance.

AC:
  Given there are 6 bays at this location
  When I open the Bay Board
  Then I see all 6 bays with status (AVAILABLE/RESERVED/OCCUPIED), current job, mechanic name, and estimated free time

  Given a bay's estimated completion time has passed
  When I view the bay board
  Then the bay card shows "Overdue by X min" in red

  Given a job transitions to COMPLETED
  When the bay board refreshes
  Then that bay shows as AVAILABLE within 30 seconds

Story Points: 5 | Priority: P0 | Module: jobs/bays
```

**US-035** — Mid-service change request
```
As a mechanic,
I want to request approval for additional parts or work discovered during service,
So that I can proceed without doing unapproved work.

AC:
  Given a job in IN_SERVICE
  When I add a change request for a new part
  Then the customer is notified via their preferred channel (email/SMS/phone)

  Given the customer approves the change request
  When approval is recorded
  Then the items are added to the job card and stock is reserved

  Given the customer declines
  When decline is recorded
  Then I am notified and the item does not appear on the job

Story Points: 5 | Priority: P1 | Module: jobs/change_requests
```

**US-036** — Post-service QA
```
As a QA technician,
I want to review completed work against the intake checklist,
So that I can confirm all issues are resolved before handover.

AC:
  Given a job at QA status
  When I open the QA screen
  Then I see the intake checklist results alongside QA fields (before/after comparison)

  Given all QA items pass
  When I submit the QA form
  Then the job advances to BILLING

  Given any QA item fails
  When I submit with fail notes
  Then the job returns to IN_SERVICE with the mechanic notified of the specific failures

Story Points: 5 | Priority: P0 | Module: jobs/qa
```

---

### Epic 5 — Inventory (Phase 4)

**US-040** — Add inventory item
```
As a manager,
I want to add a new part to the inventory catalogue,
So that mechanics can select it when building job estimates.

AC:
  Given I fill in name, cost_price, retail_price, min_stock_level, reorder_point
  When I submit the Add Inventory form
  Then the item is created with stock_qty = 0

  Given I tap the barcode scan button on mobile
  When I scan a barcode
  Then the barcode field is auto-filled

Story Points: 3 | Priority: P0 | Module: inventory
```

**US-041** — Stock threshold alerts
```
As a manager,
I want to be notified when a part falls below its reorder point,
So that I can create a purchase order before stock runs out.

AC:
  Given an item with reorder_point = 5 and stock_qty = 6
  When a job uses 2 units (stock drops to 4)
  Then a LOW STOCK alert is created and a push notification is sent to all managers

  Given stock_qty drops to or below min_stock_level
  When stock is updated
  Then a CRITICAL alert badge appears in red on the inventory module icon

Story Points: 3 | Priority: P1 | Module: inventory
```

---

### Epic 6 — Purchase Orders (Phase 4)

**US-050** — Full PO lifecycle
```
As a manager,
I want to create, approve, send, and receive purchase orders,
So that stock replenishment is traceable end-to-end.

AC:
  Given I create a PO with line items and submit for approval
  When an owner/admin approves it
  Then status changes to APPROVED and I can mark it as sent to supplier

  Given a PO is IN_TRANSIT
  When I open the in-transit dashboard
  Then I see it listed with expected delivery date and days since dispatch

  Given I create a GRN for received goods
  When all line items have QA results
  Then QA-passed items are added to inventory stock automatically

Story Points: 8 | Priority: P1 | Module: purchases
```

---

### Epic 7 — Excel Import / Export (Phase 9)

**US-060** — Export any list to Excel
```
As any staff member,
I want to export the currently visible list to an Excel file,
So that I can share data or analyse it offline.

AC:
  Given I have applied filters (e.g. status = COMPLETED, date range last 30 days)
  When I click "Export to Excel"
  Then a .xlsx file downloads containing only the filtered rows with correct column headers

Story Points: 2 | Priority: P2 | Module: import_export
```

**US-061** — Import customers from Excel
```
As a manager,
I want to import a batch of customers from an Excel template,
So that I can onboard existing customer data quickly.

AC:
  Given I upload a correctly formatted .xlsx file
  When the system parses it
  Then I see a preview of the first 10 rows with any validation errors highlighted per cell

  Given row 14 has a missing phone number (required field)
  When the preview shows
  Then row 14 is highlighted in red with tooltip "Phone is required"

  Given I confirm the import
  When it completes
  Then I receive a notification: "Imported 47 customers. Skipped 3 (duplicates). Failed 1 (see errors)."

Story Points: 5 | Priority: P2 | Module: import_export
```

---

### Epic 8 — HR & Payroll (Phase 8)

**US-070** — Employee record
```
As an HR officer,
I want to create and maintain employee records,
So that all staff data is centralised and secure.

AC:
  Given I create an employee record linked to a user account
  When I save it
  Then sensitive fields (bank account, national ID) are encrypted at rest

  Given I am logged in as MECHANIC
  When I try to access another employee's bank details
  Then I receive 403 Forbidden

Story Points: 5 | Priority: P2 | Module: hr
```

**US-071** — Monthly payroll run
```
As an HR officer,
I want to run payroll for a selected period,
So that net pay is calculated correctly for all employees.

AC:
  Given payroll deduction configs are set (Income Tax slab, PF 10%, SSF 1%)
  When I run payroll for March 2025
  Then each employee's gross, deductions, and net are calculated correctly

  Given I approve the payroll period
  When entries are locked
  Then no further edits are possible and the period is exportable as Excel

Story Points: 8 | Priority: P2 | Module: hr
```

---

## Sprint Planning Template

### Sprint Structure
- Sprint length: 2 weeks
- Sprint ceremonies: Planning (Mon AM), Daily standup (15 min), Review + Retro (last Fri)
- Velocity target: 30-40 story points per sprint (4-5 developers)

### Sprint 1 — Foundation (Weeks 1-2)
| Story | Points | Assignee |
|---|---|---|
| US-001 Login | 3 | — |
| US-002 Token refresh | 2 | — |
| US-003 Role-based sidebar | 3 | — |
| Docker setup + local dev | 3 | — |
| CI pipeline (lint + test) | 3 | — |
| DB schema + tenant provisioning | 5 | — |
| AppState + DI wiring | 3 | — |
| Design system tokens + base components | 5 | — |
| **Total** | **27** | |

### Sprint 2 — CRM Core (Weeks 3-4)
| Story | Points | Assignee |
|---|---|---|
| US-010 Create individual customer | 3 | — |
| US-011 Create org customer | 3 | — |
| US-012 Customer search | 3 | — |
| US-020 Register vehicle | 2 | — |
| Customer list + detail screens | 3 | — |
| Vehicle list + detail screens | 2 | — |
| Excel export (customers + vehicles) | 2 | — |
| **Total** | **18** | |

*(Continue this pattern for all 12 phases from the Master Plan)*

---

## GitHub Issue Template

When generating GitHub Issues from user stories:

```markdown
## User Story
[Paste the user story text]

## Acceptance Criteria
- [ ] Given [context] When [action] Then [outcome]
- [ ] Given [context] When [action] Then [outcome]

## Technical Notes
- API endpoint: `POST /api/v1/customers`
- DB tables affected: `customers`
- Feature flag: N/A
- RBAC: Manager, Admin, Owner

## Definition of Done
- [ ] Unit tests written and passing (`cargo test`)
- [ ] Integration tests written and passing
- [ ] Mobile view tested at 375px
- [ ] API response matches typed interface
- [ ] Code reviewed and approved (1 approval minimum)
- [ ] Merged to `main` branch

**Story Points:** 3
**Priority:** P0
**Module:** customers
**Sprint:** Sprint 2
```

---

## Definition of Done (Global)

Applied to every story before it can be marked complete:

### Code Quality
- [ ] No compiler warnings (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`, `prettier --check`)
- [ ] No `unwrap()` in production paths — all errors handled via `AppError`
- [ ] No `TODO` comments left in the PR

### Testing
- [ ] Service layer unit tests pass
- [ ] Integration tests pass against test DB
- [ ] Frontend component tests pass (`vitest run`)
- [ ] New code does not decrease overall coverage below targets

### Documentation
- [ ] New API endpoints documented (inline doc comments)
- [ ] Any new tenant_settings keys added to `tenant-settings-registry.md`
- [ ] Schema changes reflected in `canonical-schema.sql`

### UX
- [ ] Tested at 375px (mobile), 768px (tablet), 1280px (desktop)
- [ ] Loading states present on all async actions
- [ ] Error states handled with user-facing messages (not raw error codes)
- [ ] Accessibility: keyboard-navigable, focus-visible

### Deployment
- [ ] CI pipeline passes (lint + test + build)
- [ ] No new secrets hardcoded anywhere
- [ ] Feature flag gate applied where appropriate

---

## Reference Files

- `references/all-user-stories.md` — Complete backlog (all epics, all stories)
- `references/sprint-plans.md` — All 12 sprints broken down
- `references/github-issue-templates.md` — Issue and PR templates
