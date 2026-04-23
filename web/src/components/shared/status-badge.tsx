import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';

const statusBadgeVariants = cva(
  'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors',
  {
    variants: {
      variant: {
        intake: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400',
        audit: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
        quote: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400',
        approval: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
        in_service: 'bg-cyan-100 text-cyan-800 dark:bg-cyan-900/30 dark:text-cyan-400',
        qa: 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900/30 dark:text-indigo-400',
        billing: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
        completed: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-400',
        cancelled: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400',
        draft: 'bg-gray-100 text-gray-800 dark:bg-gray-800/30 dark:text-gray-400',
        submitted: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
        approved: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
        sent: 'bg-cyan-100 text-cyan-800 dark:bg-cyan-900/30 dark:text-cyan-400',
        in_transit: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400',
        received: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
        issued: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
        paid: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
        void: 'bg-gray-100 text-gray-800 dark:bg-gray-800/30 dark:text-gray-400',
        active: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
        inactive: 'bg-gray-100 text-gray-800 dark:bg-gray-800/30 dark:text-gray-400',
        ok: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
        low: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400',
        out: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400',
        free: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400',
        occupied: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400',
        reserved: 'bg-gray-100 text-gray-800 dark:bg-gray-800/30 dark:text-gray-400',
        maintenance: 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400',
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