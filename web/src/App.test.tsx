import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import App from './App';
import { useAuthStore } from '@/store/auth';

vi.mock('@/store/auth');

const mockedUseAuthStore = vi.mocked(useAuthStore);

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

describe('App Routing', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Protected Routes - Unauthenticated', () => {
    it('should show login form when accessing /dashboard without auth', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
        user: null,
        accessToken: null,
        refreshToken: null,
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/dashboard']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByPlaceholderText('admin@demo.com')).toBeInTheDocument();
      });
    });

    it('should show login form when accessing /customers without auth', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/customers']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByPlaceholderText('admin@demo.com')).toBeInTheDocument();
      });
    });

    it('should show login form when accessing /jobs without auth', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/jobs']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByPlaceholderText('admin@demo.com')).toBeInTheDocument();
      });
    });

    it('should show login form when accessing /inventory without auth', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/inventory']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByPlaceholderText('admin@demo.com')).toBeInTheDocument();
      });
    });
  });

  describe('Protected Routes - Authenticated', () => {
    beforeEach(() => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: true,
        user: { id: '1', email: 'a@b.com', name: 'Test', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      } as unknown as ReturnType<typeof useAuthStore>);
    });

    it('should render dashboard when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/dashboard']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /dashboard/i })).toBeInTheDocument();
      });
    });

    it('should render customers when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/customers']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /customers/i })).toBeInTheDocument();
      });
    });

    it('should render vehicles when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/vehicles']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /vehicles/i })).toBeInTheDocument();
      });
    });

    it('should render jobs when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/jobs']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /jobs/i })).toBeInTheDocument();
      });
    });

    it('should render inventory when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/inventory']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /inventory/i })).toBeInTheDocument();
      });
    });

    it('should render purchases when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/purchases']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /purchase orders/i })).toBeInTheDocument();
      });
    });

    it('should render billing when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/billing']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /invoices/i })).toBeInTheDocument();
      });
    });

    it('should render dvi when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/dvi/templates']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /dvi templates/i })).toBeInTheDocument();
      });
    });

    it('should render assets when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/assets']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /assets/i })).toBeInTheDocument();
      });
    });

    it('should render hr when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/hr/employees']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /employees/i })).toBeInTheDocument();
      });
    });

    it('should render reports when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/reports']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /reports/i })).toBeInTheDocument();
      });
    });

    it('should render create purchase order route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/purchases/new']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /create purchase order/i })).toBeInTheDocument();
      });
    });

    it('should render quote creation route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/jobs/1/approve']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /quote creation/i })).toBeInTheDocument();
      });
    });

    it('should render user management route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/settings/users']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /user management/i })).toBeInTheDocument();
      });
    });

    it('should render role management route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/settings/roles']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /role management/i })).toBeInTheDocument();
      });
    });

    it('should render inventory detail route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/inventory/1']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /inventory detail/i })).toBeInTheDocument();
      });
    });

    it('should render invoice detail route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/billing/1']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /invoice/i })).toBeInTheDocument();
      });
    });

    it('should render dvi result entry route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/dvi/results/new']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /dvi result entry/i })).toBeInTheDocument();
      });
    });

    it('should render dvi result detail route when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/dvi/results/1']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /dvi result detail/i })).toBeInTheDocument();
      });
    });

    it('should render settings when authenticated', async () => {
      render(
        <MemoryRouter initialEntries={['/settings']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /settings/i })).toBeInTheDocument();
      });
    });
  });

  describe('Auth Routes', () => {
    it('should render login page when not authenticated', () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/login']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    });

    it('should redirect forgot password page to login', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/forgot-password']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      });
    });

    it('should allow access to login when not authenticated', () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/login']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
    });
  });

  describe('Default Route', () => {
    it('should render dashboard when accessing root and authenticated', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: true,
        user: { id: '1', email: 'a@b.com', name: 'Test', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByRole('heading', { name: /dashboard/i })).toBeInTheDocument();
      });
    });
  });

  describe('404 Handling', () => {
    it('should redirect unknown routes to / when not authenticated', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      } as unknown as ReturnType<typeof useAuthStore>);

      render(
        <MemoryRouter initialEntries={['/unknown-route-xyz']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByPlaceholderText('admin@demo.com')).toBeInTheDocument();
      });
    });
  });
});
