import { describe, it, expect } from 'vitest';
import { cn } from './utils';

describe('cn (className merge utility)', () => {
  it('should merge two class strings', () => {
    const result = cn('class1', 'class2');
    expect(result).toBe('class1 class2');
  });

  it('should handle conditional classes', () => {
    const isActive = true;
    const result = cn('base', isActive && 'active');
    expect(result).toBe('base active');
  });

  it('should handle false conditions', () => {
    const isActive = false;
    const result = cn('base', isActive && 'active');
    expect(result).toBe('base');
  });

  it('should handle undefined and null values', () => {
    const result = cn('base', undefined, null, 'end');
    expect(result).toBe('base end');
  });

  it('should merge Tailwind classes correctly', () => {
    const result = cn('px-2 py-2', 'px-4');
    expect(result).toContain('py-2');
  });

  it('should handle empty strings', () => {
    const result = cn('class1', '', 'class2');
    expect(result).toBe('class1 class2');
  });

  it('should handle array input', () => {
    const classes = ['class1', 'class2'];
    const result = cn(...classes);
    expect(result).toBe('class1 class2');
  });

  it('should handle single class', () => {
    const result = cn('single-class');
    expect(result).toBe('single-class');
  });

  it('should handle no arguments', () => {
    const result = cn();
    expect(result).toBe('');
  });

  it('should handle object input for clsx', () => {
    const result = cn('base', { active: true, disabled: false });
    expect(result).toBe('base active');
  });
});
