import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface Vehicle {
  id: string;
  license_plate: string;
  make: string;
  model: string;
  year: number | null;
  color: string | null;
  vin: string | null;
  customer_id: string;
  customer_name: string;
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
      const response = await api.get<VehiclesResponse>('/v1/vehicles', { params });
      return response.data;
    },
  });
}

export function useVehicle(id: string) {
  return useQuery({
    queryKey: ['vehicle', id],
    queryFn: async () => {
      const response = await api.get<VehicleDetail>(`/v1/vehicles/${id}`);
      return response.data;
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