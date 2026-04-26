import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import api from '@/api/client';
import { useQuery } from '@tanstack/react-query';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { SearchInput } from '@/components/shared/search-input';
import { StatusBadge } from '@/components/shared/status-badge';
import { Button } from '@/components/ui/button';
import { Plus, ChevronRight, Receipt } from 'lucide-react';

interface Invoice {
  id: string;
  invoice_no: number | null;
  job_card_id: string | null;
  job_no: number | null;
  customer_id: string;
  customer_name: string;
  status: string;
  total_amount: string;
  amount_paid: string;
  balance_due: string;
  created_at: string;
}

function useInvoices(params?: { status?: string; customer_id?: string; search?: string }) {
  return useQuery({
    queryKey: ['invoices', params],
    queryFn: async () => {
      const response = await api.get<{ data: Invoice[] }>('/v1/invoices', { params });
      return response.data;
    },
  });
}

export default function InvoiceListPage() {
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('');
  
  const { data, isLoading, error } = useInvoices({
    search: search || undefined,
    status: statusFilter || undefined,
  });

  return (
    <div className="space-y-4">
      <PageHeader
        title="Invoices"
        description="Manage invoices and billing"
        actions={
          <Button asChild>
            <Link to="/billing/new">
              <Plus className="h-4 w-4 mr-1" /> Create Invoice
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
            placeholder="Search invoices..."
          />
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="h-10 rounded-md border border-input bg-background px-3 text-sm"
        >
          <option value="">All Statuses</option>
          <option value="DRAFT">Draft</option>
          <option value="ISSUED">Issued</option>
          <option value="PAID">Paid</option>
          <option value="VOID">Void</option>
        </select>
      </div>

      {/* Loading/Error/Empty */}
      {isLoading && (
        <div className="py-12">
          <LoadingSpinner />
        </div>
      )}

      {error && (
        <EmptyState
          icon="default"
          title="Error loading invoices"
          description="Please try again later"
        />
      )}

      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState
          icon="search"
          title="No invoices found"
          description={search ? 'Try adjusting your search' : 'No invoices yet'}
          action={{
            label: 'Create Invoice',
            onClick: () => navigate('/billing/new'),
          }}
        />
      )}

      {/* Table */}
      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-sm border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Invoice #</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Job #</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Customer</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Amount</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Date</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map((invoice) => (
                <tr
                  key={invoice.id}
                  className="border-b border-border last:border-0 hover:bg-surface-raised"
                >
                  <td className="p-3">
                    <Link
                      to={`/billing/${invoice.id}`}
                      className="flex items-center gap-2 hover:text-accent"
                    >
                      <Receipt className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">INV-{invoice.invoice_no || '-'}</span>
                    </Link>
                  </td>
                  <td className="p-3 text-sm">
                    <Link to={`/jobs/${invoice.job_card_id}`} className="hover:text-accent">
                      {invoice.job_no || '-'}
                    </Link>
                  </td>
                  <td className="p-3 text-sm">
                    <Link to={`/customers/${invoice.customer_id}`} className="hover:text-accent">
                      {invoice.customer_name}
                    </Link>
                  </td>
                  <td className="p-3 text-sm text-right font-medium">
                    Rs. {parseFloat(invoice.total_amount).toLocaleString()}
                  </td>
                  <td className="p-3">
                    <StatusBadge variant={invoice.status.toLowerCase() as any} />
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    {new Date(invoice.created_at).toLocaleDateString()}
                  </td>
                  <td className="p-3 text-right">
                    <Link
                      to={`/billing/${invoice.id}`}
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
    </div>
  );
}