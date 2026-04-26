import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useCustomers } from '@/api/hooks/useCustomers';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { SearchInput } from '@/components/shared/search-input';
import { Button } from '@/components/ui/button';
import { Plus, User, ChevronRight } from 'lucide-react';

export default function CustomerListPage() {
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<'INDIVIDUAL' | 'ORGANIZATION' | 'BOTH'>('BOTH');
  
  const { data, isLoading, error } = useCustomers({
    search: search || undefined,
    customer_type: typeFilter,
    limit: 20,
  });

  return (
    <div className="space-y-4">
      <PageHeader
        title="Customers"
        description="Manage your customer database"
        actions={
          <Button asChild>
            <Link to="/customers/new">
              <Plus className="h-4 w-4 mr-1" /> Add Customer
            </Link>
          </Button>
        }
      />

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="w-full sm:w-64">
          <SearchInput
            value={search}
            onChange={setSearch}
            placeholder="Search name, plate, phone, email, make/model..."
          />
        </div>
        <div className="flex items-center gap-4 rounded-sm border border-input bg-background px-3 py-2 text-sm">
          <label className="inline-flex items-center gap-2">
            <input
              type="radio"
              name="customer_type"
              value="BOTH"
              checked={typeFilter === 'BOTH'}
              onChange={() => setTypeFilter('BOTH')}
            />
            Both
          </label>
          <label className="inline-flex items-center gap-2">
            <input
              type="radio"
              name="customer_type"
              value="INDIVIDUAL"
              checked={typeFilter === 'INDIVIDUAL'}
              onChange={() => setTypeFilter('INDIVIDUAL')}
            />
            Individual
          </label>
          <label className="inline-flex items-center gap-2">
            <input
              type="radio"
              name="customer_type"
              value="ORGANIZATION"
              checked={typeFilter === 'ORGANIZATION'}
              onChange={() => setTypeFilter('ORGANIZATION')}
            />
            Organizational
          </label>
        </div>
      </div>

      {/* Table */}
      {isLoading && (
        <div className="py-12">
          <LoadingSpinner />
        </div>
      )}

      {error && (
        <EmptyState
          icon="default"
          title="Error loading customers"
          description="Please try again later"
        />
      )}

      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState
          icon="search"
          title="No customers found"
          description={search ? 'Try adjusting your search' : 'Add your first customer to get started'}
          action={{
            label: 'Add Customer',
            onClick: () => navigate('/customers/new'),
          }}
        />
      )}

      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-lg border border-border bg-surface">
          <div className="space-y-3 p-3 sm:hidden">
            {data.data.map((customer) => (
              <Link
                key={customer.id}
                to={`/customers/${customer.id}`}
                className="block rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <p className="font-medium">{customer.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {customer.type === 'ORGANIZATION' ? 'Org' : 'Individual'}
                    </p>
                  </div>
                  <ChevronRight className="h-4 w-4 text-muted-foreground" />
                </div>
                <p className="mt-2 text-xs text-muted-foreground">{customer.phone || '-'}</p>
                <p className="text-xs text-muted-foreground">{customer.email || '-'}</p>
              </Link>
            ))}
          </div>
          <div className="hidden overflow-x-auto sm:block">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Name</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Type</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Phone</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Email</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Vehicles</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map((customer) => (
                <tr
                  key={customer.id}
                  className="border-b border-border last:border-0 hover:bg-surface-raised"
                >
                  <td className="p-3">
                    <Link
                      to={`/customers/${customer.id}`}
                      className="flex items-center gap-2 hover:text-accent"
                    >
                      <User className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">{customer.name}</span>
                    </Link>
                  </td>
                  <td className="p-3 text-sm">
                    <span className={customer.type === 'ORGANIZATION' ? 'text-accent' : 'text-muted-foreground'}>
                      {customer.type === 'ORGANIZATION' ? 'Org' : 'Individual'}
                    </span>
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">{customer.phone || '-'}</td>
                  <td className="p-3 text-sm text-muted-foreground">{customer.email || '-'}</td>
                  <td className="p-3 text-sm text-muted-foreground">-</td>
                  <td className="p-3 text-right">
                    <Link
                      to={`/customers/${customer.id}`}
                      className="inline-flex items-center text-accent hover:underline"
                    >
                      View <ChevronRight className="h-3 w-3" />
                    </Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          </div>
        </div>
      )}

      {/* Pagination */}
      {data && data.total > data.limit && (
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            Showing {data.data.length} of {data.total} customers
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={data.page === 1}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={data.page * data.limit >= data.total}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}