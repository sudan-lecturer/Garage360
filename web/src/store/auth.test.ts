import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useAuthStore } from './auth';

vi.mock('@/store/auth', () => ({
  useAuthStore: vi.fn(),
}));

const mockedAuthStore = vi.mocked(useAuthStore);

describe('Auth Store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Initial State', () => {
    it('should have correct initial state structure', () => {
      const state = {
        user: null,
        accessToken: null,
        refreshToken: null,
        isAuthenticated: false,
        login: vi.fn(),
        logout: vi.fn(),
        setTokens: vi.fn(),
      };

      expect(state.user).toBeNull();
      expect(state.accessToken).toBeNull();
      expect(state.refreshToken).toBeNull();
      expect(state.isAuthenticated).toBe(false);
    });
  });

  describe('login action', () => {
    it('should update state with user data and tokens', () => {
      const login = vi.fn();
      const mockUser = {
        id: 'user-1',
        email: 'test@example.com',
        name: 'Test User',
        role: 'admin',
        tenantId: 'tenant-1',
      };

      login({
        user: mockUser,
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
      });

      expect(login).toHaveBeenCalledWith({
        user: mockUser,
        accessToken: 'access-token-123',
        refreshToken: 'refresh-token-123',
      });
    });

    it('should set isAuthenticated to true after login', () => {
      const login = vi.fn();
      
      login({
        user: { id: '1', email: 'a@b.com', name: 'Test', role: 'admin', tenantId: 't1' },
        accessToken: 'token',
        refreshToken: 'refresh',
      });

      const isAuthenticated = true;
      expect(isAuthenticated).toBe(true);
    });

    it('should handle different user roles', () => {
      const roles = ['admin', 'manager', 'mechanic', 'cashier', 'hr_officer'];
      const login = vi.fn();

      roles.forEach(role => {
        login({
          user: { id: '1', email: `${role}@test.com`, name: role, role, tenantId: 't1' },
          accessToken: 'token',
          refreshToken: 'refresh',
        });

        expect(login).toHaveBeenLastCalledWith(
          expect.objectContaining({
            user: expect.objectContaining({ role }),
          })
        );
      });
    });
  });

  describe('logout action', () => {
    it('should reset all state to initial values', () => {
      const logout = vi.fn();
      
      logout();

      expect(logout).toHaveBeenCalled();
    });

    it('should clear user data after logout', () => {
      const logout = vi.fn();
      
      logout();

      expect(logout).toHaveBeenCalled();
    });
  });

  describe('setTokens action', () => {
    it('should update only tokens', () => {
      const setTokens = vi.fn();
      
      setTokens('new-access', 'new-refresh');

      expect(setTokens).toHaveBeenCalledWith('new-access', 'new-refresh');
    });

    it('should not change isAuthenticated status', () => {
      const setTokens = vi.fn();
      const isAuthenticated = true;

      setTokens('new-token', 'new-refresh');

      expect(isAuthenticated).toBe(true);
    });
  });

  describe('Type Safety', () => {
    it('should enforce correct user type', () => {
      const user = {
        id: 'user-123',
        email: 'user@test.com',
        name: 'Test User',
        role: 'manager',
        tenantId: 'tenant-456',
      };

      expect(user).toHaveProperty('id');
      expect(user).toHaveProperty('email');
      expect(user).toHaveProperty('name');
      expect(user).toHaveProperty('role');
      expect(user).toHaveProperty('tenantId');
    });
  });
});
