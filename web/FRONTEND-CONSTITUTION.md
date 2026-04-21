# Garage360 — Frontend Constitution

**Version:** 2.1
**Last Updated:** 2026-04-21
**Scope:** React application in `/web`

---

## 1. Architecture Principles

### 1.1 Tech Stack
| Concern | Choice |
|---------|--------|
| Framework | React 19 |
| Language | TypeScript (strict) |
| Build | Vite 6 |
| Routing | React Router v7 |
| Global state | Zustand |
| Server state | TanStack Query v5 |
| Forms | React Hook Form + Zod |
| UI primitives | Radix UI |
| UI pattern | shadcn-style customized components |
| Styling | Tailwind CSS v4 |
| Icons | lucide-react |
| HTTP | Axios |
| Testing | Vitest + Testing Library + MSW |
| PWA | vite-plugin-pwa |

### 1.2 Dependency Baseline
The constitution must match the actual repo. Required packages already in use include:
- `@tanstack/react-query`
- `@radix-ui/*`
- `class-variance-authority`
- `react-hook-form`
- `zod`
- `zustand`
- `tailwindcss`
- `lucide-react`

Motion libraries are optional. Do not make `framer-motion` a required dependency unless it is added to `package.json` and adopted intentionally.

### 1.3 Product Rules
- Mobile-first is mandatory
- Every screen must remain functional at 375px width
- Feature flags control module visibility
- Frontend guards improve UX, but backend remains the source of truth for auth, RBAC, and workflow rules
- All new UI work should align with the shared design tokens in `web/src/styles/globals.css`

---

## 2. Design System

### 2.1 Visual Direction
Garage360 uses a dark industrial interface with semantic tokens, softened contrast, and amber emphasis.

Rules:
- Prefer deep charcoals and slate surfaces over pure black
- Use amber as the primary accent
- Keep borders subtle, not harsh black outlines
- Favor crisp, readable density over decorative flourish
- Use flat-to-subtle elevation, not heavy skeuomorphic shadows

### 2.2 Spacing
Follow the 4px rule for spacing and sizing:
- Spacing should be multiples of 4px
- Standard button heights: 32px, 36px, 40px, 48px, 56px
- Minimum touch target: 44px
- Default page padding should scale from mobile upward, starting around 16px on small screens

### 2.3 Color Tokens
Use the semantic token model defined in [globals.css](/Users/manees/dev/Garage360/web/src/styles/globals.css:1).

Core palette:
```css
--color-background: #0C0D0E;
--color-surface: #141618;
--color-surface-raised: #1C1E21;
--color-foreground: #E8E9EA;
--color-foreground-muted: #8B9095;
--color-primary: #F59E0B;
--color-primary-hover: #D97706;
--color-secondary: #2D3139;
--color-border: #2A2E35;
--color-border-hover: #3D434B;
--color-success: #22C55E;
--color-warning: #EAB308;
--color-destructive: #EF4444;
--color-info: #3B82F6;
```

Rules:
- Do not introduce raw hex values in components when a semantic token exists
- Do not switch to a separate light brutalist palette inside the same app
- New status styles should include foreground and muted variants when needed

### 2.4 Typography
Use the typography tokens from `globals.css`.

```css
--font-sans: "Inter", system-ui, -apple-system, sans-serif;
--font-heading: "Inter", system-ui, -apple-system, sans-serif;
--font-mono: "JetBrains Mono", ui-monospace, monospace;
```

Rules:
- Headings use tighter tracking and stronger weight
- Body text prioritizes legibility and compact scanning
- Use monospace for IDs, codes, and system-like values

### 2.5 Radius, Shadows, and Effects
- Use the defined radius scale instead of mixing rounded and square styles arbitrarily
- Prefer subtle shadows from the token set
- Glass and gradient utilities may be used sparingly for emphasis, not as a default for every panel

### 2.6 Motion
- Motion is optional and should be purposeful
- Prefer CSS/Tailwind transitions for common interactions
- If `framer-motion` is added later, update this constitution and standardize where it should be used
- Do not require animation wrappers for every component

---

## 3. Component Rules

### 3.1 Architecture
Use an atomic-ish structure:

```text
Atoms: Button, Input, Label, Icon
Molecules: FormField, InputGroup, Card
Organisms: CustomerForm, JobCard, DataTable
Templates: ListPage, DetailPage, FormPage
Pages: Route-level screens
```

### 3.2 File Structure
Frontend code should follow this shape:

```text
web/src/
├── api/
│   ├── client.ts
│   └── hooks/
├── components/
│   ├── ui/
│   └── shared/
├── modules/
├── layouts/
├── store/
├── hooks/
├── lib/
├── i18n/
├── styles/
└── App.tsx
```

### 3.3 Naming Conventions
| Type | Convention | Example |
|------|------------|---------|
| Components | PascalCase | `CustomerForm` |
| Hooks | camelCase with `use` prefix | `useCustomers` |
| Files | kebab-case | `customer-form.tsx` |
| Types | PascalCase | `CustomerFormData` |
| Stores | kebab-case file, named export hook | `auth.ts`, `useAuthStore` |

### 3.4 Component Standards
- One component should have one clear responsibility
- Pages may use default export; reusable components should use named exports
- All props must be explicitly typed
- Prefer Tailwind utilities and shared variants over inline styles
- Handle loading, empty, and error states intentionally
- Keep accessibility requirements built into primitives whenever possible

### 3.5 Button Standard
Buttons should follow the existing variant model in [button.tsx](/Users/manees/dev/Garage360/web/src/components/ui/button.tsx:1).

Supported variants:
- `primary`
- `secondary`
- `ghost`
- `outline`
- `destructive`
- `success`
- `link`

Supported sizes:
- `xs`
- `sm`
- `md`
- `lg`
- `xl`
- `icon`
- `icon-sm`
- `icon-lg`

Rules:
- Use `isLoading` for loading buttons when possible
- Left and right icons should be decorative unless they convey unique meaning
- Focus states must remain visible

### 3.6 Input Standard
Inputs should follow the shared primitive in [input.tsx](/Users/manees/dev/Garage360/web/src/components/ui/input.tsx:1).

Rules:
- Inputs use semantic border, ring, and background tokens
- Error state should be applied via class names or wrapper components, not bespoke inline CSS
- Labels should be present for all form controls

---

## 4. Code Conventions

### 4.1 Imports
Use a consistent import order:

```typescript
// 1. React and framework libraries
import { useState } from 'react';

// 2. Third-party packages
import { useMutation } from '@tanstack/react-query';
import { z } from 'zod';

// 3. Internal UI components
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

// 4. Internal hooks, stores, API, utilities
import { useAuthStore } from '@/store/auth';
import api from '@/api/client';
```

### 4.2 Example Component Shape
```typescript
import { useState } from 'react';
import { Button } from '@/components/ui/button';

interface CustomerFormProps {
  customerId?: string;
  onSuccess: () => void;
}

export function CustomerForm({ customerId, onSuccess }: CustomerFormProps) {
  const [isSaving, setIsSaving] = useState(false);

  async function handleSubmit() {
    setIsSaving(true);
    try {
      onSuccess();
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <Button isLoading={isSaving} onClick={handleSubmit}>
      Save
    </Button>
  );
}
```

### 4.3 State Management
Use Zustand for durable app-level client state and TanStack Query for server state.

Rules:
- Auth/session state belongs in Zustand
- Remote data fetching, caching, and invalidation belong in TanStack Query
- Do not mirror server collections into Zustand without a strong reason

### 4.4 Forms
- Use React Hook Form with Zod validation
- Keep schemas close to the form or feature module that owns them
- Show validation feedback inline
- Prefer typed submit handlers and inferred schema types

### 4.5 API Layer
- Use the shared Axios client
- Keep API calls in `web/src/api`
- Use typed request and response shapes
- Auth header injection belongs in the client layer, not repeated in each call site

---

## 5. Layout and Responsive Rules

### 5.1 Breakpoints
```css
--breakpoint-sm: 640px;
--breakpoint-md: 768px;
--breakpoint-lg: 1024px;
--breakpoint-xl: 1280px;
```

### 5.2 Mobile-First Rules
- Build for phone screens first, then enhance upward
- Forms should remain usable with full-width fields on small screens
- Important actions should stay reachable without precision clicking
- Use sticky headers, sticky actions, or bottom action areas when that materially improves mobile usability

### 5.3 Page Templates
Common page patterns:
- List pages: page header, filters, table/list content, pagination
- Detail pages: breadcrumb, summary header, detail cards, related records
- Form pages: breadcrumb, grouped form sections, clear cancel/save actions

---

## 6. Testing Guidelines

### 6.1 Tooling
Use the current frontend test stack:
- Vitest
- Testing Library
- `@testing-library/user-event`
- MSW for mocked network behavior

### 6.2 Test Priorities
1. User interactions
2. Form validation
3. Loading, empty, and error states
4. Auth and routing behavior
5. Accessibility-sensitive interactions

### 6.3 Test Style
- Test user-observable behavior, not implementation details
- Prefer accessible queries like `getByRole`
- Mock HTTP at the boundary instead of mocking every hook internals

---

## 7. Performance Guidance

### 7.1 Defaults
- Route-level code splitting is encouraged
- Use lazy loading for large screens or heavy modules
- Use list virtualization for genuinely large datasets
- Prefetch likely next-route data when it improves navigation meaningfully

### 7.2 React Guidance
- Do not add `useMemo` or `useCallback` by default
- Optimize only when there is a measured or obvious recomputation problem
- Prefer straightforward React code over premature memoization

### 7.3 Bundle Discipline
- Keep the initial bundle lean
- Remove dead dependencies and unused exports
- Prefer shared primitives over near-duplicate components

---

## 8. Accessibility

### 8.1 Baseline Requirements
- All interactive flows must work with keyboard navigation
- Focus indicators must remain visible
- Text and interactive contrast should meet WCAG AA expectations
- Touch targets should meet the 44px minimum where applicable
- Error messages should be announced appropriately

### 8.2 ARIA Use
- Use native semantics first
- Add ARIA only where it improves clarity
- Icon-only controls must have accessible labels

Example:
```typescript
<Button aria-label="Edit customer">
  <PencilIcon />
</Button>
```

---

## 9. Internationalization

### 9.1 Rules
- User-facing strings should be translatable
- Translation keys should be namespaced by feature
- Avoid hardcoding strings deep inside reusable primitives unless intentionally shared

### 9.2 Key Style
```json
{
  "customer.list.title": "Customers",
  "customer.list.search": "Search customers...",
  "customer.form.save": "Save Customer"
}
```

---

## 10. Build and Quality Gates

### 10.1 Commands
From `/web`:

```bash
npm run dev
npm run build
npm run lint
npm run typecheck
npm test
npm run test:ui
```

### 10.2 Environment Variables
```env
VITE_API_URL=http://localhost:8080
VITE_APP_NAME=Garage360
```

### 10.3 Definition of Done
Frontend work is not done until:
- It matches the shared design tokens
- It works on mobile
- It has sensible loading and error states
- It respects accessibility basics
- It passes relevant tests, lint, and type checks

---

## 11. Source of Truth

When guidance conflicts, use this order:
1. `docs/MASTER-PLAN.md`
2. `web/src/styles/globals.css`
3. `web/FRONTEND-CONSTITUTION.md`
4. Current implementation
