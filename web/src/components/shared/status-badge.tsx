import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';

const statusBadgeVariants = cva(
  'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors',
  {
    variants: {
      variant: {
        intake: 'bg-info-muted text-info',
        audit: 'bg-info-muted text-info',
        quote: 'bg-warning-muted text-warning',
        approval: 'bg-info-muted text-info',
        in_service: 'bg-info-muted text-info',
        qa: 'bg-info-muted text-info',
        billing: 'bg-success-muted text-success',
        completed: 'bg-success-muted text-success',
        cancelled: 'bg-destructive-muted text-destructive',
        draft: 'bg-muted text-muted-foreground',
        submitted: 'bg-info-muted text-info',
        approved: 'bg-success-muted text-success',
        sent: 'bg-info-muted text-info',
        in_transit: 'bg-warning-muted text-warning',
        received: 'bg-success-muted text-success',
        issued: 'bg-info-muted text-info',
        paid: 'bg-success-muted text-success',
        void: 'bg-muted text-muted-foreground',
        active: 'bg-success-muted text-success',
        inactive: 'bg-muted text-muted-foreground',
        ok: 'bg-success-muted text-success',
        low: 'bg-warning-muted text-warning',
        out: 'bg-destructive-muted text-destructive',
        free: 'bg-success-muted text-success',
        occupied: 'bg-warning-muted text-warning',
        reserved: 'bg-muted text-muted-foreground',
        maintenance: 'bg-destructive-muted text-destructive',
      },
      size: {
        sm: 'px-2 py-0.5 text-[10px]',
        md: 'px-2.5 py-0.5 text-xs',
        lg: 'px-3 py-1 text-sm',
      },
    },
    defaultVariants: {
      variant: 'active',
      size: 'md',
    },
  }
);

export interface StatusBadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof statusBadgeVariants> {}

function StatusBadge({ className, variant, size, ...props }: StatusBadgeProps) {
  return (
    <span
      className={cn(statusBadgeVariants({ variant, size }), className)}
      {...props}
    />
  );
}

export { StatusBadge, statusBadgeVariants };