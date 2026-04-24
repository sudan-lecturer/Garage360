import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import api from '@/api/client';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { SearchInput } from '@/components/shared/search-input';
import { StatusBadge } from '@/components/shared/status-badge';
import { Button } from '@/components/ui/button';
import { Plus, Building, ChevronRight, Edit, Trash2 } from 'lucide-react';

interface Tenant {
  id: string;
  name: string;
  slug: string;
  database_name: string;
  is_active: boolean;
  created_at: string;
}

function useTenants() {
  return useQuery({
    queryKey: ['tenants'],
    queryFn: async () => {
      const response = await api.get<{ data: Tenant[] }>('/control/v1/tenants');
      return response.data;
    },
  });
}

export default function SuperAdminTenantsPage() {
  const [search, setSearch] = useState('');
  const { data, isLoading, error } = useTenants();

  return (
    <div className="space-y-4">
      <PageHeader
        title="Tenants"
        description="Manage workshop tenants"
        actions={
          <Button>
            <Plus className="h-4 w-4 mr-1" /> Provision Tenant
          </Button>
        }
      />

      <div className="w-full sm:w-64">
        <SearchInput value={search} onChange={setSearch} placeholder="Search tenants..." />
      </div>

      {isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {error && <EmptyState icon="default" title="Error loading tenants" description="Please try again later" />}
      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState icon="default" title="No tenants yet" description="Provision your first tenant" action={{ label: 'Provision Tenant', onClick: () => {} }} />
      )}

      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-lg border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Tenant Name</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Slug</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Database</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Created</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map(tenant => (
                <tr key={tenant.id} className="border-b border-border last:border-0 hover:bg-surface-raised">
                  <td className="p-3">
                    <div className="flex items-center gap-2">
                      <Building className="h-4 w-4 text-accent" />
                      <span className="font-medium">{tenant.name}</span>
                    </div>
                  </td>
                  <td className="p-3 text-sm font-mono">{tenant.slug}</td>
                  <td className="p-3 text-sm font-mono text-muted-foreground">{tenant.database_name}</td>
                  <td className="p-3">
                    <StatusBadge variant={tenant.is_active ? 'active' : 'inactive'} />
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    {new Date(tenant.created_at).toLocaleDateString()}
                  </td>
                  <td className="p-3 text-right">
                    <div className="flex gap-2 justify-end">
                      <Button variant="ghost" size="sm" title="View details">
                        <ChevronRight className="h-4 w-4" />
                      </Button>
                      <Button variant="ghost" size="sm" title="Edit">
                        <Edit className="h-4 w-4" />
                      </Button>
                      <Button variant="ghost" size="sm" title="Delete">
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}