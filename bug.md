# Garage360 Bug Tracking

## Backend Bugs

### tenant_id mismatch prevents authenticated API calls
**Severity:** High
**Status:** Open

**Description:** 
After implementing tenant user login, the JWT token contains `tenant_id: "tenant_demo"` (text), but the `tenants` table uses UUID for the `id` column. When the tenant middleware (`TenantDbPool` extractor) tries to look up the tenant, it compares the UUID tenant ID from the token with the text "tenant_demo", causing an "operator does not exist: uuid = text" database error.

**Root Cause:**
- Login endpoint returns `tenant_id` as the database name (text) e.g., "tenant_demo"
- The `tenants` table has `id` as UUID type
- The query in `middleware/tenant.rs` at line 65 does: `WHERE id = $1` but binds a string to a UUID column

**Affected Endpoints:**
- All authenticated endpoints that use `TenantDbPool` extractor

**Fix Required:**
- Either: Return the UUID tenant_id in the JWT token
- Or: Modify the tenant lookup query to use `database_name` instead of `id`

---

### Dashboard API endpoints returning 500
**Severity:** High
**Status:** Open (blocked by tenant_id bug)

**Endpoints:**
- `GET /api/v1/dashboard/stats`
- `GET /api/v1/dashboard/recent-activities`
- `GET /api/v1/bays/board`

All return 500 Internal Server Error due to tenant_id mismatch issue above.

---

## Frontend-Bend Integration Bugs

### Response Format Mismatch
**Severity:** Medium
**Status:** Open

**Description:**
Frontend expects API responses to be wrapped in `{ data: ... }` format for certain endpoints, but the actual API returns unwrapped arrays or objects directly.

**Specific Issues:**
1. `useRecentActivity()` expects `response.data.data` but API returns `response.data` as array
2. `useBays()` expects `response.data.data` but API returns `response.data` as array
3. `useDashboardStats()` correctly uses `response.data` (this one works)

**Affected Components:**
- DashboardPage.tsx lines 56-65 (useRecentActivity)
- DashboardPage.tsx lines 67-75 (useBays)

**Fix Required:**
- Standardize on either wrapped or unwrapped response format across all endpoints
- Update frontend hooks to match actual API response format

### Property Naming Inconsistencies
**Severity:** Medium
**Status:** Open

**Description:**
Inconsistent naming conventions between database columns (snake_case), API responses (camelCase), and frontend interfaces (mixed usage).

**Specific Issues:**
1. Database columns: `open_jobs`, `jobs_change`, etc. (snake_case)
2. API responses: Should be camelCase (`openJobs`, `jobsChange`) due to Serde serialization
3. Frontend interfaces: Mixed usage of snake_case and camelCase properties
4. Causes mapping errors and potential undefined value access

**Examples from DashboardPage.tsx:**
- Interface uses `open_jobs` but API serializes to `openJobs`
- Component accesses `stats?.open_jobs` but should be `stats?.openJobs`

**Fix Required:**
- Standardize on camelCase for all API responses and frontend interfaces
- Update database queries to alias columns appropriately or rely on automatic serialization
- Ensure frontend interfaces match actual API response property names

---