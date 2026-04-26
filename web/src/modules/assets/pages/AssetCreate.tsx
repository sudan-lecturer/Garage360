import { useState } from 'react';
import { AxiosError } from 'axios';
import { useNavigate } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { Button } from '@/components/ui/button';
import { useCreateAsset } from '@/api/hooks/useAssets';

export default function AssetCreatePage() {
  const navigate = useNavigate();
  const [errorMessage, setErrorMessage] = useState('');
  const [form, setForm] = useState({
    assetTag: '',
    name: '',
    category: '',
    locationId: '',
    purchaseDate: '',
    purchaseCost: '',
    usefulLifeYears: '',
    status: 'ACTIVE',
    notes: '',
  });
  const createMutation = useCreateAsset();

  const onSubmit = () => {
    setErrorMessage('');
    if (!form.assetTag.trim() || !form.name.trim()) {
      setErrorMessage('Asset tag and name are required.');
      return;
    }

    createMutation.mutate(
      {
        assetTag: form.assetTag.trim(),
        name: form.name.trim(),
        category: form.category.trim() || undefined,
        locationId: form.locationId.trim() || undefined,
        purchaseDate: form.purchaseDate || undefined,
        purchaseCost: form.purchaseCost || undefined,
        usefulLifeYears: form.usefulLifeYears ? Number(form.usefulLifeYears) : undefined,
        status: form.status,
        notes: form.notes.trim() || undefined,
      },
      {
        onSuccess: () => navigate('/assets'),
        onError: (error) => {
          const typed = error as AxiosError<{ error?: { message?: string } }>;
          setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to create asset.');
        },
      }
    );
  };

  return (
    <div className="space-y-6">
      <PageHeader title="Create Asset" description="Register workshop asset and lifecycle metadata." />

      <section className="rounded-sm border border-border bg-surface p-4 space-y-3">
        <div className="grid gap-3 sm:grid-cols-2">
          <input value={form.assetTag} onChange={(e) => setForm((s) => ({ ...s, assetTag: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm" placeholder="Asset Tag *" />
          <input value={form.name} onChange={(e) => setForm((s) => ({ ...s, name: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm" placeholder="Asset Name *" />
          <input value={form.category} onChange={(e) => setForm((s) => ({ ...s, category: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm" placeholder="Category" />
          <input value={form.locationId} onChange={(e) => setForm((s) => ({ ...s, locationId: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm" placeholder="Location ID" />
          <input type="date" value={form.purchaseDate} onChange={(e) => setForm((s) => ({ ...s, purchaseDate: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm" />
          <input type="number" value={form.purchaseCost} onChange={(e) => setForm((s) => ({ ...s, purchaseCost: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm" placeholder="Purchase Cost" min="0" step="0.01" />
          <input type="number" value={form.usefulLifeYears} onChange={(e) => setForm((s) => ({ ...s, usefulLifeYears: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm" placeholder="Useful Life (years)" min="0" />
          <select value={form.status} onChange={(e) => setForm((s) => ({ ...s, status: e.target.value }))} className="h-10 rounded-sm border border-input bg-background px-3 text-sm">
            <option value="ACTIVE">Active</option>
            <option value="MAINTENANCE">Maintenance</option>
            <option value="RETIRED">Retired</option>
          </select>
        </div>
        <textarea value={form.notes} onChange={(e) => setForm((s) => ({ ...s, notes: e.target.value }))} className="min-h-20 w-full rounded-sm border border-input bg-background px-3 py-2 text-sm" placeholder="Notes" />
      </section>

      {errorMessage && <div className="rounded-sm border border-destructive bg-destructive-muted p-3 text-sm text-destructive">{errorMessage}</div>}

      <div className="flex flex-wrap gap-2">
        <Button variant="outline" onClick={() => navigate('/assets')}>Cancel</Button>
        <Button onClick={onSubmit} disabled={createMutation.isPending}>
          {createMutation.isPending ? 'Creating...' : 'Create Asset'}
        </Button>
      </div>
    </div>
  );
}
