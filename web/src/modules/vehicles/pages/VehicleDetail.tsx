import { useParams, Link } from 'react-router-dom';
import { useVehicle } from '@/api/hooks/useVehicles';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { Button } from '@/components/ui/button';
import { Edit, Car, User, FileText } from 'lucide-react';

export default function VehicleDetailPage() {
  const { id } = useParams();
  const { data: vehicle, isLoading, error } = useVehicle(id!);

  if (isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  if (error || !vehicle) {
    return (
      <EmptyState
        icon="default"
        title="Vehicle not found"
        description="The vehicle you're looking for doesn't exist"
        action={{
          label: 'Back to Vehicles',
          onClick: () => window.history.back(),
        }}
      />
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={vehicle.license_plate}
        breadcrumbs={[
          { label: 'Vehicles', href: '/vehicles' },
          { label: vehicle.license_plate },
        ]}
        actions={
          <Button asChild>
            <Link to={`/vehicles/${vehicle.id}/edit`}>
              <Edit className="h-4 w-4 mr-1" /> Edit
            </Link>
          </Button>
        }
      />

      <div className="grid lg:grid-cols-3 gap-6">
        {/* Vehicle Info Card */}
        <div className="lg:col-span-1">
          <div className="rounded-lg border border-border bg-surface p-6 space-y-4">
            <div className="flex items-center gap-4">
              <div className="p-3 rounded-full bg-surface-raised">
                <Car className="h-6 w-6 text-accent" />
              </div>
              <div>
                <h2 className="text-xl font-bold">{vehicle.license_plate}</h2>
                <p className="text-sm text-muted-foreground">
                  {vehicle.make} {vehicle.model}
                </p>
              </div>
            </div>

            <div className="space-y-3 pt-4 border-t border-border">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Make</span>
                <span className="font-medium">{vehicle.make}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Model</span>
                <span className="font-medium">{vehicle.model}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Year</span>
                <span className="font-medium">{vehicle.year || '-'}</span>
              </div>
              {vehicle.color && (
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Color</span>
                  <span className="font-medium">{vehicle.color}</span>
                </div>
              )}
              {vehicle.vin && (
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">VIN</span>
                  <span className="font-medium text-xs">{vehicle.vin}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Customer Card */}
        <div className="lg:col-span-2 space-y-6">
          <div className="rounded-lg border border-border bg-surface p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <User className="h-5 w-5" /> Owner
            </h3>
            <Link
              to={`/customers/${vehicle.customer_id}`}
              className="flex items-center justify-between p-3 rounded-md border border-border hover:border-accent"
            >
              <div className="flex items-center gap-3">
                <User className="h-5 w-5 text-muted-foreground" />
                <div>
                  <p className="font-medium">{vehicle.customer_name}</p>
                  <p className="text-sm text-muted-foreground">Customer</p>
                </div>
              </div>
            </Link>
          </div>

          {/* Job History */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <FileText className="h-5 w-5" /> Service History
            </h3>
            {vehicle.jobs && vehicle.jobs.length > 0 ? (
              <div className="space-y-2">
                {vehicle.jobs.map((job) => (
                  <Link
                    key={job.id}
                    to={`/jobs/${job.id}`}
                    className="flex items-center justify-between p-3 rounded-md border border-border hover:border-accent"
                  >
                    <div>
                      <p className="font-medium">{job.job_number}</p>
                      <p className="text-sm text-muted-foreground">
                        {new Date(job.created_at).toLocaleDateString()}
                      </p>
                    </div>
                    <span className="text-sm px-2 py-1 rounded-full bg-surface-raised">
                      {job.status}
                    </span>
                  </Link>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">
                No service history
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}