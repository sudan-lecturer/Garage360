import * as React from 'react';
import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/utils';

/* ============================================
   BUTTON VARIANTS
   Following the 4px rule & atomic design
   ============================================ */
const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap font-medium transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50',
  {
    variants: {
      variant: {
        /* Primary - Industrial Amber */
        primary: 
          'bg-primary text-primary-foreground hover:bg-primary-hover active:scale-[0.98] shadow-sm hover:shadow-md',
        
        /* Secondary - Slate */
        secondary: 
          'bg-secondary text-secondary-foreground hover:bg-secondary-hover border border-border hover:border-border-hover',
        
        /* Ghost - Minimal */
        ghost: 
          'hover:bg-surface-raised text-foreground-muted hover:text-foreground',
        
        /* Outline */
        outline: 
          'border-2 border-border bg-transparent hover:border-primary hover:text-primary',
        
        /* Destructive */
        destructive: 
          'bg-destructive text-white hover:bg-destructive/90 active:scale-[0.98]',
        
        /* Success */
        success: 
          'bg-success text-white hover:bg-success/90 active:scale-[0.98]',
        
        /* Link */
        link: 
          'text-primary underline-offset-4 hover:underline p-0 h-auto',
      },
      size: {
        /* 4px rule: heights are multiples of 4 */
        xs:    'h-8  px-3 text-xs rounded-sm',   /* 32px */
        sm:    'h-9  px-4 text-sm rounded-sm',   /* 36px */
        md:    'h-10 px-4 text-sm rounded-md',  /* 40px - default */
        lg:    'h-12 px-6 text-base rounded-md', /* 48px */
        xl:    'h-14 px-8 text-lg rounded-lg',  /* 56px */
        icon:  'h-10 w-10  rounded-md',         /* 44px min for touch */
        'icon-sm': 'h-8 w-8 rounded-sm',
        'icon-lg': 'h-12 w-12 rounded-lg',
      },
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
    },
  }
);

/* ============================================
   BUTTON PROPS
   ============================================ */
export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
  isLoading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

/* ============================================
   BUTTON COMPONENT
   Atom: Most fundamental interactive element
   ============================================ */
const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ 
    className, 
    variant, 
    size, 
    asChild = false, 
    isLoading = false,
    leftIcon,
    rightIcon,
    children,
    disabled,
    ...props 
  }, ref) => {
    const Comp = asChild ? Slot : 'button';

    return (
      <Comp
        ref={ref}
        className={cn(
          buttonVariants({ variant, size, className }),
          isLoading && 'cursor-wait opacity-70'
        )}
        disabled={disabled || isLoading}
        {...props}
      >
        {isLoading ? (
          <span className="flex items-center gap-2">
            <svg 
              className="h-4 w-4 animate-spin" 
              xmlns="http://www.w3.org/2000/svg" 
              fill="none" 
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
            <span>Loading...</span>
          </span>
        ) : (
          <>
            {leftIcon && (
              <span className="flex-shrink-0" aria-hidden="true">
                {leftIcon}
              </span>
            )}
            {children}
            {rightIcon && (
              <span className="flex-shrink-0" aria-hidden="true">
                {rightIcon}
              </span>
            )}
          </>
        )}
      </Comp>
    );
  }
);

Button.displayName = 'Button';

export { Button, buttonVariants };
