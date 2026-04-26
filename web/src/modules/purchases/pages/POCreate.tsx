import { useMemo, useState } from 'react';
import { AxiosError } from 'axios';
import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { PageHeader } from '@/components/shared/page-header';
import { useCreatePurchaseOrder, usePurchaseOrders } from '@/api/hooks/usePurchases';

interface PurchaseItemDraft {
  description: string;
  quantity: string;
  unitPrice: string;
}

function useSupplierOptions() {
  return usePurchaseOrders({ page: 1, limit: 100, search: undefined, status: undefined, }).data?.data;
}

export default function POCreatePage() {
  const navigate = useNavigate();
  const [supplierId, setSupplierId] = useState('');
  const [expectedDelivery, setExpectedDelivery] = useState('');
  const [notes, setNotes] = useState('');
  const [errorMessage, setErrorMessage] = useState('');
  const [items, setItems] = useState<PurchaseItemDraft[]>([
    { description: '', quantity: '1', unitPrice: '0' },
  ]);
  const supplierRows = useSupplierOptions() ?? [];
  const supplierOptions = useMemo(() => {
      const seen = new Map<string, string>();
      supplierRows.forEach((row) => {
        if (!seen.has(row.supplier_id)) {
          seen.set(row.supplier_id, row.supplier_name);
        }
      });

      return Array.from(seen.entries()).map(([value, label]) => ({ value, label }));
  }, [supplierRows]);

  const subtotal = useMemo(() => {
    return items.reduce((sum, item) => {
      const quantity = Number(item.quantity) || 0;
      const unitPrice = Number(item.unitPrice) || 0;
      return sum + quantity * unitPrice;
    }, 0);
  }, [items]);

  const createMutation = useCreatePurchaseOrder();

  const updateItem = (index: number, field: keyof PurchaseItemDraft, value: string) => {
    setItems((current) =>
      current.map((item, itemIndex) =>
        itemIndex === index
          ? {
              ...item,
              [field]: value,
            }
          : item
      )
    );
  };

  const removeItem = (index: number) => {
    setItems((current) => current.filter((_, itemIndex) => itemIndex !== index));
  };

  const onSubmit = () => {
    setErrorMessage('');
    const validItems = items.filter(
      (item) => item.description.trim().length > 0 && Number(item.quantity) > 0
    );

    if (supplierId.trim().length === 0) {
      setErrorMessage('Supplier is required.');
      return;
    }

    if (validItems.length === 0) {
      setErrorMessage('Add at least one valid line item.');
      return;
    }

    createMutation.mutate(
      {
        supplierId,
        expectedDelivery: expectedDelivery || undefined,
        notes: notes.trim().length > 0 ? notes : undefined,
        items: validItems.map((item) => ({
          description: item.description.trim(),
          quantity: Number(item.quantity),
          unitPrice: Number(item.unitPrice),
        })),
      },
      {
        onSuccess: () => {
          navigate('/purchases');
        },
        onError: (error) => {
          const typed = error as AxiosError<{ error?: { message?: string } }>;
          const message = typed.response?.data?.error?.message ?? 'Failed to create purchase order.';
          setErrorMessage(message);
        },
      }
    );
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="Create Purchase Order"
        description="Create a purchase order and line items."
        breadcrumbs={[
          { label: 'Purchases', href: '/purchases' },
          { label: 'New Purchase Order' },
        ]}
      />

      <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
        <h2 className="text-lg font-semibold text-foreground">Order Details</h2>

        <div className="space-y-2">
          <label htmlFor="supplierId" className="text-sm font-medium">
            Supplier
          </label>
          <select
            id="supplierId"
            value={supplierId}
            onChange={(event) => setSupplierId(event.target.value)}
            className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
          >
            <option value="">Select supplier</option>
            {supplierOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          <p className="text-xs text-muted-foreground">
            Supplier options are inferred from existing purchase history.
          </p>
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          <div className="space-y-2">
            <label htmlFor="expectedDelivery" className="text-sm font-medium">
              Expected Delivery
            </label>
            <input
              id="expectedDelivery"
              type="date"
              value={expectedDelivery}
              onChange={(event) => setExpectedDelivery(event.target.value)}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="notes" className="text-sm font-medium">
              Notes
            </label>
            <input
              id="notes"
              type="text"
              value={notes}
              onChange={(event) => setNotes(event.target.value)}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
              placeholder="Optional notes"
            />
          </div>
        </div>
      </section>

      <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-foreground">Line Items</h2>
          <Button
            type="button"
            variant="outline"
            onClick={() =>
              setItems((current) => [
                ...current,
                { description: '', quantity: '1', unitPrice: '0' },
              ])
            }
          >
            Add Item
          </Button>
        </div>

        <div className="space-y-3">
          {items.map((item, index) => (
            <div key={`${index}-${item.description}`} className="grid gap-2 sm:grid-cols-12">
              <input
                value={item.description}
                onChange={(event) => updateItem(index, 'description', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-6"
                placeholder="Description"
              />
              <input
                value={item.quantity}
                onChange={(event) => updateItem(index, 'quantity', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-2"
                type="number"
                min="0.001"
                step="0.001"
                placeholder="Qty"
              />
              <input
                value={item.unitPrice}
                onChange={(event) => updateItem(index, 'unitPrice', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-2"
                type="number"
                min="0"
                step="0.01"
                placeholder="Unit Price"
              />
              <Button
                type="button"
                variant="ghost"
                className="sm:col-span-2"
                onClick={() => removeItem(index)}
                disabled={items.length === 1}
              >
                Remove
              </Button>
            </div>
          ))}
        </div>

        <div className="rounded-md border border-border bg-background p-3 text-sm">
          Subtotal: <span className="font-semibold">Rs. {subtotal.toLocaleString()}</span>
        </div>
      </section>

      {errorMessage && (
        <div className="rounded-md border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
          {errorMessage}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" onClick={() => navigate('/purchases')}>
          Cancel
        </Button>
        <Button type="button" onClick={onSubmit} disabled={createMutation.isPending}>
          {createMutation.isPending ? 'Creating...' : 'Create Purchase Order'}
        </Button>
      </div>
    </div>
  );
}
