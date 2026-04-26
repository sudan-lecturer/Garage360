import { useParams, Link } from 'react-router-dom';
import { useCustomer } from '@/api/hooks/useCustomers';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { Avatar } from '@/components/shared/avatar';
import { Button } from '@/components/ui/button';
import {
  Edit,
  Car,
  FileText,
  Phone,
  Mail,
  MapPin,
  Calendar,
  ChevronRight,
  ShieldCheck,
  Wallet,
  AlertTriangle,
} from 'lucide-react';

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

  const fleetCount = customer.vehicles?.length ?? 0;
  const activeTickets = customer.jobs?.filter((job) => job.status.toLowerCase() !== 'completed').length ?? 0;
  const loyaltyPoints = Math.round(Number(customer.financialSnapshot?.totalSpend ?? '0') / 5);
  const totalLtv = customer.financialSnapshot?.totalSpend ?? '0.00';
  const lastVisit = customer.financialSnapshot?.lastInvoiceAt ?? customer.created_at;
  const nextServiceHint = customer.serviceChronicle?.[0]?.occurredAt ?? customer.created_at;
  const leadNote = customer.serviceChronicle?.[0]?.summary;

  return (
    <div className="space-y-6">
      <PageHeader
        title={customer.name}
        description="Customer command profile"
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

      <section className="grid gap-3 md:grid-cols-3 xl:grid-cols-6">
        <div className="rounded-sm border border-border bg-surface p-3 md:col-span-2 xl:col-span-1">
          <p className="text-[10px] uppercase tracking-[0.14em] text-muted-foreground">UUID</p>
          <p className="mt-1 text-xs font-semibold uppercase tracking-[0.08em] text-foreground/90">
            {customer.id.slice(0, 8)}-{customer.id.slice(9, 13)}
          </p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Tier</p>
          <p className="mt-1 text-sm font-semibold uppercase tracking-wide">{customer.tier || 'BRONZE'}</p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Active Tickets</p>
          <p className="mt-1 text-sm font-semibold">{activeTickets.toString().padStart(2, '0')}</p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Loyalty Credits</p>
          <p className="mt-1 text-sm font-semibold">{loyaltyPoints.toLocaleString()} PTS</p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Total LTV</p>
          <p className="mt-1 text-sm font-semibold">Rs. {totalLtv}</p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Fleet Size</p>
          <p className="mt-1 text-sm font-semibold">{fleetCount}</p>
        </div>
      </section>

      <div className="grid gap-6 lg:grid-cols-12">
        <aside className="space-y-5 lg:col-span-4">
          <div className="rounded-sm border border-border bg-surface p-5 space-y-4">
            <div className="flex items-start gap-4">
              <Avatar name={customer.name} size="xl" />
              <div className="min-w-0">
                <h2 className="truncate text-xl font-bold uppercase tracking-[0.04em]">{customer.name}</h2>
                <p className="text-xs uppercase tracking-[0.12em] text-muted-foreground">
                  {customer.type === 'ORGANIZATION' ? 'Organization account' : 'Individual account'}
                </p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3 border-t border-border pt-4 text-xs uppercase tracking-[0.14em]">
              <div className="rounded-sm bg-background p-2">
                <p className="text-muted-foreground">Last Visit</p>
                <p className="mt-1 text-[11px] text-foreground">{new Date(lastVisit).toLocaleDateString()}</p>
              </div>
              <div className="rounded-sm bg-background p-2">
                <p className="text-muted-foreground">Next Service</p>
                <p className="mt-1 text-[11px] text-foreground">{new Date(nextServiceHint).toLocaleDateString()}</p>
              </div>
            </div>

            <div className="space-y-3 border-t border-border pt-4">
              {customer.phone && (
                <div className="flex items-start gap-3 text-sm">
                  <Phone className="h-4 w-4 text-muted-foreground" />
                  <div>
                    <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Primary Comm</p>
                    <p>{customer.phone}</p>
                  </div>
                </div>
              )}
              {customer.email && (
                <div className="flex items-start gap-3 text-sm">
                  <Mail className="h-4 w-4 text-muted-foreground" />
                  <div className="min-w-0">
                    <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Secure Mail</p>
                    <p className="truncate">{customer.email}</p>
                  </div>
                </div>
              )}
              {customer.address && (
                <div className="flex items-start gap-3 text-sm">
                  <MapPin className="h-4 w-4 text-muted-foreground" />
                  <div>
                    <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Hangar Location</p>
                    <p>{customer.address}</p>
                  </div>
                </div>
              )}
              <div className="flex items-start gap-3 text-sm">
                <Calendar className="h-4 w-4 text-muted-foreground" />
                <div>
                  <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Created</p>
                  <p>{new Date(customer.created_at).toLocaleDateString()}</p>
                </div>
              </div>
            </div>
          </div>

          <div className="rounded-sm border border-border bg-surface p-5">
            <h3 className="mb-4 text-base font-semibold uppercase tracking-[0.08em] flex items-center gap-2">
              <ShieldCheck className="h-5 w-5" /> Loyalty Analysis
            </h3>
            <div className="space-y-3 text-sm">
              <div>
                <p className="text-xs uppercase tracking-[0.14em] text-muted-foreground">Progress to next tier</p>
                <div className="mt-2 h-2 rounded-sm bg-background">
                  <div
                    className="h-2 rounded-sm bg-primary"
                    style={{ width: `${Math.min(96, Math.max(24, loyaltyPoints % 100))}%` }}
                  />
                </div>
              </div>
              <p className="text-xs uppercase tracking-[0.12em] text-muted-foreground">Priority bay allocation active</p>
              <p className="text-xs uppercase tracking-[0.12em] text-muted-foreground">Free recovery available by tier</p>
            </div>
          </div>

          <div className="rounded-sm border border-border bg-surface p-5">
            <h3 className="mb-4 text-base font-semibold uppercase tracking-[0.08em] flex items-center gap-2">
              <AlertTriangle className="h-5 w-5" /> Lead Mechanic Notes
            </h3>
            <p className="text-sm text-muted-foreground">{leadNote ?? 'No critical workshop notes on file.'}</p>
          </div>
        </aside>

        <section className="space-y-5 lg:col-span-8">
          <div className="rounded-sm border border-border bg-surface p-5">
            <h3 className="mb-4 text-base font-semibold uppercase tracking-[0.08em]">Financial Snapshot</h3>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              <div className="rounded-sm border border-border bg-background p-3">
                <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Total Invoices</p>
                <p className="mt-1 text-sm font-semibold">{customer.financialSnapshot?.totalInvoices ?? 0}</p>
              </div>
              <div className="rounded-sm border border-border bg-background p-3">
                <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Total Spend</p>
                <p className="mt-1 text-sm font-semibold">Rs. {customer.financialSnapshot?.totalSpend ?? '0.00'}</p>
              </div>
              <div className="rounded-sm border border-border bg-background p-3">
                <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Outstanding</p>
                <p className="mt-1 text-sm font-semibold">Rs. {customer.financialSnapshot?.outstandingBalance ?? '0.00'}</p>
              </div>
              <div className="rounded-sm border border-border bg-background p-3">
                <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Paid Invoices</p>
                <p className="mt-1 text-sm font-semibold">{customer.financialSnapshot?.paidInvoices ?? 0}</p>
              </div>
            </div>
            <div className="mt-3">
              <Button variant="outline" size="sm">
                <Wallet className="mr-1 h-4 w-4" />
                Generate Statement
              </Button>
            </div>
          </div>

          <div className="rounded-sm border border-border bg-surface p-5">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-base font-semibold uppercase tracking-[0.08em] flex items-center gap-2">
                <Car className="h-5 w-5" /> Registered Fleet
              </h3>
              <Button variant="outline" size="sm" asChild>
                <Link to={`/vehicles/new?customerId=${customer.id}`}>Add Vehicle</Link>
              </Button>
            </div>

            {customer.vehicles && customer.vehicles.length > 0 ? (
              <div className="space-y-2">
                {customer.vehicles.map((vehicle) => (
                  <Link
                    key={vehicle.id}
                    to={`/vehicles/${vehicle.id}`}
                    className="flex items-center justify-between rounded-sm border border-border p-3 hover:border-accent"
                  >
                    <div>
                      <p className="font-medium">{vehicle.brand} {vehicle.model}</p>
                      <p className="text-xs uppercase tracking-wide text-muted-foreground">VIN/PLATE: {vehicle.registration_no}</p>
                    </div>
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <span>{vehicle.year || '-'}</span>
                      <ChevronRight className="h-3 w-3" />
                    </div>
                  </Link>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">No vehicles registered</p>
            )}
          </div>

          <div className="rounded-sm border border-border bg-surface p-5">
            <div className="mb-4 flex items-center justify-between">
              <h3 className="text-base font-semibold uppercase tracking-[0.08em] flex items-center gap-2">
                <FileText className="h-5 w-5" /> Service Chronicle
              </h3>
            </div>
            {customer.serviceChronicle && customer.serviceChronicle.length > 0 ? (
              <div className="space-y-2">
                {customer.serviceChronicle.map((entry) => (
                  <div key={entry.id} className="rounded-sm border border-border p-3 bg-background">
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-sm font-semibold uppercase tracking-[0.06em]">{entry.referenceNo}</p>
                      <span className="text-xs uppercase tracking-wide text-muted-foreground">
                        {new Date(entry.occurredAt).toLocaleDateString()}
                      </span>
                    </div>
                    <p className="mt-1 text-xs uppercase tracking-wide text-muted-foreground">
                      {entry.kind.replaceAll('_', ' ')} - {entry.status}
                    </p>
                    <p className="mt-2 text-sm">{entry.summary}</p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">No service chronicle entries yet</p>
            )}
          </div>

          <div className="rounded-sm border border-border bg-surface p-5">
            <div className="mb-4 flex items-center justify-between">
              <h3 className="text-base font-semibold uppercase tracking-[0.08em] flex items-center gap-2">
                <FileText className="h-5 w-5" /> Billing Ledger
              </h3>
            </div>

            {customer.invoices && customer.invoices.length > 0 ? (
              <div className="space-y-2">
                {customer.invoices.map((invoice) => (
                  <Link
                    key={invoice.id}
                    to={`/billing/${invoice.id}`}
                    className="flex items-center justify-between rounded-sm border border-border p-3 hover:border-accent"
                  >
                    <div>
                      <p className="font-medium">INV-{invoice.invoice_no ?? '-'}</p>
                      <p className="text-xs uppercase tracking-wide text-muted-foreground">
                        {new Date(invoice.created_at).toLocaleDateString()}
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="font-medium">Rs. {invoice.total_amount}</p>
                      <p className="text-xs uppercase tracking-wide text-muted-foreground">{invoice.status}</p>
                    </div>
                  </Link>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">No invoices yet</p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}