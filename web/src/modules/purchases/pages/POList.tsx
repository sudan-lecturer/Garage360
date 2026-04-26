import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { SearchInput } from '@/components/shared/search-input';
import { StatusBadge } from '@/components/shared/status-badge';
import { Button } from '@/components/ui/button';
import { Plus, ShoppingCart, ChevronRight } from 'lucide-react';
import { usePurchaseOrders } from '@/api/hooks/usePurchases';

export default function POListPage() {
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('');
  
  const { data, isLoading, error } = usePurchaseOrders({
    search: search || undefined,
    status: statusFilter || undefined,
  });

  const statusOptions = ['DRAFT', 'SUBMITTED', 'APPROVED', 'SENT', 'IN_TRANSIT', 'RECEIVED'];

  return (
    <div className="space-y-4">
      <PageHeader
        title="Purchase Orders"
        description="Manage purchase orders and GRN"
        actions={
          <Button asChild>
            <Link to="/purchases/new">
              <Plus className="h-4 w-4 mr-1" /> New PO
            </Link>
          </Button>
        }
      />

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="w-full sm:w-64">
          <SearchInput value={search} onChange={setSearch} placeholder="Search POs..." />
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="h-10 rounded-md border border-input bg-background px-3 text-sm"
        >
          <option value="">All Statuses</option>
          {statusOptions.map(s => (
            <option key={s} value={s}>{s.replace('_', ' ')}</option>
          ))}
        </select>
      </div>

      {isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {error && <EmptyState icon="default" title="Error loading purchase orders" description="Please try again later" />}
      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState icon="search" title="No purchase orders found" description={search ? 'Try adjusting search' : 'No POs yet'}
          action={{ label: 'New PO', onClick: () => navigate('/purchases/new') }} />
      )}

      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-lg border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">PO #</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Supplier</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Items</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Total</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Date</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map(po => (
                <tr key={po.id} className="border-b border-border last:border-0 hover:bg-surface-raised">
                  <td className="p-3">
                    <Link to={`/purchases/${po.id}`} className="flex items-center gap-2 hover:text-accent">
                      <ShoppingCart className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">PO-{po.po_no || '-'}</span>
                    </Link>
                  </td>
                  <td className="p-3 text-sm">{po.supplier_name}</td>
                  <td className="p-3"><StatusBadge variant={po.status.toLowerCase() as any} /></td>
                  <td className="p-3 text-sm text-right">-</td>
                  <td className="p-3 text-sm text-right font-medium">Rs. {parseFloat(po.total_amount).toLocaleString()}</td>
                  <td className="p-3 text-sm text-muted-foreground">{new Date(po.created_at).toLocaleDateString()}</td>
                  <td className="p-3 text-right">
                    <Link to={`/purchases/${po.id}`} className="inline-flex items-center text-accent hover:underline">
                      View <ChevronRight className="h-3 w-3" />
                    </Link>
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