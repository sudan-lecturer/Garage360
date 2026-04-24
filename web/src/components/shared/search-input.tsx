/* eslint-disable react-hooks/set-state-in-effect */
import { useState, useEffect } from 'react';
import { cn } from '@/lib/utils';
import { Search, X } from 'lucide-react';
import { Input } from '@/components/ui/input';

interface SearchInputProps {
  value?: string;
  onChange: (value: string) => void;
  placeholder?: string;
  debounceMs?: number;
  className?: string;
}

export function SearchInput({
  value: initialValue,
  onChange,
  placeholder = 'Search...',
  debounceMs = 300,
  className,
}: SearchInputProps) {
  const [internalValue, setInternalValue] = useState(initialValue || '');

  // Sync external value to internal state (required for controlled component)
  useEffect(() => {
    if (initialValue !== undefined && initialValue !== internalValue) {
      setInternalValue(initialValue);
    }
  }, [initialValue]);

  useEffect(() => {
    const timer = setTimeout(() => {
      onChange(internalValue);
    }, debounceMs);

    return () => clearTimeout(timer);
  }, [internalValue, debounceMs, onChange]);

  return (
    <div className={cn('relative', className)}>
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <Input
        type="text"
        value={internalValue}
        onChange={(e) => setInternalValue(e.target.value)}
        placeholder={placeholder}
        className="pl-9 pr-8"
      />
      {internalValue && (
        <button
          type="button"
          onClick={() => {
            setInternalValue('');
            onChange('');
          }}
          className="absolute right-2 top-1/2 -translate-y-1/2 p-1 hover:bg-surface-raised rounded"
        >
          <X className="h-3 w-3 text-muted-foreground" />
        </button>
      )}
    </div>
  );
}