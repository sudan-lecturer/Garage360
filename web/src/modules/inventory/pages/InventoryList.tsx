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
import { Plus, Package, ChevronRight, AlertTriangle } from 'lucide-react';

interface InventoryItem {
  id: string;
  sku: string;
  name: string;
  description: string | null;
  category: string | null;
  unit: string;
  cost_price: string;
  sell_price: string;
  current_quantity: string;
  min_stock_level: number;
  is_active: boolean;
}

function useInventory(params?: { search?: string; category?: string; low_stock?: boolean }) {
  return useQuery({
    queryKey: ['inventory', params],
    queryFn: async () => {
      const response = await api.get<{ data: InventoryItem[] }>('/v1/inventory', { params });
      return response.data;
    },
  });
}

export default function InventoryListPage() {
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('');
  const [lowStockOnly, setLowStockOnly] = useState(false);
  
  const { data, isLoading, error } = useInventory({
    search: search || undefined,
    category: categoryFilter || undefined,
    low_stock: lowStockOnly || undefined,
  });

  const categories = [...new Set(data?.data?.map(i => i.category || '').filter(Boolean) || [])];

  return (
    <div className="space-y-4">
      <PageHeader
        title="Inventory"
        description="Manage stock and inventory"
        actions={
          <Button asChild>
            <Link to="/inventory/new">
              <Plus className="h-4 w-4 mr-1" /> Add Item
            </Link>
          </Button>
        }
      />

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-3 items-start sm:items-center">
        <div className="w-full sm:w-64">
          <SearchInput
            value={search}
            onChange={setSearch}
            placeholder="Search inventory..."
          />
        </div>
        <select
          value={categoryFilter}
          onChange={(e) => setCategoryFilter(e.target.value)}
          className="h-10 rounded-md border border-input bg-background px-3 text-sm"
        >
          <option value="">All Categories</option>
          {categories.map(cat => (
            <option key={cat} value={cat}>{cat}</option>
          ))}
        </select>
        <label className="flex items-center gap-2 text-sm cursor-pointer">
          <input
            type="checkbox"
            checked={lowStockOnly}
            onChange={(e) => setLowStockOnly(e.target.checked)}
            className="h-4 w-4 rounded border-input"
          />
          <AlertTriangle className="h-4 w-4 text-warning" />
          Low stock only
        </label>
      </div>

      {/* Loading/Error/Empty */}
      {isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {error && <EmptyState icon="default" title="Error loading inventory" description="Please try again later" />}
      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState icon="search" title="No items found" description={search ? 'Try adjusting search' : 'Add your first item'} action={{ label: 'Add Item', onClick: () => navigate('/inventory/new') }} />
      )}

      {/* Table */}
      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-sm border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Item Name</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">SKU</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Category</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Stock</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Price</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map(item => {
                const qty = parseInt(item.current_quantity) || 0;
                const min = item.min_stock_level || 0;
                const status = qty === 0 ? 'out' : qty <= min ? 'low' : 'ok';
                return (
                  <tr key={item.id} className="border-b border-border last:border-0 hover:bg-surface-raised">
                    <td className="p-3">
                      <Link to={`/inventory/${item.id}`} className="flex items-center gap-2 hover:text-accent">
                        <Package className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">{item.name}</span>
                      </Link>
                    </td>
                    <td className="p-3 text-sm text-muted-foreground">{item.sku}</td>
                    <td className="p-3 text-sm text-muted-foreground">{item.category}</td>
                    <td className="p-3 text-sm text-right font-medium">{qty}</td>
                    <td className="p-3"><StatusBadge variant={status} /></td>
                    <td className="p-3 text-sm text-right">Rs. {item.sell_price}</td>
                    <td className="p-3 text-right">
                      <Link to={`/inventory/${item.id}`} className="inline-flex items-center text-accent hover:underline">
                        View <ChevronRight className="h-3 w-3" />
                      </Link>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}