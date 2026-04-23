import { useParams, Link } from 'react-router-dom';
import { useCustomer } from '@/api/hooks/useCustomers';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { Avatar } from '@/components/shared/avatar';
import { Button } from '@/components/ui/button';
import { Edit, Car, FileText, Phone, Mail, MapPin, Calendar } from 'lucide-react';

export default function CustomerDetailPage() {
  const { id } = useParams();
  const { data: customer, isLoading, error } = useCustomer(id!);

  if (isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  if (error || !customer) {
    return (
      <EmptyState
        icon="default"
        title="Customer not found"
        description="The customer you're looking for doesn't exist"
        action={{
          label: 'Back to Customers',
          onClick: () => window.history.back(),
        }}
      />
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={customer.name}
        breadcrumbs={[
          { label: 'Customers', href: '/customers' },
          { label: customer.name },
        ]}
        actions={
          <Button asChild>
            <Link to={`/customers/${customer.id}/edit`}>
              <Edit className="h-4 w-4 mr-1" /> Edit
            </Link>
          </Button>
        }
      />

      <div className="grid lg:grid-cols-3 gap-6">
        {/* Profile Card */}
        <div className="lg:col-span-1">
          <div className="rounded-lg border border-border bg-surface p-6 space-y-4">
            <div className="flex items-center gap-4">
              <Avatar name={customer.name} size="xl" />
              <div>
                <h2 className="text-xl font-bold">{customer.name}</h2>
                <p className="text-sm text-muted-foreground">
                  {customer.type === 'ORGANISATION' ? 'Organisation' : 'Individual'}
                </p>
              </div>
            </div>

            <div className="space-y-3 pt-4 border-t border-border">
              {customer.phone && (
                <div className="flex items-center gap-3 text-sm">
                  <Phone className="h-4 w-4 text-muted-foreground" />
                  <span>{customer.phone}</span>
                </div>
              )}
              {customer.email && (
                <div className="flex items-center gap-3 text-sm">
                  <Mail className="h-4 w-4 text-muted-foreground" />
                  <span>{customer.email}</span>
                </div>
              )}
              {customer.address && (
                <div className="flex items-center gap-3 text-sm">
                  <MapPin className="h-4 w-4 text-muted-foreground" />
                  <span>{customer.address}</span>
                </div>
              )}
              <div className="flex items-center gap-3 text-sm">
                <Calendar className="h-4 w-4 text-muted-foreground" />
                <span>Added {new Date(customer.created_at).toLocaleDateString()}</span>
              </div>
            </div>
          </div>
        </div>

        {/* Tabs Content */}
        <div className="lg:col-span-2 space-y-6">
          {/* Vehicles Tab */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold flex items-center gap-2">
                <Car className="h-5 w-5" /> Vehicles
              </h3>
              <Button variant="outline" size="sm">
                Add Vehicle
              </Button>
            </div>

            {customer.vehicles && customer.vehicles.length > 0 ? (
              <div className="space-y-2">
                {customer.vehicles.map((vehicle) => (
                  <Link
                    key={vehicle.id}
                    to={`/vehicles/${vehicle.id}`}
                    className="flex items-center justify-between p-3 rounded-md border border-border hover:border-accent"
                  >
                    <div>
                      <p className="font-medium">{vehicle.license_plate}</p>
                      <p className="text-sm text-muted-foreground">
                        {vehicle.make} {vehicle.model} {vehicle.year}
                      </p>
                    </div>
                    <span className="text-sm text-muted-foreground">
                      {vehicle.year}
                    </span>
                  </Link>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">
                No vehicles registered
              </p>
            )}
          </div>

          {/* Job History Tab */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold flex items-center gap-2">
                <FileText className="h-5 w-5" /> Job History
              </h3>
            </div>

            {customer.jobs && customer.jobs.length > 0 ? (
              <div className="space-y-2">
                {customer.jobs.map((job) => (
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
                No jobs yet
              </p>
            )}
          </div>

          {/* Invoices Tab */}
          <div className="rounded-lg border border-border bg-surface p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold flex items-center gap-2">
                <FileText className="h-5 w-5" /> Invoices
              </h3>
            </div>

            {customer.invoices && customer.invoices.length > 0 ? (
              <div className="space-y-2">
                {customer.invoices.map((invoice) => (
                  <Link
                    key={invoice.id}
                    to={`/billing/${invoice.id}`}
                    className="flex items-center justify-between p-3 rounded-md border border-border hover:border-accent"
                  >
                    <div>
                      <p className="font-medium">{invoice.invoice_number}</p>
                      <p className="text-sm text-muted-foreground">
                        {new Date(invoice.created_at).toLocaleDateString()}
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="font-medium">Rs. {invoice.amount.toLocaleString()}</p>
                      <p className="text-sm text-muted-foreground">{invoice.status}</p>
                    </div>
                  </Link>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">
                No invoices yet
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}