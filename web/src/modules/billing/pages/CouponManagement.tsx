import { useState } from 'react';
import { PageHeader } from '@/components/shared/page-header';
import { Button } from '@/components/ui/button';

interface Coupon {
  id: string;
  code: string;
  discountPct: number;
  status: 'ACTIVE' | 'EXPIRED' | 'DISABLED';
}

export default function CouponManagementPage() {
  const [coupons, setCoupons] = useState<Coupon[]>([]);
  const [code, setCode] = useState('');
  const [discountPct, setDiscountPct] = useState('10');

  return (
    <div className="space-y-4">
      <PageHeader title="Coupon Management" description="Manage discounts and promotional offers." />

      <section className="rounded-sm border border-border bg-surface p-4 space-y-3">
        <h3 className="text-base font-semibold uppercase tracking-[0.08em]">Create Coupon</h3>
        <div className="grid gap-2 sm:grid-cols-12">
          <input
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-6"
            placeholder="Coupon code"
            value={code}
            onChange={(e) => setCode(e.target.value.toUpperCase())}
          />
          <input
            type="number"
            min="1"
            max="100"
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-3"
            placeholder="Discount %"
            value={discountPct}
            onChange={(e) => setDiscountPct(e.target.value)}
          />
          <Button
            className="sm:col-span-3"
            onClick={() => {
              if (!code.trim()) return;
              setCoupons((prev) => [
                {
                  id: crypto.randomUUID(),
                  code: code.trim(),
                  discountPct: Number(discountPct) || 0,
                  status: 'ACTIVE',
                },
                ...prev,
              ]);
              setCode('');
            }}
          >
            Add Coupon
          </Button>
        </div>
      </section>

      <section className="rounded-sm border border-border bg-surface p-4">
        <h3 className="mb-3 text-base font-semibold uppercase tracking-[0.08em]">Coupons</h3>
        {coupons.length === 0 ? (
          <p className="text-sm text-muted-foreground">No coupons yet.</p>
        ) : (
          <div className="space-y-2">
            {coupons.map((coupon) => (
              <div key={coupon.id} className="flex items-center justify-between rounded-sm border border-border p-3">
                <div>
                  <p className="text-sm font-semibold">{coupon.code}</p>
                  <p className="text-xs uppercase tracking-wide text-muted-foreground">
                    {coupon.discountPct}% off
                  </p>
                </div>
                <span className="text-xs uppercase tracking-wide text-muted-foreground">{coupon.status}</span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
