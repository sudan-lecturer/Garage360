import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Input } from './input';

describe('Input', () => {
  describe('Rendering', () => {
    it('should render input element', () => {
      render(<Input />);
      expect(screen.getByRole('textbox')).toBeInTheDocument();
    });

    it('should render with placeholder', () => {
      render(<Input placeholder="Enter text" />);
      expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument();
    });
  });

  describe('States', () => {
    it('should be disabled when disabled prop is true', () => {
      render(<Input disabled />);
      expect(screen.getByRole('textbox')).toBeDisabled();
    });

    it('should have disabled styling', () => {
      render(<Input disabled />);
      expect(screen.getByRole('textbox')).toHaveClass('disabled:opacity-50');
    });

    it('should have focus ring styling', () => {
      render(<Input />);
      expect(screen.getByRole('textbox')).toHaveClass('focus-visible:ring-2');
    });
  });

  describe('Props forwarding', () => {
    it('should forward ref', () => {
      let ref: HTMLInputElement | null = null;
      render(<Input ref={(el) => { ref = el; }} />);
      expect(ref).toBeInstanceOf(HTMLInputElement);
    });

    it('should merge className', () => {
      render(<Input className="custom-class" />);
      expect(screen.getByRole('textbox')).toHaveClass('custom-class');
    });

    it('should forward id attribute', () => {
      render(<Input id="email-input" />);
      expect(screen.getByRole('textbox')).toHaveAttribute('id', 'email-input');
    });

    it('should forward name attribute', () => {
      render(<Input name="email" />);
      expect(screen.getByRole('textbox')).toHaveAttribute('name', 'email');
    });

    it('should forward autoComplete attribute', () => {
      render(<Input autoComplete="email" />);
      expect(screen.getByRole('textbox')).toHaveAttribute('autoComplete', 'email');
    });
  });

  describe('Styling', () => {
    it('should have base styling classes', () => {
      render(<Input />);
      expect(screen.getByRole('textbox')).toHaveClass('flex', 'h-10', 'w-full');
    });

    it('should have border styling', () => {
      render(<Input />);
      expect(screen.getByRole('textbox')).toHaveClass('border');
    });

    it('should have placeholder styling', () => {
      render(<Input placeholder="Test" />);
      expect(screen.getByRole('textbox')).toHaveClass('placeholder:text-muted-foreground');
    });
  });
});
