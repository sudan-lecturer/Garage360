import { Link, useParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { StatusBadge } from '@/components/shared/status-badge';
import { Button } from '@/components/ui/button';
import { usePurchaseOrder, usePurchaseOrderHistory } from '@/api/hooks/usePurchases';

export default function PODetailPage() {
  const { id } = useParams();
  const orderQuery = usePurchaseOrder(id!);
  const historyQuery = usePurchaseOrderHistory(id!);

  if (orderQuery.isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  if (orderQuery.error || !orderQuery.data) {
    return (
      <EmptyState
        icon="default"
        title="Purchase order not found"
        description="The purchase order you requested does not exist."
        action={{ label: 'Back to Purchases', onClick: () => window.history.back() }}
      />
    );
  }

  const order = orderQuery.data;
  const history = historyQuery.data;

  return (
    <div className="space-y-6">
      <PageHeader
        title={`PO-${order.poNo ?? '-'}`}
        description={`Supplier: ${order.supplierName}`}
        breadcrumbs={[
          { label: 'Purchases', href: '/purchases' },
          { label: `PO-${order.poNo ?? '-'}` },
        ]}
        actions={
          <Button variant="outline" asChild>
            <Link to="/purchases">Back to list</Link>
          </Button>
        }
      />

      <section className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Status</p>
          <div className="mt-2">
            <StatusBadge variant={order.status.toLowerCase() as any} />
          </div>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Expected Delivery</p>
          <p className="mt-1 text-sm font-semibold">
            {order.expectedDelivery ? new Date(order.expectedDelivery).toLocaleDateString() : '-'}
          </p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Subtotal</p>
          <p className="mt-1 text-sm font-semibold">Rs. {order.subtotal}</p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Tax</p>
          <p className="mt-1 text-sm font-semibold">Rs. {order.taxAmount}</p>
        </div>
        <div className="rounded-sm border border-border bg-surface p-3">
          <p className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">Total</p>
          <p className="mt-1 text-sm font-semibold">Rs. {order.totalAmount}</p>
        </div>
      </section>

      <section className="rounded-sm border border-border bg-surface p-5">
        <h3 className="mb-3 text-base font-semibold uppercase tracking-[0.08em]">Line Items</h3>
        {order.items.length === 0 ? (
          <p className="text-sm text-muted-foreground">No items on this purchase order.</p>
        ) : (
          <div className="overflow-x-auto rounded-sm border border-border">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border">
                  <th className="p-3 text-left text-xs uppercase tracking-wide text-muted-foreground">Description</th>
                  <th className="p-3 text-right text-xs uppercase tracking-wide text-muted-foreground">Qty</th>
                  <th className="p-3 text-right text-xs uppercase tracking-wide text-muted-foreground">Unit Price</th>
                  <th className="p-3 text-right text-xs uppercase tracking-wide text-muted-foreground">Received</th>
                </tr>
              </thead>
              <tbody>
                {order.items.map((item) => (
                  <tr key={item.id} className="border-b border-border last:border-b-0">
                    <td className="p-3 text-sm">{item.description}</td>
                    <td className="p-3 text-right text-sm">{item.quantity}</td>
                    <td className="p-3 text-right text-sm">Rs. {item.unitPrice}</td>
                    <td className="p-3 text-right text-sm">{item.receivedQty}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className="grid gap-6 lg:grid-cols-2">
        <div className="rounded-sm border border-border bg-surface p-5">
          <h3 className="mb-3 text-base font-semibold uppercase tracking-[0.08em]">Status History</h3>
          {historyQuery.isLoading ? (
            <LoadingSpinner />
          ) : history && history.statusHistory.length > 0 ? (
            <div className="space-y-2">
              {history.statusHistory.map((entry) => (
                <div key={entry.id} className="rounded-sm border border-border bg-background p-3">
                  <p className="text-sm font-semibold">
                    {(entry.fromStatus ?? 'NEW').replaceAll('_', ' ')} {'->'} {entry.toStatus.replaceAll('_', ' ')}
                  </p>
                  <p className="mt-1 text-xs uppercase tracking-wide text-muted-foreground">
                    {entry.changedByName ?? 'System'} / {new Date(entry.createdAt).toLocaleString()}
                  </p>
                  {entry.notes && <p className="mt-2 text-sm">{entry.notes}</p>}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No status history yet.</p>
          )}
        </div>

        <div className="rounded-sm border border-border bg-surface p-5">
          <h3 className="mb-3 text-base font-semibold uppercase tracking-[0.08em]">Approvals</h3>
          {historyQuery.isLoading ? (
            <LoadingSpinner />
          ) : history && history.approvals.length > 0 ? (
            <div className="space-y-2">
              {history.approvals.map((approval) => (
                <div key={approval.id} className="rounded-sm border border-border bg-background p-3">
                  <p className="text-sm font-semibold">{approval.isApproved ? 'Approved' : 'Declined'}</p>
                  <p className="mt-1 text-xs uppercase tracking-wide text-muted-foreground">
                    {approval.approvedByName ?? 'Unknown'} / {new Date(approval.createdAt).toLocaleString()}
                  </p>
                  {approval.notes && <p className="mt-2 text-sm">{approval.notes}</p>}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No approvals recorded yet.</p>
          )}
        </div>
      </section>
    </div>
  );
}
