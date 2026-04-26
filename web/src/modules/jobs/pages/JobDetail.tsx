import { useParams, Link } from 'react-router-dom';
import { useJob } from '@/api/hooks/useJobs';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { StatusBadge } from '@/components/shared/status-badge';
import { Avatar } from '@/components/shared/avatar';
import { Button } from '@/components/ui/button';
import { 
  Wrench, Car, User, FileText, ClipboardCheck, 
  Clock, Edit, CheckCircle
} from 'lucide-react';

export default function JobDetailPage() {
  const { id } = useParams();
  const { data: job, isLoading, error } = useJob(id!);

  if (isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  if (error || !job) {
    return (
      <EmptyState
        icon="default"
        title="Job not found"
        description="The job you're looking for doesn't exist"
        action={{
          label: 'Back to Jobs',
          onClick: () => window.history.back(),
        }}
      />
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={job.job_number}
        breadcrumbs={[
          { label: 'Jobs', href: '/jobs' },
          { label: job.job_number },
        ]}
        actions={
          <div className="flex flex-wrap gap-2">
            <Button variant="secondary" asChild>
              <Link to={`/jobs/${job.id}/edit`}>
                <Edit className="h-4 w-4 mr-1" /> Edit
              </Link>
            </Button>
            {job.status === 'QUOTE' && (
              <Button asChild>
                <Link to={`/jobs/${job.id}/approve`}>
                  <CheckCircle className="h-4 w-4 mr-1" /> Approve
                </Link>
              </Button>
            )}
            {job.status === 'IN_SERVICE' && (
              <Button asChild>
                <Link to={`/jobs/${job.id}/qa`}>
                  <ClipboardCheck className="h-4 w-4 mr-1" /> Submit QA
                </Link>
              </Button>
            )}
          </div>
        }
      />

      <div className="grid lg:grid-cols-3 gap-6">
        {/* Main Info */}
        <div className="lg:col-span-2 space-y-6">
          {/* Overview Tab */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <h3 className="text-lg font-semibold mb-4">Overview</h3>
            
            <div className="grid sm:grid-cols-2 gap-4">
              <div>
                <p className="text-sm text-muted-foreground">Status</p>
                <StatusBadge variant={job.status.toLowerCase().replace('_', '_') as any} size="lg" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Created</p>
                <p className="font-medium">{new Date(job.created_at).toLocaleString()}</p>
              </div>
              {job.complaint && (
                <div className="sm:col-span-2">
                  <p className="text-sm text-muted-foreground">Complaint</p>
                  <p className="font-medium">{job.complaint}</p>
                </div>
              )}
            </div>
          </div>

          {/* Items Tab */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold flex items-center gap-2">
                <FileText className="h-5 w-5" /> Line Items
              </h3>
              {(job.status === 'QUOTE' || job.status === 'AUDIT') && (
                <Button variant="outline" size="sm" asChild>
                  <Link to={`/jobs/${job.id}/approve`}>Add Item</Link>
                </Button>
              )}
            </div>
            
            {job.items && job.items.length > 0 ? (
              <div className="space-y-2">
                {job.items.map((item, idx) => (
                  <div key={idx} className="flex items-center justify-between p-3 rounded-md border border-border">
                    <div>
                      <p className="font-medium">{item.description}</p>
                      <p className="text-sm text-muted-foreground">
                        {item.type} • Qty: {item.quantity} × Rs. {item.unit_price}
                      </p>
                    </div>
                    <p className="font-medium">Rs. {item.total.toLocaleString()}</p>
                  </div>
                ))}
                <div className="flex justify-between pt-3 border-t border-border">
                  <p className="font-medium">Total</p>
                  <p className="font-bold text-lg">Rs. {job.items.reduce((sum, i) => sum + i.total, 0).toLocaleString()}</p>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">
                No line items yet
              </p>
            )}
          </div>

          {/* Timeline Tab */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Clock className="h-5 w-5" /> Activity Timeline
            </h3>
            
            {job.activities && job.activities.length > 0 ? (
              <div className="space-y-4">
                {job.activities.map((activity) => (
                  <div key={activity.id} className="flex gap-3">
                    <div className="flex flex-col items-center">
                      <div className="w-2 h-2 rounded-full bg-accent" />
                      <div className="flex-1 w-px bg-border" />
                    </div>
                    <div className="flex-1 pb-4">
                      <p className="font-medium">{activity.action}</p>
                      <p className="text-sm text-muted-foreground">{activity.description}</p>
                      <p className="text-xs text-muted-foreground mt-1">
                        {activity.performed_by} ({activity.performed_by_role}) • {new Date(activity.created_at).toLocaleString()}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">
                No activity yet
              </p>
            )}
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Customer Card */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <User className="h-5 w-5" /> Customer
            </h3>
            <Link
              to={`/customers/${job.customer_id}`}
              className="flex items-center gap-3 p-3 rounded-md border border-border hover:border-accent"
            >
              <Avatar name={job.customer_name} />
              <div>
                <p className="font-medium">{job.customer_name}</p>
                <p className="text-sm text-muted-foreground">View profile</p>
              </div>
            </Link>
          </div>

          {/* Vehicle Card */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Car className="h-5 w-5" /> Vehicle
            </h3>
            <Link
              to={`/vehicles/${job.vehicle_id}`}
              className="flex items-center gap-3 p-3 rounded-md border border-border hover:border-accent"
            >
              <div className="p-2 rounded-full bg-surface-raised">
                <Car className="h-5 w-5 text-accent" />
              </div>
              <div>
                <p className="font-medium">{job.vehicle_plate}</p>
                <p className="text-sm text-muted-foreground">
                  {job.vehicle_make} {job.vehicle_model}
                </p>
              </div>
            </Link>
          </div>

          {/* Assignment Card */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Wrench className="h-5 w-5" /> Assignment
            </h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Mechanic</span>
                <span className="font-medium">{job.mechanic_name || '-'}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Bay</span>
                <span className="font-medium">{job.bay_name || '-'}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}