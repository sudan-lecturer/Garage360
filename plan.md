# Garage360 Frontend vs Stitch Comprehensive Plan

## Scope

This document captures a comprehensive comparison between:

- Stitch design package: `/Users/sudan/Downloads/stitch_garage360.zip`
- Current frontend implementation: `web/`

It includes findings, gaps, recommendations, and implementation path options for decision-making before development.

---

## Sources Reviewed

- Stitch package screens and design spec
  - `stitch_garage360/torque_tactical/DESIGN.md`
  - 45 `code.html` screens and associated screenshots
- Frontend implementation
  - `web/src/styles/globals.css`
  - `web/src/App.tsx`
  - `web/src/layouts/MainLayout.tsx`
  - `web/src/layouts/AuthLayout.tsx`
  - `web/src/components/shared/status-badge.tsx`
  - module pages under `web/src/modules/**/pages/*.tsx`

---

## Executive Summary

- Design direction mismatch is significant: Stitch defines an industrial brutalist identity, while current frontend is closer to modern SaaS style.
- Screen coverage is partial: Stitch has 45 variants / 23 base screen types; frontend has 19 implemented page components plus several placeholders.
- Structural alignment exists (module architecture and route grouping), but visual identity and interaction language diverge.
- Mobile readiness is responsive but not parity-level with Stitch mobile variants.

---

## Detailed Findings

## 1) Design System Differences

### 1.1 Color system

- Stitch:
  - Background-heavy dark palette around `#121416`
  - Surface around `#1E2226`
  - Primary signal yellow `#FFD100`
  - Secondary high-visibility orange `#FF5800`
- Frontend:
  - Background/surface base around `#2B2B2B`
  - Primary/accent amber `#F59E0B`
  - Uses a softer neutral contrast profile

Impact:
- Both are dark-first, but Stitch is sharper, more tactical, and higher-contrast in “industrial signal” moments.

### 1.2 Typography

- Stitch:
  - Barlow for headings
  - IBM Plex Mono for technical/data text
  - Work Sans for body
  - Heavy uppercase/tracking for labels/navigation
- Frontend:
  - Inter for heading/body
  - JetBrains Mono tokenized but not used as core data language

Impact:
- This is one of the biggest identity gaps.

### 1.3 Shape, elevation, and depth

- Stitch:
  - Sharp edges and tight radii
  - Tonal hierarchy over soft shadow
  - “No-line” and “no soft shadows” guidance in core concept
- Frontend:
  - Rounded card/button feel
  - Shadow tokens and softer containers

Impact:
- Current UI feels polished but consumer-SaaS, not workshop-command tactical.

### 1.4 Component behavior and states

- Stitch patterns:
  - Left accent rails, strong binary active states
  - High-contrast tactical CTA emphasis
  - Data-dense mono labeling style
- Frontend patterns:
  - Standard card/table + badge semantics
  - More conventional UX affordances

Impact:
- Interaction language is functionally good but stylistically inconsistent with Stitch intent.

---

## 2) Layout and Navigation Differences

### 2.1 Main shell

- Stitch:
  - Strong, persistent industrial side nav
  - Aggressive active highlighting
  - Command-style top bar and global search blocks
- Frontend (`MainLayout.tsx`):
  - Correct information architecture (top bar + sidebar + responsive mobile drawer)
  - Visual treatment is softer (rounded navigation, muted active style)

### 2.2 Auth shell

- Stitch login:
  - Split-panel cinematic layout
  - Industrial branding and role context
  - Heavy visual hierarchy and tactical copy
- Frontend (`AuthLayout.tsx` + `LoginPage.tsx`):
  - Minimal centered card flow
  - Lower visual distinction and brand signal

---

## 3) Screen Coverage and Functional Parity

### 3.1 Stitch package coverage

- 45 screen variants
- 23 base screen categories
- Includes desktop and mobile variants for key domains

### 3.2 Frontend coverage

Implemented major pages include:
- Dashboard
- Customers (list/detail/form)
- Vehicles (list/detail)
- Jobs (list/detail/form/intake)
- Inventory list
- Purchases list
- Billing list
- DVI template list
- Assets list
- HR employee list
- Settings
- Super Admin tenants

### 3.3 Route/page gaps

Placeholder or incomplete routes exist in `web/src/App.tsx`, including:
- Reports page placeholder
- Create PO placeholder
- New billing/inventory/vehicle placeholders
- Some job approval/QA/detail placeholders
- DVI template detail/editor placeholders

### 3.4 Stitch screens with no or weak equivalent

No clear equivalent (or incomplete equivalent) found for:
- Coupon management
- Payment confirmation
- Quote creation
- User role management
- User management
- Inventory item detail
- Reports dashboard parity (current reports route is placeholder)

---

## 4) Mobile Parity Assessment

- Stitch includes dedicated mobile compositions for core workflows (dashboard, customer list, job details, inventory, settings, DVI, etc.).
- Frontend is responsive and usable, but mostly desktop-first adaptation.
- Result: mobile usability exists, but visual/interaction parity with Stitch mobile designs is not comprehensive.

---

## 5) UX and Accessibility Observations

Strengths:
- Consistent module/page shell structure
- Good loading/error/empty-state handling in several pages
- Focus-visible global styles defined
- Robust form validation patterns in key forms

Gaps:
- Some icon-only controls lack explicit accessible labels
- Placeholder CTAs and routes reduce end-to-end UX integrity
- `status-badge.tsx` uses hardcoded Tailwind palettes, not full semantic token consistency

---

## 6) Quantified Parity Estimate

- Functional parity with product scope: 60% to 70%
- Visual/system parity with Stitch direction: 35% to 45%

Primary reason for lower visual parity:
- Typography + token divergence
- Component geometry/state language mismatch
- Missing high-priority Stitch screens

---

## Recommendation Matrix (Decision Table)

| Path | Focus | What gets done first | Pros | Risks | Estimated Effort |
|---|---|---|---|---|---|
| A | Design-system alignment first | Token/typography/nav shell alignment, shared primitives | Fastest way to close visual gap globally | Functional gaps remain temporarily | Medium |
| B | Core missing screens first | Reports, Create PO, Quote, User/Role management | Improves product completeness quickly | Visual inconsistency persists | Medium-High |
| C | Mobile-first parity first | Dedicated mobile versions for core journeys | Immediate field usability uplift | Desktop parity delayed | Medium |
| D | UX/accessibility hardening first | Labels, keyboard/focus quality, placeholder removal | Improves quality and readiness | Limited visual parity improvement | Low-Medium |
| E | Hybrid sprint | Partial token alignment + top 3 missing screens | Balanced progress across visuals + function | Coordination complexity | High |

---

## Recommended Execution Sequence

1. Design foundation convergence
   - Align key color tokens and type system with Stitch intent
   - Refactor shared primitives (buttons, inputs, cards, badges)
   - Update main and auth layout shells for tactical navigation language

2. Screen completion for high-impact business flows
   - Reports dashboard
   - Create purchase order
   - Quote creation
   - User and role management

3. Mobile parity uplift
   - Dashboard mobile
   - Customer list mobile
   - Job detail mobile

4. UX/accessibility and consistency hardening
   - Add aria-label/title where missing
   - Replace placeholder CTAs/routes with working flows
   - Standardize status styles on semantic tokens

---

## Implementation Backlog (Proposed)

### Phase 1: Foundation
- `web/src/styles/globals.css`
- `web/src/layouts/MainLayout.tsx`
- `web/src/layouts/AuthLayout.tsx`
- `web/src/components/ui/*`
- `web/src/components/shared/status-badge.tsx`

### Phase 2: Missing Core Screens
- `web/src/modules/reports/pages/*` (new)
- `web/src/modules/purchases/pages/POCreate.tsx` (new)
- `web/src/modules/jobs/pages/QuoteCreate.tsx` (new or module-aligned path)
- `web/src/modules/settings/pages/UserManagement.tsx` (new)
- `web/src/modules/settings/pages/RoleManagement.tsx` (new)
- Route wiring in `web/src/App.tsx`

### Phase 3: Mobile Refinements
- Responsive composition updates in:
  - `DashboardPage.tsx`
  - `CustomerList.tsx`
  - `JobDetail.tsx`

### Phase 4: Quality and Accessibility
- Add accessible names for icon-only controls
- Replace placeholder route components in `App.tsx`
- Validate all major flows with keyboard and mobile interaction checks

---

## Suggested Success Criteria

- Visual identity: app immediately reads as “industrial tactical” (type, color, nav, state language).
- Functional completeness: all top-priority Stitch business screens have implemented frontend equivalents.
- Mobile readiness: core 3 flows achieve consistent task completion on small screens.
- Accessibility baseline: no unlabeled icon-only interactive controls in core flows.

---

## Selected Direction

Chosen path: **B — Core missing screens first**

Reason this path is selected:
- Fastest way to close functional parity gaps with Stitch screen inventory
- Reduces placeholder routes and incomplete workflows early
- Gives immediate business-value coverage before full visual polish pass

---

## Path B Execution Plan (Locked)

### Phase B1: Replace Placeholder Routes With Real Screens

Target outcomes:
- No critical placeholder routes for top-priority modules
- Working pages for high-impact missing Stitch flows

Primary work:
- Implement reports dashboard page
- Implement create purchase order page
- Implement quote creation page
- Implement user management page
- Implement role management page
- Wire all new pages in `web/src/App.tsx`

Suggested file targets:
- `web/src/modules/reports/pages/ReportsDashboard.tsx` (new)
- `web/src/modules/purchases/pages/POCreate.tsx` (new)
- `web/src/modules/jobs/pages/QuoteCreate.tsx` (new)
- `web/src/modules/settings/pages/UserManagement.tsx` (new)
- `web/src/modules/settings/pages/RoleManagement.tsx` (new)
- `web/src/App.tsx`

### Phase B2: Align Existing Nearby Flows

Target outcomes:
- New screens integrate naturally with current module UX patterns
- Consistent loading/error/empty/form patterns across modules

Primary work:
- Reuse existing shared primitives (`PageHeader`, `SearchInput`, `StatusBadge`, `Button`, `Input`)
- Connect each new page to existing API layer/hooks where available
- Add temporary structured mock adapters only where backend APIs are not yet available

Suggested file targets:
- `web/src/api/client.ts`
- `web/src/api/hooks/*` (add only when justified)
- Existing module list/detail pages for linking entry points

### Phase B3: Functional Hardening of New Screens

Target outcomes:
- End-to-end navigation works from menus and list actions
- Core interactions are not placeholder-only

Primary work:
- Ensure buttons and actions execute meaningful flow
- Add minimal validation and error states
- Add route-level navigation entry points in layout or module pages

Suggested file targets:
- `web/src/layouts/MainLayout.tsx` (if nav entries need extension)
- New pages from Phase B1
- Related list/detail pages per module

### Phase B4: Quality Gate Before Visual Parity Pass

Target outcomes:
- Newly implemented screens are stable and testable
- Ready for later design-system convergence pass

Primary work:
- Add/extend tests for route rendering and key interactions
- Confirm no major accessibility misses on icon-only actions and forms
- Ensure no remaining critical placeholders for selected modules

Suggested file targets:
- `web/src/App.test.tsx`
- `web/src/modules/**/pages/*.test.tsx` (targeted additions)

---

## Delivery Order Under Path B

1. Reports dashboard
2. Create purchase order
3. Quote creation
4. User management
5. Role management
6. Route and navigation integration
7. Test and hardening pass

---

## Deferred Until After Path B

- Full visual system convergence to Stitch tokens/typography
- Deep mobile parity redesign for all major screens
- Advanced theme-level refactor across shared primitives

These remain important and will be taken up immediately after functional parity closure from Path B.

# Garage360 Frontend vs Stitch Comprehensive Plan

## Objective
Create a single execution reference that captures the full design audit findings between the Stitch package (`/Users/sudan/Downloads/stitch_garage360.zip`) and the current Garage360 frontend (`web/`), then define prioritized implementation options for the next development phase.

## Audit Scope Covered
- Stitch design system source (`stitch_garage360/torque_tactical/DESIGN.md`)
- Stitch screens: 45 variants across 23 base screen types (desktop, mobile, updated nav)
- Garage360 frontend routes, layouts, tokens, shared UI components, and module pages under `web/src`
- UX/accessibility/responsiveness signals from code implementation

## Key Findings

### 1) Design Direction Mismatch (Major)
- Stitch follows an industrial brutalist language ("Digital Foreman"): high-contrast tactical dark UI, sharp geometry, bold technical typography, and heavy operational tone.
- Current frontend uses a softer SaaS visual system with different core tokens and typography.
- Result: functional alignment exists, but visual identity alignment is significantly off.

### 2) Design Token and Typography Drift (Major)
- Stitch palette centers around:
  - Background: `#121416`
  - Surface: `#1E2226`
  - Primary: `#FFD100`
  - Secondary: `#FF5800`
- Current frontend (`web/src/styles/globals.css`) uses:
  - Background/Surface: `#2B2B2B`
  - Primary/Accent: `#F59E0B`
  - Typography: Inter + JetBrains Mono
- Stitch expects Barlow + IBM Plex Mono + Work Sans with uppercase technical rhythm.

### 3) Component Style and Interaction Delta (Major)
- Stitch specifies:
  - Very low radius / sharp edges
  - Strong left-accent rails and binary active states
  - Tonal layering over soft shadows
  - Industrial controls and tactical panel composition
- Current UI primitives and module implementations are cleaner and rounded by comparison.
- `web/src/components/shared/status-badge.tsx` uses hardcoded Tailwind ramps instead of unified semantic token mapping.

### 4) Layout and Navigation Differences (Major)
- Stitch "updated_nav" screens consistently use a tactical side navigation + technical top bar (global search/branch/status/operator).
- `web/src/layouts/MainLayout.tsx` already has fixed top + sidebar architecture, but treatment is visually softer and less industrial.
- Auth experience differs strongly:
  - Stitch login is split-panel, role-aware, tactical.
  - Current login (`AuthLayout`, `LoginPage`) is a minimal card form.

### 5) Screen Coverage Gaps (Major)
- Stitch includes 45 screen variants / 23 base screen categories.
- Frontend has 19 implemented page components, but several routes are placeholders in `web/src/App.tsx`.
- Missing or not yet equivalent to Stitch coverage:
  - Coupon management
  - User role management
  - User management
  - Quote creation
  - Payment confirmation
  - Inventory item detail
  - Reports dashboard parity
  - Create purchase order flow parity

### 6) Mobile Fidelity Gap (Medium-High)
- Stitch provides dedicated mobile compositions for key modules.
- Current frontend is responsive and mobile-friendly but mostly "adaptive desktop", not dedicated tactical mobile variants.

### 7) UX/Accessibility and Execution Completeness (Medium)
- Strengths:
  - Good route/module organization
  - Shared loading/error/empty state patterns
  - Focus-visible baseline in global styles
- Gaps:
  - Icon-only controls need explicit accessible names in several places
  - Placeholder routes/CTAs break end-to-end journey completeness
  - Some flows still stubbed/TODO-level

## Current Parity Assessment
- Functional parity (domain coverage): ~60-70%
- Visual/system parity (Stitch fidelity): ~35-45%
- Main reason for low visual parity: design language divergence + missing high-priority screens

## Recommended Execution Paths

### Path A: Design-System Alignment First
Best when visual consistency is top priority.

1. Align global tokens in `web/src/styles/globals.css` to Stitch palette and tonal hierarchy
2. Introduce Stitch typography stack and usage roles
3. Refactor shared primitives (`button`, `input`, badges, nav atoms) to industrial interaction patterns
4. Update `MainLayout` + `AuthLayout` to tactical shell patterns
5. Then execute missing screens with aligned system

Pros:
- Reduces rework during feature implementation
- Immediate visible cohesion across existing screens

Cons:
- Slower delivery of missing feature pages in the short term

### Path B: Core Missing Screens First
Best when feature completeness is top priority.

1. Implement reports dashboard
2. Implement create purchase order flow
3. Implement quote creation
4. Implement user management + role management
5. Add payment confirmation / inventory item detail if required
6. Polish visual system after feature closure

Pros:
- Faster functional parity with Stitch screen map

Cons:
- Increased risk of stylistic refactor debt later

### Path C: Mobile-First Parity
Best when operator mobile workflow is critical.

1. Build dedicated tactical mobile variants for:
  - Dashboard
  - Customer list
  - Job card detail
2. Standardize mobile nav, action patterns, and dense data readability
3. Backport desktop refinements afterward

Pros:
- Improves real-world usability quickly for on-floor use

Cons:
- Desktop visual drift remains until next phase

### Path D: UX/Accessibility Hardening First
Best when production stability and quality are top priority.

1. Remove/replace placeholder routes and CTA dead-ends
2. Add accessibility labels for icon controls
3. Standardize keyboard focus and interaction affordances
4. Normalize status badge semantics to design tokens
5. Then proceed with visual/design convergence

Pros:
- Fast quality uplift and lower user friction

Cons:
- Does not materially close Stitch visual gap early

### Path E: Hybrid Sprint (Recommended Balanced Option)
Best for balanced speed + quality.

Phase 1 (quick foundation):
- Token adjustments (high-impact subset)
- Typography alignment
- Nav shell restyling (`MainLayout`, `AuthLayout`)

Phase 2 (core completeness):
- Build top 3 missing pages:
  - Reports dashboard
  - Create purchase order
  - User role management

Phase 3 (hardening):
- Accessibility labels + placeholder replacement
- Status badge tokenization
- Mobile refinements for key flows

Pros:
- Delivers visible design improvement and feature progress together
- Keeps rework manageable

Cons:
- Requires tighter sequencing discipline

## Prioritized Implementation Backlog

### Priority 0 (Foundational)
- `web/src/styles/globals.css`: token convergence toward Stitch palette/typography hierarchy
- `web/src/layouts/MainLayout.tsx`: tactical nav visual language
- `web/src/layouts/AuthLayout.tsx` and `web/src/modules/auth/pages/LoginPage.tsx`: industrial login parity

### Priority 1 (Critical Missing Screens)
- `web/src/modules/reports/...` (new module, currently placeholder route)
- `web/src/modules/purchases/pages/...` (create PO page/flow)
- `web/src/modules/settings/...` or dedicated module for user/role management
- `web/src/App.tsx`: replace placeholder route elements with real pages

### Priority 2 (Quality + Consistency)
- `web/src/components/shared/status-badge.tsx`: semantic token mapping
- shared icon-only actions across layout/list pages: add `aria-label` and predictable keyboard behavior
- remove dead CTAs and stub actions

### Priority 3 (Mobile Tactical Parity)
- Dedicated mobile variants for dashboard, customer list, and job detail
- Improve dense-table mobile transformations where necessary

## Risks and Mitigations
- Risk: broad visual refactor can break consistency mid-way.
  - Mitigation: apply token and primitive-level changes first, then page-level updates.
- Risk: implementing screens before token alignment causes duplicate rework.
  - Mitigation: use hybrid plan with minimum token baseline first.
- Risk: accessibility debt hidden by visual focus.
  - Mitigation: enforce accessibility checklist per page before completion.

## Definition of Done for Next Phase
- Selected path approved
- No placeholder routes for selected scope
- Selected screens implemented with production-grade interactions
- Shared tokens/components aligned to agreed Stitch strictness level
- Accessibility checks pass for implemented scope
- Mobile behavior verified for selected screens

## Decision Needed
Choose one path to start:
- Path A (Design-system first)
- Path B (Core screens first)
- Path C (Mobile-first)
- Path D (UX/a11y first)
- Path E (Hybrid balanced)
- Custom scope

Once selected, implementation will proceed in that order.

---

## Operational Knowledge (Critical)

### Symptom Observed

- "Error loading data" appeared across most frontend screens.
- API responses for tenant module routes were returning `500`.

### Root Causes Identified

1. **Tenant provisioning failure**
   - Tenant DB bootstrap failed during `/control/v1/tenants` creation due to schema conflict handling on non-unique columns.
   - Failure mode in API logs:
     - `there is no unique or exclusion constraint matching the ON CONFLICT specification`
   - Downstream effect:
     - Tenant tables like `customers`, `job_cards`, `vehicles`, `inventory_items` were missing.
     - Resulting SQL errors:
       - `relation "..." does not exist`

2. **Role/context mismatch during login**
   - Super admin login (`admin@garage360.io`) authenticates to control-plane context.
   - Tenant app screens require tenant-scoped user context.
   - This created additional authorization/data-context confusion while testing app modules.

### Fix Applied

- Updated schema constraints in `api/schema/tenant_schema.sql` by making these `name` columns unique:
  - `intake_checklist_templates.name`
  - `dvi_templates.name`
  - `asset_inspection_templates.name`

- Rebuilt and restarted API container.
- Provisioned a tenant successfully via control API.
- Added/updated tenant admin credentials for app-level testing.

### Verified Working App Credentials

- Tenant user (recommended for frontend app modules):
  - Email: `admin@demo.com`
  - Password: `admin123`

### Verified Endpoints (tenant auth)

All returned `200` after fix and tenant provisioning:
- `/api/v1/dashboard/stats`
- `/api/v1/customers`
- `/api/v1/vehicles`
- `/api/v1/jobs`
- `/api/v1/inventory`
- `/api/v1/invoices`
- `/api/v1/settings/users`
- `/api/v1/reports/saved`

### Preventive Guidance

- Validate tenant creation success before QA on tenant module screens.
- For app QA, use tenant credentials (not super admin control credentials).
- Keep tenant bootstrap schema conflict targets aligned with actual unique constraints.

---

## Execution Progress Update (Current Block Complete)

The following pending Path-B continuation block has now been implemented:

1. **Inventory detail + stock adjustment flow**
   - Added inventory detail screen with stock adjustment action panel.
   - Wired to backend `GET /v1/inventory/:id` and `POST /v1/inventory/:id/adjust`.

2. **Invoice detail + payment flow**
   - Added invoice detail screen with line items and payment recording action panel.
   - Wired to backend `GET /v1/invoices/:id` and `POST /v1/invoices/:id/payment`.

3. **DVI result entry + detail**
   - Added DVI result creation screen and DVI result detail screen.
   - Wired to backend `POST /v1/dvi/results` and `GET /v1/dvi/results/:id`.

4. **Settings sub-pages backend wiring**
   - Settings page now uses real backend data/actions for:
     - Workshop profile
     - Locations list/create
     - Notification preferences read/update
     - Feature flags list/toggle

### Route Wiring Added

- `/inventory/:id`
- `/billing/:id`
- `/dvi/results/new`
- `/dvi/results/:id`

### API Mapping Hardening

To prevent repeat "data loading" regressions, the frontend hooks now normalize backend snake_case payloads/responses into stable UI-facing structures for billing, DVI, and settings modules.

### Next-Phase Follow-up Completed

- Converted the new inline action panels into **modal-style flows** for:
  - Inventory stock adjustment
  - Invoice payment recording
- Improved detail-route resiliency by keeping route headers visible during loading/empty states:
  - Inventory detail
  - Invoice detail
  - DVI result detail
- Extended route tests in `web/src/App.test.tsx` to include:
  - `/inventory/:id`
  - `/billing/:id`
  - `/dvi/results/new`
  - `/dvi/results/:id`
- Verification passed in Docker:
  - `npm run typecheck`
  - `npm run test -- src/App.test.tsx --run`

### Backend Validation Hardening (Latest)

- Inventory stock-adjust backend:
  - Added unit tests for adjustment-type validation and quantity decimal/zero constraints.
- Invoice payment backend:
  - Added normalization for `paymentMethod` (allowed set enforced).
  - Added normalization for `paymentRef` (trim/empty-safe).
  - Updated `paidAt` parsing to treat blank strings as unset instead of failing.
- Added focused Rust unit tests for these validation paths in:
  - `api/src/modules/inventory/service.rs`
  - `api/src/modules/billing/service.rs`
- Docker verification:
  - `docker compose -f docker-compose.dev.yml exec api cargo test service::tests` passed (`6` tests).

### API Integration Tests Added (Inventory + Billing)

- Added black-box integration test file:
  - `api/tests/payment_inventory_integration.rs`
- Coverage implemented in one end-to-end flow:
  1. Tenant login via `/api/v1/auth/login`
  2. Create inventory item via `/api/v1/inventory`
  3. Stock adjustment via `/api/v1/inventory/:id/adjust`
  4. Create customer via `/api/v1/customers`
  5. Create invoice via `/api/v1/invoices`
  6. Record payment via `/api/v1/invoices/:id/payment`
- Assertions include:
  - Inventory quantity update correctness
  - Payment amount update correctness
  - Payment method normalization behavior (`cash` -> `CASH`)
- Docker verification:
  - `docker compose -f docker-compose.dev.yml exec api cargo test --test payment_inventory_integration -- --nocapture` passed (`1` test).
