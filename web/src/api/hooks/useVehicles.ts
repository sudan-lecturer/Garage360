import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface Vehicle {
  id: string;
  registration_no: string;
  brand: string;
  model: string;
  year: number | null;
  color: string | null;
  vin: string | null;
  customer_id: string;
  customer_name: string;
  last_service_at?: string | null;
  photo_path?: string | null;
  created_at: string;
  is_active: boolean;
}

interface VehiclesResponse {
  data: Vehicle[];
  total: number;
  page: number;
  limit: number;
}

interface VehicleDetail extends Vehicle {
  jobs?: JobSummary[];
}

interface JobSummary {
  id: string;
  job_number: string;
  status: string;
  created_at: string;
}

export function useVehicles(params?: {
  search?: string;
  customer_id?: string;
  page?: number;
  limit?: number;
}) {
  return useQuery({
    queryKey: ['vehicles', params],
    queryFn: async () => {
      const response = await api.get('/v1/vehicles', { params });
      const payload = response.data as {
        data: Array<{
          id: string;
          registrationNo?: string;
          registration_no?: string;
          make?: string;
          brand?: string;
          model: string;
          year?: number | null;
          color?: string | null;
          vin?: string | null;
          customerId?: string;
          customer_id?: string;
          customerName?: string;
          customer_name?: string;
          lastServiceAt?: string | null;
          last_service_at?: string | null;
          photoPath?: string | null;
          photo_path?: string | null;
          createdAt?: string;
          created_at?: string;
          isActive?: boolean;
          is_active?: boolean;
        }>;
        meta?: { page: number; limit: number; total: number };
      };
      return {
        data: (payload.data ?? []).map((v) => ({
          id: v.id,
          registration_no: v.registrationNo ?? v.registration_no ?? '',
          brand: v.brand ?? v.make ?? '',
          model: v.model ?? '',
          year: v.year ?? null,
          color: v.color ?? null,
          vin: v.vin ?? null,
          customer_id: v.customerId ?? v.customer_id ?? '',
          customer_name: v.customerName ?? v.customer_name ?? '',
          last_service_at: v.lastServiceAt ?? v.last_service_at ?? null,
          photo_path: v.photoPath ?? v.photo_path ?? null,
          created_at: v.createdAt ?? v.created_at ?? '',
          is_active: v.isActive ?? v.is_active ?? true,
        })),
        page: payload.meta?.page ?? 1,
        limit: payload.meta?.limit ?? 20,
        total: payload.meta?.total ?? 0,
      } as VehiclesResponse;
    },
  });
}

export function useVehicle(id: string) {
  return useQuery({
    queryKey: ['vehicle', id],
    queryFn: async () => {
      const response = await api.get(`/v1/vehicles/${id}`);
      const v = response.data as any;
      return {
        id: v.id,
        registration_no: v.registrationNo ?? v.registration_no ?? '',
        brand: v.brand ?? v.make ?? '',
        model: v.model ?? '',
        year: v.year ?? null,
        color: v.color ?? null,
        vin: v.vin ?? null,
        customer_id: v.customerId ?? v.customer_id ?? '',
        customer_name: v.customerName ?? v.customer_name ?? '',
        last_service_at: v.lastServiceAt ?? v.last_service_at ?? null,
        photo_path: v.photoPath ?? v.photo_path ?? null,
        created_at: v.createdAt ?? v.created_at ?? '',
        is_active: v.isActive ?? v.is_active ?? true,
      } as VehicleDetail;
    },
    enabled: !!id,
  });
}

export function useCreateVehicle() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: Partial<Vehicle>) => {
      const response = await api.post<Vehicle>('/v1/vehicles', data);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vehicles'] });
    },
  });
}

export function useUpdateVehicle() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, ...data }: Partial<Vehicle> & { id: string }) => {
      const response = await api.put<Vehicle>(`/v1/vehicles/${id}`, data);
      return response.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['vehicles'] });
      queryClient.invalidateQueries({ queryKey: ['vehicle', variables.id] });
    },
  });
}

export function useDeleteVehicle() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/v1/vehicles/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vehicles'] });
    },
  });
}