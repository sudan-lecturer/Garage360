import { useMemo, useState } from 'react';
import { AxiosError } from 'axios';
import { useNavigate } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { Button } from '@/components/ui/button';
import { useCreateInvoice } from '@/api/hooks/useBilling';

interface LineDraft {
  description: string;
  quantity: string;
  unitPrice: string;
  discountPct: string;
}

export default function InvoiceCreatePage() {
  const navigate = useNavigate();
  const [customerId, setCustomerId] = useState('');
  const [jobCardId, setJobCardId] = useState('');
  const [notes, setNotes] = useState('');
  const [taxAmount, setTaxAmount] = useState('0');
  const [discountPct, setDiscountPct] = useState('0');
  const [errorMessage, setErrorMessage] = useState('');
  const [lines, setLines] = useState<LineDraft[]>([
    { description: '', quantity: '1', unitPrice: '0', discountPct: '0' },
  ]);

  const createMutation = useCreateInvoice();

  const subtotal = useMemo(
    () =>
      lines.reduce((sum, l) => {
        const qty = Number(l.quantity) || 0;
        const unit = Number(l.unitPrice) || 0;
        const d = Number(l.discountPct) || 0;
        return sum + qty * unit * (1 - d / 100);
      }, 0),
    [lines]
  );

  const onSubmit = () => {
    setErrorMessage('');
    if (!customerId.trim()) {
      setErrorMessage('Customer ID is required.');
      return;
    }
    const lineItems = lines
      .filter((l) => l.description.trim().length > 0 && Number(l.quantity) > 0)
      .map((l) => ({
        description: l.description.trim(),
        quantity: Number(l.quantity),
        unitPrice: Number(l.unitPrice),
        discountPct: Number(l.discountPct) || 0,
      }));
    if (lineItems.length === 0) {
      setErrorMessage('At least one line item is required.');
      return;
    }

    createMutation.mutate(
      {
        customerId: customerId.trim(),
        jobCardId: jobCardId.trim() || undefined,
        discountPct: Number(discountPct) || 0,
        taxAmount: Number(taxAmount) || 0,
        notes: notes.trim() || undefined,
        lineItems,
      },
      {
        onSuccess: () => navigate('/billing'),
        onError: (error) => {
          const typed = error as AxiosError<{ error?: { message?: string } }>;
          setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to create invoice.');
        },
      }
    );
  };

  return (
    <div className="space-y-6">
      <PageHeader title="Create Invoice" description="Create invoice from job work and manual line items." />

      <section className="rounded-sm border border-border bg-surface p-4 space-y-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <input
            value={customerId}
            onChange={(e) => setCustomerId(e.target.value)}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Customer ID *"
          />
          <input
            value={jobCardId}
            onChange={(e) => setJobCardId(e.target.value)}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Job Card ID (optional)"
          />
          <input
            type="number"
            value={discountPct}
            onChange={(e) => setDiscountPct(e.target.value)}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Invoice Discount %"
            min="0"
            max="100"
          />
          <input
            type="number"
            value={taxAmount}
            onChange={(e) => setTaxAmount(e.target.value)}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Tax Amount"
            min="0"
            step="0.01"
          />
        </div>
        <textarea
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          className="min-h-20 w-full rounded-sm border border-input bg-background px-3 py-2 text-sm"
          placeholder="Notes"
        />
      </section>

      <section className="rounded-sm border border-border bg-surface p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Line Items</h2>
          <Button
            variant="outline"
            onClick={() =>
              setLines((current) => [
                ...current,
                { description: '', quantity: '1', unitPrice: '0', discountPct: '0' },
              ])
            }
          >
            Add Line
          </Button>
        </div>
        {lines.map((line, idx) => (
          <div key={idx} className="grid gap-2 sm:grid-cols-12">
            <input
              value={line.description}
              onChange={(e) =>
                setLines((current) =>
                  current.map((row, i) => (i === idx ? { ...row, description: e.target.value } : row))
                )
              }
              className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-5"
              placeholder="Description"
            />
            <input
              type="number"
              value={line.quantity}
              onChange={(e) =>
                setLines((current) =>
                  current.map((row, i) => (i === idx ? { ...row, quantity: e.target.value } : row))
                )
              }
              className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-2"
              placeholder="Qty"
              min="0.001"
              step="0.001"
            />
            <input
              type="number"
              value={line.unitPrice}
              onChange={(e) =>
                setLines((current) =>
                  current.map((row, i) => (i === idx ? { ...row, unitPrice: e.target.value } : row))
                )
              }
              className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-2"
              placeholder="Unit Price"
              min="0"
              step="0.01"
            />
            <input
              type="number"
              value={line.discountPct}
              onChange={(e) =>
                setLines((current) =>
                  current.map((row, i) => (i === idx ? { ...row, discountPct: e.target.value } : row))
                )
              }
              className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-2"
              placeholder="Disc %"
              min="0"
              max="100"
            />
            <Button
              variant="ghost"
              className="sm:col-span-1"
              onClick={() => setLines((current) => current.filter((_, i) => i !== idx))}
              disabled={lines.length === 1}
            >
              X
            </Button>
          </div>
        ))}
        <div className="text-sm text-muted-foreground">Subtotal: Rs. {subtotal.toLocaleString()}</div>
      </section>

      {errorMessage && (
        <div className="rounded-sm border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
          {errorMessage}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button variant="outline" onClick={() => navigate('/billing')}>
          Cancel
        </Button>
        <Button onClick={onSubmit} disabled={createMutation.isPending}>
          {createMutation.isPending ? 'Creating...' : 'Create Invoice'}
        </Button>
      </div>
    </div>
  );
}
