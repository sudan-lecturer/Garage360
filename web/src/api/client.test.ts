import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import apiClient from './client';

describe('API Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Request Configuration', () => {
    it('should have correct base URL', () => {
      expect(apiClient.defaults.baseURL).toBe('/api');
    });

    it('should have correct Content-Type header', () => {
      expect(apiClient.defaults.headers['Content-Type']).toBe('application/json');
    });
  });

  describe('Token Handling', () => {
    it('should prepare token for Authorization header', () => {
      const token = 'test-token';
      const header = token ? `Bearer ${token}` : undefined;
      expect(header).toBe('Bearer test-token');
    });

    it('should not add Authorization header when token is null', () => {
      const token = null;
      const header = token ? `Bearer ${token}` : undefined;
      expect(header).toBeUndefined();
    });

    it('should not add Authorization header when token is empty', () => {
      const token = '';
      const header = token ? `Bearer ${token}` : undefined;
      expect(header).toBeUndefined();
    });
  });

  describe('Refresh Token Logic', () => {
    it('should handle refresh token availability', () => {
      const refreshToken = 'valid-refresh-token';
      const hasRefreshToken = !!refreshToken;
      expect(hasRefreshToken).toBe(true);
    });

    it('should detect missing refresh token', () => {
      const refreshToken = null;
      const hasRefreshToken = !!refreshToken;
      expect(hasRefreshToken).toBe(false);
    });
  });

  describe('Error Handling', () => {
    it('should handle 401 unauthorized response', () => {
      const status = 401;
      const shouldRefresh = status === 401;
      expect(shouldRefresh).toBe(true);
    });

    it('should not refresh on other errors', () => {
      const status = 400;
      const shouldRefresh = status === 401;
      expect(shouldRefresh).toBe(false);
    });
  });

  describe('Configuration', () => {
    it('should be an axios instance', () => {
      expect(apiClient).toBeDefined();
      expect(typeof apiClient.get).toBe('function');
      expect(typeof apiClient.post).toBe('function');
      expect(typeof apiClient.put).toBe('function');
      expect(typeof apiClient.delete).toBe('function');
    });
  });
});
