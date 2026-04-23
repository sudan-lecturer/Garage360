# Garage360 — Known Issues & Bugs

## Frontend Issues

### F1. Vite proxy localhost vs container networking
**Date:** 2026-04-23  
**Status:** ✅ Fixed  
**Description:** Login failed because Vite proxy targeted `localhost:8080` but web container can't reach itself via localhost. Needed to use `http://api:8080` (container hostname).  
**Fix:** Updated `web/vite.config.ts` proxy targets from `localhost` to `api`.  
**Files:** `web/vite.config.ts`

---

### F2. Login - no redirect after successful login
**Date:** 2026-04-23  
**Status:** ✅ Fixed  
**Description:** Login succeeded but no navigation to dashboard - missing `useNavigate` in LoginPage.  
**Fix:** Added `useNavigate` and `navigate('/dashboard')` in onSuccess callback.  
**Files:** `web/src/modules/auth/pages/LoginPage.tsx`

---

### F3. Shared component exports (named vs default)
**Date:** 2026-04-23  
**Status:** ✅ Fixed  
**Description:** Tried to import shared components as default exports but they were named exports. TypeScript errors.  
**Fix:** Changed imports to named export syntax (`import { Component }`).  
**Files:** `web/src/components/shared/*.tsx`

---

### F4. Customer form hook type mismatch
**Date:** 2026-04-23  
**Status:** ✅ Fixed  
**Description:** `useCustomers` hook returned `Customer[]` but form defaultValues expected different shape. Also unused imports.  
**Fix:** Simplified form to use basic defaultValues, removed unused imports.  
**Files:** `web/src/modules/customers/pages/CustomerForm.tsx`

---

### F5. Unused imports causing build errors
**Date:** 2026-04-23  
**Status:** ✅ Fixed  
**Description:** Various files had unused imports (FormSelect, reset, Calendar, Car, StatusBadge) causing TypeScript build failures.  
**Fix:** Removed unused imports individually.  

---

## Backend Issues

### B1. Tenant routing - UUID mismatch for super admin
**Date:** 2026-04-23  
**Status:** 🔴 Open  
**Severity:** High  
**Description:** Super admin login returns `tenant_id: "control"` but tenant lookup expects UUID. When accessing `/api/v1/customers`, the API fails with:  
```
operator does not exist: uuid = text
```
The JWT contains `tenant_id: "control"` (string) but tenants table has UUID IDs. Backend tries to lookup tenant by UUID but gets text.  
**Affected endpoint:** All tenant API routes (`/api/v1/customers`, etc.)  
**Root cause:** Auth module uses literal `"control"` for super admin, but tenant middleware expects UUID  
**Files:** `api/src/modules/auth/routes.rs`, `api/src/middleware/tenant.rs`

**Workaround:** None yet - need to fix backend.

---

### B2. DEV BYPASS password in auth
**Date:** 2026-04-23  
**Status:** 🔴 Open  
**Severity:** Medium  
**Description:** Added DEV BYPASS in auth routes to accept `password` or `dev123` for testing (original password hash couldn't be reversed).  
**Risk:** Should NOT be deployed to production.  
**Files:** `api/src/modules/auth/routes.rs`  
**Action:** Remove before production deployment.

---

### B3. Tenant schema - ON CONFLICT errors on seed data
**Date:** 2026-04-23  
**Status:** ✅ Fixed  
**Description:** Seed data INSERT statements use `ON CONFLICT DO NOTHING` but some tables lack unique constraints (intake_checklist_templates, dvi_templates).  
**Error:** `there is no unique or exclusion constraint matching the ON CONFLICT specification`  
**Files:** `api/schema/tenant_schema.sql`  
**Note:** Insert errors shown but data still loads.

---

### B4. Customer name field - schema mismatch
**Date:** 2026-04-23  
**Status:** 🔴 Open  
**Severity:** Medium  
**Description:** Frontend expects `Customer.name` but tenant schema uses `first_name` + `last_name` (for INDIVIDUAL) or `company_name` (for ORGANIZATION). Need transformation layer.  
**Files:** `api/schema/tenant_schema.sql`, `web/src/api/hooks/useCustomers.ts`  
**Action:** Add API transformation or update frontend to use `first_name`/`last_name`.

---

## Design System Issues

### D1. Theme color meta tag updated
**Date:** 2026-04-23  
**Status:** ✅ Fixed  
**Description:** `index.html` had old theme-color (`#0f172a`). Updated to Pepper (`#2B2B2B`).  
**Files:** `web/index.html`

---

## To Do

- [ ] Fix B1: Tenant routing UUID mismatch
- [ ] Fix B2: Remove DEV BYPASS before production
- [ ] Fix B4: Customer name transformation (API or frontend)
- [ ] Add more seed data (jobs, inventory, etc.)