import { useEffect, useState } from 'react';
import { AxiosError } from 'axios';
import { useNavigate, useParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { Button } from '@/components/ui/button';
import {
  useCreateInventoryItem,
  useInventoryItem,
  useUpdateInventoryItem,
  type InventoryItemRequest,
} from '@/api/hooks/useInventory';

const initialForm: InventoryItemRequest = {
  sku: '',
  name: '',
  description: '',
  category: '',
  unit: 'pcs',
  cost_price: '0',
  sell_price: '0',
  min_stock_level: 0,
};

export default function InventoryFormPage() {
  const { id } = useParams();
  const editing = Boolean(id);
  const navigate = useNavigate();
  const [formState, setFormState] = useState<InventoryItemRequest>(initialForm);
  const [errorMessage, setErrorMessage] = useState('');

  const detailQuery = useInventoryItem(id);
  const createMutation = useCreateInventoryItem();
  const updateMutation = useUpdateInventoryItem();

  useEffect(() => {
    const item = detailQuery.data?.item;
    if (!item || !editing) return;
    setFormState({
      sku: item.sku,
      name: item.name,
      description: item.description ?? '',
      category: item.category ?? '',
      unit: item.unit,
      cost_price: item.cost_price,
      sell_price: item.sell_price,
      min_stock_level: item.min_stock_level,
    });
  }, [detailQuery.data, editing]);

  const submit = () => {
    setErrorMessage('');
    if (!formState.sku.trim() || !formState.name.trim()) {
      setErrorMessage('SKU and item name are required.');
      return;
    }

    if (editing && id) {
      updateMutation.mutate(
        { id, payload: formState },
        {
          onSuccess: () => navigate('/inventory'),
          onError: (error) => {
            const typed = error as AxiosError<{ error?: { message?: string } }>;
            setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to update item.');
          },
        }
      );
      return;
    }

    createMutation.mutate(formState, {
      onSuccess: () => navigate('/inventory'),
      onError: (error) => {
        const typed = error as AxiosError<{ error?: { message?: string } }>;
        setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to create item.');
      },
    });
  };

  if (editing && detailQuery.isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={editing ? 'Edit Inventory Item' : 'Add Inventory Item'}
        description="Create and maintain inventory catalog entries."
      />

      <section className="rounded-sm border border-border bg-surface p-4 space-y-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <input
            value={formState.sku}
            onChange={(e) => setFormState((s) => ({ ...s, sku: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="SKU *"
          />
          <input
            value={formState.name}
            onChange={(e) => setFormState((s) => ({ ...s, name: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Item Name *"
          />
          <input
            value={formState.category}
            onChange={(e) => setFormState((s) => ({ ...s, category: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Category"
          />
          <input
            value={formState.unit}
            onChange={(e) => setFormState((s) => ({ ...s, unit: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Unit"
          />
          <input
            type="number"
            value={formState.cost_price}
            onChange={(e) => setFormState((s) => ({ ...s, cost_price: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Cost Price"
            min="0"
            step="0.01"
          />
          <input
            type="number"
            value={formState.sell_price}
            onChange={(e) => setFormState((s) => ({ ...s, sell_price: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Sell Price"
            min="0"
            step="0.01"
          />
          <input
            type="number"
            value={formState.min_stock_level}
            onChange={(e) =>
              setFormState((s) => ({ ...s, min_stock_level: Number(e.target.value) || 0 }))
            }
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Min Stock Level"
            min="0"
          />
        </div>
        <textarea
          value={formState.description}
          onChange={(e) => setFormState((s) => ({ ...s, description: e.target.value }))}
          className="min-h-24 w-full rounded-sm border border-input bg-background px-3 py-2 text-sm"
          placeholder="Description"
        />
      </section>

      {errorMessage && (
        <div className="rounded-sm border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
          {errorMessage}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button variant="outline" onClick={() => navigate('/inventory')}>
          Cancel
        </Button>
        <Button onClick={submit} disabled={createMutation.isPending || updateMutation.isPending}>
          {editing ? 'Save Item' : 'Create Item'}
        </Button>
      </div>
    </div>
  );
}
