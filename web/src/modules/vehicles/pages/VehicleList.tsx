import { useState } from 'react';
import { Link } from 'react-router-dom';
import { useVehicles } from '@/api/hooks/useVehicles';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { SearchInput } from '@/components/shared/search-input';
import { Button } from '@/components/ui/button';
import { Plus, Car, ChevronRight } from 'lucide-react';

export default function VehicleListPage() {
  const [search, setSearch] = useState('');
  const [makeFilter, setMakeFilter] = useState('');
  
  const { data, isLoading, error } = useVehicles({
    search: search || undefined,
    page: 1,
    limit: 20,
  });

  // Extract unique makes for filter
  const makes = data?.data 
    ? [...new Set(data.data.map(v => v.make).filter(Boolean))]
    : [];

  return (
    <div className="space-y-4">
      <PageHeader
        title="Vehicles"
        description="Manage your vehicle database"
        actions={
          <Button asChild>
            <Link to="/vehicles/new">
              <Plus className="h-4 w-4 mr-1" /> Add Vehicle
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
            placeholder="Search by license plate..."
          />
        </div>
        {makes.length > 0 && (
          <select
            value={makeFilter}
            onChange={(e) => setMakeFilter(e.target.value)}
            className="h-10 rounded-md border border-input bg-background px-3 text-sm"
          >
            <option value="">All Makes</option>
            {makes.map((make) => (
              <option key={make} value={make}>{make}</option>
            ))}
          </select>
        )}
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
          title="Error loading vehicles"
          description="Please try again later"
        />
      )}

      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState
          icon="search"
          title="No vehicles found"
          description={search ? 'Try adjusting your search' : 'Add your first vehicle to get started'}
          action={{
            label: 'Add Vehicle',
            onClick: () => {},
          }}
        />
      )}

      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-lg border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">License Plate</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Make & Model</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Year</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Customer</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Last Service</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map((vehicle) => (
                <tr
                  key={vehicle.id}
                  className="border-b border-border last:border-0 hover:bg-surface-raised"
                >
                  <td className="p-3">
                    <Link
                      to={`/vehicles/${vehicle.id}`}
                      className="flex items-center gap-2 hover:text-accent"
                    >
                      <Car className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">{vehicle.license_plate}</span>
                    </Link>
                  </td>
                  <td className="p-3 text-sm">
                    {vehicle.make} {vehicle.model}
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">{vehicle.year || '-'}</td>
                  <td className="p-3 text-sm text-muted-foreground">
                    <Link to={`/customers/${vehicle.customer_id}`} className="hover:text-accent">
                      {vehicle.customer_name}
                    </Link>
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">-</td>
                  <td className="p-3 text-right">
                    <Link
                      to={`/vehicles/${vehicle.id}`}
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
      )}

      {/* Pagination */}
      {data && data.total > data.limit && (
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            Showing {data.data.length} of {data.total} vehicles
          </p>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" disabled={data.page === 1}>
              Previous
            </Button>
            <Button variant="outline" size="sm" disabled={data.page * data.limit >= data.total}>
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}