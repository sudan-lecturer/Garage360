import { useState } from 'react';
import { PageHeader } from '@/components/shared/page-header';
import { Button } from '@/components/ui/button';
import { CheckCircle } from 'lucide-react';

export default function PaymentConfirmationPage() {
  const [referenceNo, setReferenceNo] = useState('');
  const [invoiceNo, setInvoiceNo] = useState('');
  const [amount, setAmount] = useState('');
  const [confirmed, setConfirmed] = useState(false);

  return (
    <div className="space-y-4">
      <PageHeader title="Payment Confirmation" description="Confirm payment settlement records." />

      <section className="max-w-xl rounded-sm border border-border bg-surface p-4 space-y-3">
        <input
          className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm"
          placeholder="Invoice number"
          value={invoiceNo}
          onChange={(e) => setInvoiceNo(e.target.value)}
        />
        <input
          className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm"
          placeholder="Reference number"
          value={referenceNo}
          onChange={(e) => setReferenceNo(e.target.value)}
        />
        <input
          type="number"
          className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm"
          placeholder="Amount"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
        />
        <Button
          onClick={() => setConfirmed(Boolean(invoiceNo.trim() && referenceNo.trim() && Number(amount) > 0))}
        >
          Confirm Payment
        </Button>

        {confirmed && (
          <div className="rounded-sm border border-success/40 bg-success/10 p-3 text-sm">
            <p className="flex items-center gap-2 font-semibold text-success">
              <CheckCircle className="h-4 w-4" />
              Payment confirmed successfully.
            </p>
            <p className="mt-1 text-muted-foreground">
              Invoice {invoiceNo} / Ref {referenceNo} / Amount Rs. {amount}
            </p>
          </div>
        )}
      </section>
    </div>
  );
}
