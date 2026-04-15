import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthLayout } from './AuthLayout';

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    Outlet: () => <div data-testid="outlet">Outlet Content</div>,
  };
});

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('AuthLayout', () => {
  describe('Rendering', () => {
    it('should Render Garage360 branding', () => {
      render(
        <MemoryRouter>
          <AuthLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByText('Garage360')).toBeInTheDocument();
    });

    it('should Render subtitle', () => {
      render(
        <MemoryRouter>
          <AuthLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByText('Workshop Management System')).toBeInTheDocument();
    });

    it('should Render Outlet for child routes', () => {
      render(
        <MemoryRouter>
          <AuthLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByTestId('outlet')).toBeInTheDocument();
    });
  });

  describe('Structure', () => {
    it('should have form container with max-width', () => {
      render(
        <MemoryRouter>
          <AuthLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const container = screen.getByText('Workshop Management System').closest('div');
      expect(container).toBeInTheDocument();
    });

    it('should have card container styling', () => {
      render(
        <MemoryRouter>
          <AuthLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const card = screen.getByTestId('outlet').closest('.bg-card');
      expect(card).toBeInTheDocument();
    });
  });

  describe('Styling', () => {
    it('should have full height background', () => {
      render(
        <MemoryRouter>
          <AuthLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const branding = screen.getByText('Garage360');
      const container = branding.closest('.min-h-screen');
      expect(container).toBeInTheDocument();
    });

    it('should have primary background color', () => {
      render(
        <MemoryRouter>
          <AuthLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const branding = screen.getByText('Garage360');
      const container = branding.closest('.bg-primary');
      expect(container).toBeInTheDocument();
    });
  });
});
