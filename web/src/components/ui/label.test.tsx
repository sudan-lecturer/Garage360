import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Label } from './label';
import { Input } from './input';

describe('Label', () => {
  describe('Rendering', () => {
    it('should render label element', () => {
      render(<Label>Label Text</Label>);
      expect(screen.getByText('Label Text')).toBeInTheDocument();
    });

    it('should be associated with input via htmlFor', () => {
      render(
        <>
          <Label htmlFor="email">Email</Label>
          <Input id="email" data-testid="input" />
        </>
      );

      expect(screen.getByText('Email')).toHaveAttribute('for', 'email');
    });
  });

  describe('Props forwarding', () => {
    it('should forward ref', () => {
      let ref: HTMLLabelElement | null = null;
      render(<Label ref={(el) => { ref = el; }}>Label</Label>);
      expect(ref).toBeInstanceOf(HTMLLabelElement);
    });

    it('should merge className', () => {
      render(<Label className="custom-class">Label</Label>);
      expect(screen.getByText('Label')).toHaveClass('custom-class');
    });

    it('should forward htmlFor attribute', () => {
      render(<Label htmlFor="test-input">Test</Label>);
      expect(screen.getByText('Test')).toHaveAttribute('for', 'test-input');
    });

    it('should allow additional props', () => {
      render(<Label id="my-label">Additional</Label>);
      expect(screen.getByText('Additional')).toHaveAttribute('id', 'my-label');
    });
  });

  describe('Styling', () => {
    it('should have text-sm styling', () => {
      render(<Label>Styled</Label>);
      expect(screen.getByText('Styled')).toHaveClass('text-sm');
    });

    it('should have font-medium styling', () => {
      render(<Label>Medium</Label>);
      expect(screen.getByText('Medium')).toHaveClass('font-medium');
    });
  });

  describe('Accessibility', () => {
    it('should properly associate with input', () => {
      render(
        <>
          <Label htmlFor="username">Username</Label>
          <Input id="username" data-testid="input" />
        </>
      );

      expect(screen.getByLabelText('Username')).toBeInTheDocument();
    });

    it('should have proper element type', () => {
      render(<Label>Type Test</Label>);
      expect(screen.getByText('Type Test').tagName).toBe('LABEL');
    });
  });
});
