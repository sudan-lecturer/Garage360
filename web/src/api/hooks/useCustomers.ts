import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface Customer {
  id: string;
  name: string;
  type: 'INDIVIDUAL' | 'ORGANISATION';
  email: string | null;
  phone: string | null;
  address: string | null;
  created_at: string;
  is_active: boolean;
}

interface CustomersResponse {
  data: Customer[];
  total: number;
  page: number;
  limit: number;
}

interface CustomerDetail extends Customer {
  vehicles?: VehicleSummary[];
  jobs?: JobSummary[];
  invoices?: InvoiceSummary[];
}

interface VehicleSummary {
  id: string;
  license_plate: string;
  make: string;
  model: string;
  year: number | null;
}

interface JobSummary {
  id: string;
  job_number: string;
  status: string;
  created_at: string;
}

interface InvoiceSummary {
  id: string;
  invoice_number: string;
  amount: number;
  status: string;
  created_at: string;
}

export function useCustomers(params?: {
  search?: string;
  type?: 'INDIVIDUAL' | 'ORGANISATION';
  page?: number;
  limit?: number;
}) {
  return useQuery({
    queryKey: ['customers', params],
    queryFn: async () => {
      const response = await api.get<CustomersResponse>('/v1/customers', { params });
      return response.data;
    },
  });
}

export function useCustomer(id: string) {
  return useQuery({
    queryKey: ['customer', id],
    queryFn: async () => {
      const response = await api.get<CustomerDetail>(`/v1/customers/${id}`);
      return response.data;
    },
    enabled: !!id,
  });
}

export function useCreateCustomer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: Partial<Customer>) => {
      const response = await api.post<Customer>('/v1/customers', data);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customers'] });
    },
  });
}

export function useUpdateCustomer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, ...data }: Partial<Customer> & { id: string }) => {
      const response = await api.put<Customer>(`/v1/customers/${id}`, data);
      return response.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['customers'] });
      queryClient.invalidateQueries({ queryKey: ['customer', variables.id] });
    },
  });
}

export function useDeleteCustomer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/v1/customers/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customers'] });
    },
  });
}