import { useState } from 'react';
import { Link } from 'react-router-dom';
import { useJobs } from '@/api/hooks/useJobs';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { SearchInput } from '@/components/shared/search-input';
import { StatusBadge } from '@/components/shared/status-badge';
import { Button } from '@/components/ui/button';
import { Plus, LayoutGrid, List, ChevronRight } from 'lucide-react';

const statusColumns = [
  { status: 'INTAKE', label: 'Intake', color: 'border-l-amber-500' },
  { status: 'AUDIT', label: 'Audit', color: 'border-l-blue-500' },
  { status: 'QUOTE', label: 'Quote', color: 'border-l-yellow-500' },
  { status: 'APPROVAL', label: 'Approval', color: 'border-l-purple-500' },
  { status: 'IN_SERVICE', label: 'In Service', color: 'border-l-cyan-500' },
  { status: 'QA', label: 'QA', color: 'border-l-indigo-500' },
  { status: 'BILLING', label: 'Billing', color: 'border-l-green-500' },
  { status: 'COMPLETED', label: 'Completed', color: 'border-l-emerald-500' },
];

export default function JobListPage() {
  const [search, setSearch] = useState('');
  const [view, setView] = useState<'kanban' | 'table'>('kanban');
  const [statusFilter, setStatusFilter] = useState('');
  
  const { data, isLoading, error } = useJobs({
    search: search || undefined,
    status: statusFilter as any || undefined,
    limit: 50,
  });

  return (
    <div className="space-y-4">
      <PageHeader
        title="Jobs"
        description="Manage job cards"
        actions={
          <Button asChild>
            <Link to="/jobs/new">
              <Plus className="h-4 w-4 mr-1" /> New Job
            </Link>
          </Button>
        }
      />

      {/* Filters & View Toggle */}
      <div className="flex flex-col sm:flex-row gap-3 items-start sm:items-center justify-between">
        <div className="flex flex-col sm:flex-row gap-3 w-full sm:w-auto">
          <div className="w-full sm:w-64">
            <SearchInput
              value={search}
              onChange={setSearch}
              placeholder="Search by job # or customer..."
            />
          </div>
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="h-10 rounded-md border border-input bg-background px-3 text-sm w-full sm:w-auto"
          >
            <option value="">All Statuses</option>
            {statusColumns.map(col => (
              <option key={col.status} value={col.status}>{col.label}</option>
            ))}
          </select>
        </div>
        
        <div className="flex gap-1 border border-border rounded-md p-1">
          <button
            onClick={() => setView('kanban')}
            className={`p-2 rounded ${view === 'kanban' ? 'bg-surface-raised' : ''}`}
            title="Kanban view"
          >
            <LayoutGrid className="h-4 w-4" />
          </button>
          <button
            onClick={() => setView('table')}
            className={`p-2 rounded ${view === 'table' ? 'bg-surface-raised' : ''}`}
            title="Table view"
          >
            <List className="h-4 w-4" />
          </button>
        </div>
      </div>

      {/* Loading/Error States */}
      {isLoading && (
        <div className="py-12">
          <LoadingSpinner />
        </div>
      )}

      {error && (
        <EmptyState
          icon="default"
          title="Error loading jobs"
          description="Please try again later"
        />
      )}

      {/* Kanban View */}
      {!isLoading && !error && view === 'kanban' && (
        <div className="flex gap-4 overflow-x-auto pb-4">
          {statusColumns.map(column => {
            const jobsInColumn = data?.data?.filter(j => j.status === column.status) || [];
            return (
              <div key={column.status} className="flex-shrink-0 w-72">
                <div className={`border-t-4 ${column.color} rounded-t-md bg-surface-raised p-3`}>
                  <h3 className="font-medium text-sm">
                    {column.label}
                    <span className="ml-2 text-muted-foreground">({jobsInColumn.length})</span>
                  </h3>
                </div>
                <div className="border border-t-0 border-border bg-surface-raised/50 p-2 min-h-[200px] space-y-2">
                  {jobsInColumn.length === 0 ? (
                    <p className="text-sm text-muted-foreground text-center py-4">
                      No jobs
                    </p>
                  ) : (
                    jobsInColumn.map(job => (
                      <Link
                        key={job.id}
                        to={`/jobs/${job.id}`}
                        className="block p-3 rounded-md border border-border bg-surface hover:border-accent transition-colors"
                      >
                        <div className="flex items-center justify-between mb-2">
                          <span className="font-medium text-sm">{job.job_number}</span>
                          <StatusBadge variant={job.status.toLowerCase().replace('_', '_') as any} />
                        </div>
                        <p className="text-sm text-muted-foreground truncate">
                          {job.customer_name}
                        </p>
                        <p className="text-xs text-muted-foreground truncate">
                          {job.vehicle_plate} - {job.vehicle_make} {job.vehicle_model}
                        </p>
                      </Link>
                    ))
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Table View */}
      {!isLoading && !error && view === 'table' && (!data?.data || data.data.length === 0) && (
        <EmptyState
          icon="search"
          title="No jobs found"
          description={search ? 'Try adjusting your search' : 'No jobs yet'}
          action={{
            label: 'Create Job',
            onClick: () => {},
          }}
        />
      )}

      {!isLoading && !error && view === 'table' && data?.data && data.data.length > 0 && (
        <div className="rounded-lg border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Job #</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Customer</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Vehicle</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Mechanic</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Bay</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Created</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map((job) => (
                <tr
                  key={job.id}
                  className="border-b border-border last:border-0 hover:bg-surface-raised"
                >
                  <td className="p-3">
                    <Link
                      to={`/jobs/${job.id}`}
                      className="font-medium hover:text-accent"
                    >
                      {job.job_number}
                    </Link>
                  </td>
                  <td className="p-3">
                    <StatusBadge variant={job.status.toLowerCase().replace('_', '_') as any} />
                  </td>
                  <td className="p-3 text-sm">
                    <Link to={`/customers/${job.customer_id}`} className="hover:text-accent">
                      {job.customer_name}
                    </Link>
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    <Link to={`/vehicles/${job.vehicle_id}`} className="hover:text-accent">
                      {job.vehicle_plate}
                    </Link>
                    <span className="block text-xs">{job.vehicle_make} {job.vehicle_model}</span>
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    {job.mechanic_name || '-'}
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    {job.bay_name || '-'}
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    {new Date(job.created_at).toLocaleDateString()}
                  </td>
                  <td className="p-3 text-right">
                    <Link
                      to={`/jobs/${job.id}`}
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
            Showing {data.data.length} of {data.total} jobs
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