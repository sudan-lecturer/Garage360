import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import App from '@/App';
import { useAuthStore } from '@/store/auth';

vi.mock('@/store/auth', () => ({
  useAuthStore: vi.fn(),
}));

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

describe('Full Auth Flow Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Login Flow', () => {
    it('should show login form on /login route', () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
        user: null,
        accessToken: null,
        refreshToken: null,
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      });

      render(
        <MemoryRouter initialEntries={['/login']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    });

    it('should pre-fill placeholder text', () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      });

      render(
        <MemoryRouter initialEntries={['/login']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByPlaceholderText('admin@demo.com')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('••••••••')).toBeInTheDocument();
    });
  });

  describe('Logout Flow', () => {
    it('should show logout button when authenticated', async () => {
      const mockLogout = vi.fn();
      
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: true,
        user: { id: '1', email: 'a@b.com', name: 'Test', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
        logout: mockLogout,
        login: vi.fn(),
        setTokens: vi.fn(),
      });

      render(
        <MemoryRouter initialEntries={['/dashboard']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        const buttons = screen.getAllByRole('button');
        const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'));
        expect(logoutButton).toBeInTheDocument();
      });
    });
  });

  describe('Route Protection', () => {
    it('should render login page for /dashboard when not authenticated', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
        user: null,
        accessToken: null,
        refreshToken: null,
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      });

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

    it('should render login page for /customers when not authenticated', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: false,
      });

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
  });

  describe('Protected Route Access', () => {
    it('should render dashboard when authenticated', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: true,
        user: { id: '1', email: 'a@b.com', name: 'Test', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      });

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
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: true,
        user: { id: '1', email: 'a@b.com', name: 'Test', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      });

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
  });

  describe('Navigation', () => {
    it('should show navigation items when authenticated', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: true,
        user: { id: '1', email: 'a@b.com', name: 'Test', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      });

      render(
        <MemoryRouter initialEntries={['/dashboard']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getAllByText('Dashboard').length).toBeGreaterThan(0);
        expect(screen.getByText('Settings')).toBeInTheDocument();
      });
    });

    it('should show user name in header', async () => {
      mockedUseAuthStore.mockReturnValue({
        isAuthenticated: true,
        user: { id: '1', email: 'a@b.com', name: 'Test User', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      });

      render(
        <MemoryRouter initialEntries={['/dashboard']}>
          <App />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(screen.getByText('Test User')).toBeInTheDocument();
      });
    });
  });
});
