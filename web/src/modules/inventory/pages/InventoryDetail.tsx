import { useState } from 'react';
import { AxiosError } from 'axios';
import { useParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { Button } from '@/components/ui/button';
import { useAdjustInventoryStock, useInventoryItem } from '@/api/hooks/useInventory';

export default function InventoryDetailPage() {
  const { id } = useParams();
  const query = useInventoryItem(id);
  const adjustMutation = useAdjustInventoryStock();
  const [errorMessage, setErrorMessage] = useState('');
  const [showAdjust, setShowAdjust] = useState(false);
  const [adjustType, setAdjustType] = useState<'ADD' | 'REMOVE' | 'SET' | 'COUNT'>('ADD');
  const [quantity, setQuantity] = useState('0');
  const [reason, setReason] = useState('');

  const item = query.data?.item;

  const onAdjust = () => {
    setErrorMessage('');
    adjustMutation.mutate(
      { id: item?.id ?? '', adjustmentType: adjustType, quantity, reason: reason || undefined },
      {
        onSuccess: () => {
          setShowAdjust(false);
          setQuantity('0');
          setReason('');
        },
        onError: (error) => {
          const typed = error as AxiosError<{ error?: { message?: string } }>;
          setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to adjust stock.');
        },
      }
    );
  };

  return (
    <div className="space-y-6">
      <PageHeader title={item?.name ?? 'Inventory Detail'} description="Inventory detail and stock movements." />

      {query.isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {!query.isLoading && !item && <EmptyState title="Inventory item not found" description="Unable to load item details." />}

      {item && <section className="rounded-sm border border-border bg-surface p-4">
        <dl className="grid gap-3 sm:grid-cols-2">
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">SKU</dt><dd>{item.sku}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Category</dt><dd>{item.category || '-'}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Current Quantity</dt><dd>{item.current_quantity}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Min Stock Level</dt><dd>{item.min_stock_level}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Cost Price</dt><dd>Rs. {item.cost_price}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Sell Price</dt><dd>Rs. {item.sell_price}</dd></div>
        </dl>
      </section>}

      {item && <section className="rounded-sm border border-border bg-surface p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Stock Adjustment</h2>
          <Button variant="outline" onClick={() => setShowAdjust(true)}>Adjust Stock</Button>
        </div>
      </section>}

      {item && showAdjust && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" role="dialog" aria-modal="true" aria-labelledby="adjust-stock-title">
          <div className="w-full max-w-2xl rounded-sm border border-border bg-surface p-4 space-y-3">
            <div className="flex items-center justify-between">
              <h2 id="adjust-stock-title" className="text-lg font-semibold">Adjust Stock</h2>
              <Button variant="ghost" onClick={() => setShowAdjust(false)}>Close</Button>
            </div>
            <div className="grid gap-2 sm:grid-cols-12">
              <select
                value={adjustType}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value === 'ADD' || value === 'REMOVE' || value === 'SET' || value === 'COUNT') {
                    setAdjustType(value);
                  }
                }}
                className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-3"
              >
                <option value="ADD">Add</option>
                <option value="REMOVE">Remove</option>
                <option value="SET">Set</option>
                <option value="COUNT">Count</option>
              </select>
              <input type="number" value={quantity} onChange={(e) => setQuantity(e.target.value)} className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-3" min="0" step="0.001" placeholder="Quantity" />
              <input value={reason} onChange={(e) => setReason(e.target.value)} className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-4" placeholder="Reason (optional)" />
              <Button className="sm:col-span-2" onClick={onAdjust} disabled={adjustMutation.isPending}>Apply</Button>
            </div>
          </div>
        </div>
      )}

      {errorMessage && (
        <div className="rounded-sm border border-destructive bg-destructive-muted p-3 text-sm text-destructive">{errorMessage}</div>
      )}
    </div>
  );
}
