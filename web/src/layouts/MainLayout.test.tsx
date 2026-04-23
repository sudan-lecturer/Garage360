import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MainLayout } from './MainLayout';

vi.mock('@/store/auth', () => ({
  useAuthStore: vi.fn().mockReturnValue({
    user: {
      id: 'user-1',
      email: 'test@example.com',
      name: 'Test User',
      role: 'admin',
      tenantId: 'tenant-1',
    },
    logout: vi.fn(),
    login: vi.fn(),
    accessToken: 'token',
    refreshToken: 'refresh',
    isAuthenticated: true,
    setTokens: vi.fn(),
  }),
}));

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

describe('MainLayout', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Header', () => {
    it('should render Garage360 branding in header', () => {
      render(
        <MemoryRouter>
          <MainLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByText('Garage360')).toBeInTheDocument();
    });

    it('should display user name in header', () => {
      render(
        <MemoryRouter>
          <MainLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByText('Test User')).toBeInTheDocument();
    });
  });

  describe('Sidebar Navigation', () => {
    it('should render Dashboard navigation item', () => {
      render(
        <MemoryRouter>
          <MainLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByText('Dashboard')).toBeInTheDocument();
    });

    it('should render Customers navigation item', () => {
      render(
        <MemoryRouter>
          <MainLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByText('Customers')).toBeInTheDocument();
    });

    it('should render all 12 navigation items', () => {
      render(
        <MemoryRouter>
          <MainLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const navItems = [
        'Dashboard',
        'Customers',
        'Vehicles',
        'Jobs',
        'Inventory',
        'Purchases',
        'Billing',
        'DVI',
        'Assets',
        'HR',
        'Reports',
        'Settings',
      ];

      navItems.forEach(item => {
        expect(screen.getByText(item)).toBeInTheDocument();
      });
    });

    it('should have correct navigation links', () => {
      render(
        <MemoryRouter>
          <MainLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const dashboardLink = screen.getByText('Dashboard').closest('a');
      expect(dashboardLink).toHaveAttribute('href', '/dashboard');
    });
  });

  describe('Active Route Highlighting', () => {
    it('should highlight Dashboard when on /dashboard', () => {
      render(
        <MemoryRouter initialEntries={['/dashboard']}>
          <MainLayout />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const dashboardLink = screen.getByText('Dashboard').closest('a');
      expect(dashboardLink).toHaveClass('bg-accent', 'text-accent-foreground');
    });
  });
});
