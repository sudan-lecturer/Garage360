import { useMemo } from 'react';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { useSettingsUsers } from '@/api/hooks/useSettings';

interface RoleDefinition {
  role: string;
  description: string;
  permissions: string[];
}

const roleDefinitions: RoleDefinition[] = [
  {
    role: 'OWNER',
    description: 'Full tenant control including billing, user administration, and approvals.',
    permissions: ['Manage all settings', 'Approve purchases', 'View all reports', 'Manage users'],
  },
  {
    role: 'ADMIN',
    description: 'Operational admin with broad access across workshop modules.',
    permissions: ['Manage jobs and inventory', 'Manage users', 'Approve purchases', 'View reports'],
  },
  {
    role: 'MANAGER',
    description: 'Runs day-to-day operations and supervises service progress.',
    permissions: ['Create and assign jobs', 'Approve job transitions', 'View operational reports'],
  },
  {
    role: 'ACCOUNT_MGR',
    description: 'Handles customer communication and quotation coordination.',
    permissions: ['Manage customers', 'Create quotes', 'Track approvals'],
  },
  {
    role: 'MECHANIC',
    description: 'Executes service work and updates technical progress.',
    permissions: ['Update assigned jobs', 'Submit QA notes', 'View work queue'],
  },
  {
    role: 'CASHIER',
    description: 'Manages invoicing and payment collection tasks.',
    permissions: ['Create invoices', 'Record payments', 'View billing list'],
  },
  {
    role: 'HR_OFFICER',
    description: 'Maintains employee records and payroll operations.',
    permissions: ['Manage HR records', 'Process payroll', 'Approve leave requests'],
  },
];

export default function RoleManagementPage() {
  const usersQuery = useSettingsUsers();

  const roleCounts = useMemo(() => {
    const counts = new Map<string, { total: number; active: number }>();
    roleDefinitions.forEach((definition) => {
      counts.set(definition.role, { total: 0, active: 0 });
    });

    (usersQuery.data ?? []).forEach((user) => {
      const current = counts.get(user.role) ?? { total: 0, active: 0 };
      counts.set(user.role, {
        total: current.total + 1,
        active: current.active + (user.isActive ? 1 : 0),
      });
    });

    return counts;
  }, [usersQuery.data]);

  return (
    <div className="space-y-6">
      <PageHeader
        title="Role Management"
        description="Review role definitions and current user distribution."
        breadcrumbs={[
          { label: 'Settings', href: '/settings' },
          { label: 'Role Management' },
        ]}
      />

      {usersQuery.isLoading && (
        <div className="py-10">
          <LoadingSpinner />
        </div>
      )}

      {!usersQuery.isLoading && (
        <div className="grid gap-4 lg:grid-cols-2">
          {roleDefinitions.map((definition) => {
            const counts = roleCounts.get(definition.role) ?? { total: 0, active: 0 };
            return (
              <section
                key={definition.role}
                className="rounded-lg border border-border bg-surface p-4 space-y-3"
              >
                <div className="flex items-center justify-between">
                  <h2 className="text-lg font-semibold">{definition.role}</h2>
                  <span className="text-xs text-muted-foreground">
                    {counts.active} active / {counts.total} total
                  </span>
                </div>
                <p className="text-sm text-muted-foreground">{definition.description}</p>
                <ul className="list-disc space-y-1 pl-5 text-sm">
                  {definition.permissions.map((permission) => (
                    <li key={`${definition.role}-${permission}`}>{permission}</li>
                  ))}
                </ul>
              </section>
            );
          })}
        </div>
      )}
    </div>
  );
}
