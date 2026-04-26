import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface UserRecord {
  id: string;
  email: string;
  name: string;
  role: string;
  isActive: boolean;
  lastLoginAt?: string;
  createdAt: string;
}

export interface CreateUserPayload {
  email: string;
  name: string;
  role: string;
  password: string;
  isActive: boolean;
}

export interface UpdateUserPayload {
  email: string;
  name: string;
  role: string;
  password?: string;
  isActive: boolean;
}

export interface WorkshopProfilePayload {
  name?: string;
  address?: string;
  phone?: string;
  email?: string;
  logoUrl?: string;
  taxId?: string;
  timezone?: string;
  currencyCode?: string;
  currencySymbol?: string;
}

export interface LocationPayload {
  name: string;
  address?: string;
  isPrimary?: boolean;
  isActive?: boolean;
}

export interface FeatureFlagRecord {
  key: string;
  description?: string;
  defaultEnabled: boolean;
  isEnabled: boolean;
  hasOverride: boolean;
}

export interface NotificationPrefs {
  jobUpdatesEmail: boolean;
  jobUpdatesSms: boolean;
  pushEnabled: boolean;
  lowStockAlerts: boolean;
  approvalAlerts: boolean;
  dailySummaryEmail?: string;
}

export function useSettingsUsers() {
  return useQuery({
    queryKey: ['settings', 'users'],
    queryFn: async () => {
      const response = await api.get('/v1/settings/users');
      const data = response.data as Array<{
        id: string;
        email: string;
        name: string;
        role: string;
        is_active: boolean;
        last_login_at?: string;
        created_at: string;
      }>;
      return data.map((user) => ({
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        isActive: user.is_active,
        lastLoginAt: user.last_login_at,
        createdAt: user.created_at,
      })) as UserRecord[];
    },
  });
}

export function useCreateSettingsUser() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (payload: CreateUserPayload) => {
      const response = await api.post('/v1/settings/users', {
        email: payload.email,
        name: payload.name,
        role: payload.role,
        password: payload.password,
        is_active: payload.isActive,
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings', 'users'] });
    },
  });
}

export function useUpdateSettingsUser() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateUserPayload }) => {
      const response = await api.put(`/v1/settings/users/${id}`, {
        email: data.email,
        name: data.name,
        role: data.role,
        password: data.password,
        is_active: data.isActive,
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings', 'users'] });
    },
  });
}

export function useDeleteSettingsUser() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/v1/settings/users/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings', 'users'] });
    },
  });
}

export function useWorkshopProfile() {
  return useQuery({
    queryKey: ['settings', 'profile'],
    queryFn: async () => {
      const response = await api.get('/v1/settings/profile');
      const data = response.data as {
        name?: string;
        address?: string;
        phone?: string;
        email?: string;
        logo_url?: string;
        tax_id?: string;
        timezone?: string;
        currency_code?: string;
        currency_symbol?: string;
      };
      return {
        name: data.name,
        address: data.address,
        phone: data.phone,
        email: data.email,
        logoUrl: data.logo_url,
        taxId: data.tax_id,
        timezone: data.timezone,
        currencyCode: data.currency_code,
        currencySymbol: data.currency_symbol,
      } as WorkshopProfilePayload;
    },
  });
}

export function useUpdateWorkshopProfile() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (payload: WorkshopProfilePayload) => {
      const response = await api.put('/v1/settings/profile', {
        name: payload.name,
        address: payload.address,
        phone: payload.phone,
        email: payload.email,
        logo_url: payload.logoUrl,
        tax_id: payload.taxId,
        timezone: payload.timezone,
        currency_code: payload.currencyCode,
        currency_symbol: payload.currencySymbol,
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings', 'profile'] });
    },
  });
}

export function useLocations() {
  return useQuery({
    queryKey: ['settings', 'locations'],
    queryFn: async () => {
      const response = await api.get('/v1/settings/locations');
      const data = response.data as Array<{
        id: string;
        name: string;
        address?: string;
        is_primary: boolean;
        is_active: boolean;
      }>;
      return data.map((location) => ({
        id: location.id,
        name: location.name,
        address: location.address,
        isPrimary: location.is_primary,
        isActive: location.is_active,
      }));
    },
  });
}

export function useCreateLocation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (payload: LocationPayload) => {
      const response = await api.post('/v1/settings/locations', {
        name: payload.name,
        address: payload.address,
        is_primary: payload.isPrimary,
        is_active: payload.isActive,
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings', 'locations'] });
    },
  });
}

export function useFeatureFlags() {
  return useQuery({
    queryKey: ['settings', 'feature-flags'],
    queryFn: async () => {
      const response = await api.get('/v1/settings/feature-flags');
      const data = response.data as Array<{
        key: string;
        description?: string;
        default_enabled: boolean;
        is_enabled: boolean;
        has_override: boolean;
      }>;
      return data.map((flag) => ({
        key: flag.key,
        description: flag.description,
        defaultEnabled: flag.default_enabled,
        isEnabled: flag.is_enabled,
        hasOverride: flag.has_override,
      }));
    },
  });
}

export function useUpdateFeatureFlag() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ key, isEnabled }: { key: string; isEnabled: boolean }) => {
      const response = await api.put(`/v1/settings/feature-flags/${key}`, { is_enabled: isEnabled });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings', 'feature-flags'] });
    },
  });
}

export function useNotificationPreferences() {
  return useQuery({
    queryKey: ['settings', 'notifications'],
    queryFn: async () => {
      const response = await api.get('/v1/settings/notification-preferences');
      const data = response.data as {
        job_updates_email: boolean;
        job_updates_sms: boolean;
        push_enabled: boolean;
        low_stock_alerts: boolean;
        approval_alerts: boolean;
        daily_summary_email?: string;
      };
      return {
        jobUpdatesEmail: data.job_updates_email,
        jobUpdatesSms: data.job_updates_sms,
        pushEnabled: data.push_enabled,
        lowStockAlerts: data.low_stock_alerts,
        approvalAlerts: data.approval_alerts,
        dailySummaryEmail: data.daily_summary_email,
      };
    },
  });
}

export function useUpdateNotificationPreferences() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (payload: NotificationPrefs) => {
      const response = await api.put('/v1/settings/notification-preferences', {
        job_updates_email: payload.jobUpdatesEmail,
        job_updates_sms: payload.jobUpdatesSms,
        push_enabled: payload.pushEnabled,
        low_stock_alerts: payload.lowStockAlerts,
        approval_alerts: payload.approvalAlerts,
        daily_summary_email: payload.dailySummaryEmail,
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings', 'notifications'] });
    },
  });
}
