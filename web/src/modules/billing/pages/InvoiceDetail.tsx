import { useState } from 'react';
import { AxiosError } from 'axios';
import { useParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { Button } from '@/components/ui/button';
import { useInvoiceDetail, useRecordInvoicePayment } from '@/api/hooks/useBilling';

export default function InvoiceDetailPage() {
  const { id } = useParams();
  const query = useInvoiceDetail(id);
  const paymentMutation = useRecordInvoicePayment();
  const [showPayment, setShowPayment] = useState(false);
  const [amount, setAmount] = useState('');
  const [paymentMethod, setPaymentMethod] = useState('CASH');
  const [notes, setNotes] = useState('');
  const [errorMessage, setErrorMessage] = useState('');
  const invoice = query.data;
  const onRecordPayment = () => {
    setErrorMessage('');
    paymentMutation.mutate(
      { id: invoice?.id ?? '', amount: Number(amount), paymentMethod, notes: notes || undefined },
      {
        onSuccess: () => {
          setShowPayment(false);
          setAmount('');
          setNotes('');
        },
        onError: (error) => {
          const typed = error as AxiosError<{ error?: { message?: string } }>;
          setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to record payment.');
        },
      }
    );
  };

  return (
    <div className="space-y-6">
      <PageHeader title={`Invoice #${invoice?.invoiceNo ?? '-'}`} description="Invoice detail and payment updates." />
      {query.isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {!query.isLoading && !invoice && <EmptyState title="Invoice not found" description="Unable to load invoice detail." />}

      {invoice && <section className="rounded-sm border border-border bg-surface p-4">
        <dl className="grid gap-3 sm:grid-cols-2">
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Status</dt><dd>{invoice.status}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Customer</dt><dd>{invoice.customerName}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Total</dt><dd>Rs. {invoice.totalAmount}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Paid</dt><dd>Rs. {invoice.amountPaid}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Balance</dt><dd>Rs. {invoice.balanceDue}</dd></div>
        </dl>
      </section>}

      {invoice && <section className="rounded-sm border border-border bg-surface p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Payments</h2>
          <Button variant="outline" onClick={() => setShowPayment(true)}>Record Payment</Button>
        </div>
      </section>}

      {invoice && <section className="rounded-sm border border-border bg-surface">
        <table className="w-full">
          <thead className="border-b border-border">
            <tr>
              <th className="p-3 text-left text-xs uppercase tracking-wide text-muted-foreground">Description</th>
              <th className="p-3 text-right text-xs uppercase tracking-wide text-muted-foreground">Qty</th>
              <th className="p-3 text-right text-xs uppercase tracking-wide text-muted-foreground">Unit</th>
              <th className="p-3 text-right text-xs uppercase tracking-wide text-muted-foreground">Total</th>
            </tr>
          </thead>
          <tbody>
            {invoice.lineItems.map((line) => (
              <tr key={line.id} className="border-b border-border last:border-b-0">
                <td className="p-3">{line.description}</td>
                <td className="p-3 text-right">{line.quantity}</td>
                <td className="p-3 text-right">{line.unitPrice}</td>
                <td className="p-3 text-right">{line.lineTotal}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>}

      {invoice && showPayment && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" role="dialog" aria-modal="true" aria-labelledby="record-payment-title">
          <div className="w-full max-w-2xl rounded-sm border border-border bg-surface p-4 space-y-3">
            <div className="flex items-center justify-between">
              <h2 id="record-payment-title" className="text-lg font-semibold">Record Payment</h2>
              <Button variant="ghost" onClick={() => setShowPayment(false)}>Close</Button>
            </div>
            <div className="grid gap-2 sm:grid-cols-12">
              <input className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-3" placeholder="Amount" value={amount} onChange={(e) => setAmount(e.target.value)} type="number" min="0" step="0.01" />
              <select className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-3" value={paymentMethod} onChange={(e) => setPaymentMethod(e.target.value)}>
                <option value="CASH">Cash</option>
                <option value="CARD">Card</option>
                <option value="BANK_TRANSFER">Bank Transfer</option>
                <option value="WALLET">Wallet</option>
              </select>
              <input className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-4" placeholder="Notes (optional)" value={notes} onChange={(e) => setNotes(e.target.value)} />
              <Button className="sm:col-span-2" disabled={paymentMutation.isPending} onClick={onRecordPayment}>Save</Button>
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
