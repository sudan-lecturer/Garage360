import { useParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { useEmployeeDetail } from '@/api/hooks/useHR';

export default function EmployeeDetailPage() {
  const { id } = useParams();
  const employeeQuery = useEmployeeDetail(id);

  if (employeeQuery.isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  if (!employeeQuery.data || employeeQuery.error) {
    return <EmptyState title="Employee not found" description="The employee could not be loaded." />;
  }

  const employee = employeeQuery.data;

  return (
    <div className="space-y-6">
      <PageHeader
        title={`${employee.firstName} ${employee.lastName}`}
        description="Employee profile and employment details."
        breadcrumbs={[
          { label: 'Employees', href: '/hr/employees' },
          { label: `${employee.firstName} ${employee.lastName}` },
        ]}
      />

      <section className="rounded-sm border border-border bg-surface p-4">
        <dl className="grid gap-3 sm:grid-cols-2">
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Employee No</dt>
            <dd className="text-sm">{employee.employeeNo}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Employment Type</dt>
            <dd className="text-sm">{employee.employmentType}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Email</dt>
            <dd className="text-sm">{employee.email || '-'}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Phone</dt>
            <dd className="text-sm">{employee.phone}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Department</dt>
            <dd className="text-sm">{employee.department || '-'}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Designation</dt>
            <dd className="text-sm">{employee.designation || '-'}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Join Date</dt>
            <dd className="text-sm">{employee.joinDate || '-'}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wide text-muted-foreground">Salary</dt>
            <dd className="text-sm">{employee.salary || '-'}</dd>
          </div>
        </dl>
      </section>
    </div>
  );
}
