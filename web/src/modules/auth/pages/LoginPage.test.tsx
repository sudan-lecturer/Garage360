import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { LoginPage } from './LoginPage';

vi.mock('@/store/auth', () => ({
  useAuthStore: vi.fn().mockReturnValue({
    login: vi.fn(),
    logout: vi.fn(),
    user: null,
    accessToken: null,
    refreshToken: null,
    isAuthenticated: false,
    setTokens: vi.fn(),
  }),
}));

vi.mock('@/api/client', () => ({
  default: {
    post: vi.fn(),
  },
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

describe('LoginPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render email input', () => {
      render(
        <MemoryRouter>
          <LoginPage />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );
      
      expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    });

    it('should render password input', () => {
      render(
        <MemoryRouter>
          <LoginPage />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );
      
      expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    });

    it('should render submit button', () => {
      render(
        <MemoryRouter>
          <LoginPage />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );
      
      expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
    });

    it('should render forgot password link', () => {
      render(
        <MemoryRouter>
          <LoginPage />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );
      
      expect(screen.getByText(/forgot password/i)).toBeInTheDocument();
    });

    it('should have correct placeholders', () => {
      render(
        <MemoryRouter>
          <LoginPage />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );
      
      expect(screen.getByPlaceholderText('admin@garage360.io')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('••••••••')).toBeInTheDocument();
    });
  });

  describe('Navigation', () => {
    it('should have forgot password link with href', () => {
      render(
        <MemoryRouter>
          <LoginPage />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      const link = screen.getByText(/forgot password/i);
      expect(link).toHaveAttribute('href', '/forgot-password');
    });
  });

  describe('Form Structure', () => {
    it('should have form element with submit button', () => {
      render(
        <MemoryRouter>
          <LoginPage />
        </MemoryRouter>,
        { wrapper: createWrapper() }
      );

      expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
    });
  });
});
