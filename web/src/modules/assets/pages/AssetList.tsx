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
import { Plus, Box, ChevronRight } from 'lucide-react';

interface Asset {
  id: string;
  asset_tag: string;
  name: string;
  category: string | null;
  location_id: string | null;
  status: string;
  purchase_date: string | null;
  purchase_cost: string | null;
  useful_life_years: number | null;
  open_defect_count: number;
  last_inspection_at: string | null;
}

function useAssets(params?: { search?: string; category?: string; status?: string }) {
  return useQuery({
    queryKey: ['assets', params],
    queryFn: async () => {
      const response = await api.get<{ data: Asset[] }>('/v1/assets', { params });
      return response.data;
    },
  });
}

export default function AssetListPage() {
  const navigate = useNavigate();
  const [search, setSearch] = useState('');
  const [categoryFilter, setCategoryFilter] = useState('');
  const { data, isLoading, error } = useAssets({ search: search || undefined });

  const categories = [...new Set(data?.data?.map(a => a.category || '').filter(Boolean) || [])];

  return (
    <div className="space-y-4">
      <PageHeader
        title="Assets"
        description="Manage workshop assets and equipment"
        actions={
          <Button asChild>
            <Link to="/assets/new">
              <Plus className="h-4 w-4 mr-1" /> Add Asset
            </Link>
          </Button>
        }
      />

      <div className="flex flex-col sm:flex-row gap-3">
        <div className="w-full sm:w-64">
          <SearchInput value={search} onChange={setSearch} placeholder="Search assets..." />
        </div>
        <select value={categoryFilter} onChange={(e) => setCategoryFilter(e.target.value)} className="h-10 rounded-md border border-input bg-background px-3 text-sm">
          <option value="">All Categories</option>
          {categories.map(cat => <option key={cat} value={cat}>{cat}</option>)}
        </select>
      </div>

      {isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {error && <EmptyState icon="default" title="Error loading assets" description="Please try again later" />}
      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState icon="default" title="No assets found" description="Add your first asset" action={{ label: 'Add Asset', onClick: () => navigate('/assets/new') }} />
      )}

      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-sm border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Asset Tag</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Asset Name</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Category</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Last Inspection</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map(asset => (
                <tr key={asset.id} className="border-b border-border last:border-0 hover:bg-surface-raised">
                  <td className="p-3 text-sm font-mono text-muted-foreground">{asset.asset_tag}</td>
                  <td className="p-3">
                    <Link to={`/assets/${asset.id}`} className="flex items-center gap-2 hover:text-accent">
                      <Box className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">{asset.name}</span>
                    </Link>
                  </td>
                  <td className="p-3 text-sm">{asset.category || '-'}</td>
                  <td className="p-3"><StatusBadge variant={asset.status === 'ACTIVE' ? 'active' : asset.status === 'MAINTENANCE' ? 'maintenance' : 'reserved'} /></td>
                  <td className="p-3 text-sm text-muted-foreground">{asset.last_inspection_at ? new Date(asset.last_inspection_at).toLocaleDateString() : '-'}</td>
                  <td className="p-3 text-right">
                    <Link to={`/assets/${asset.id}`} className="inline-flex items-center text-accent hover:underline">
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