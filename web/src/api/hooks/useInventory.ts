import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface InventoryItem {
  id: string;
  sku: string;
  name: string;
  description?: string | null;
  category?: string | null;
  unit: string;
  cost_price: string;
  sell_price: string;
  min_stock_level: number;
  current_quantity: string;
  is_active: boolean;
}

export interface InventoryItemRequest {
  sku: string;
  name: string;
  description?: string;
  category?: string;
  unit: string;
  cost_price: string;
  sell_price: string;
  min_stock_level: number;
}

export function useInventoryList(params?: { search?: string; category?: string; page?: number; limit?: number }) {
  return useQuery({
    queryKey: ['inventory', params],
    queryFn: async () => {
      const response = await api.get('/v1/inventory', { params });
      return response.data as { data: InventoryItem[]; total: number; page: number; limit: number };
    },
  });
}

export function useInventoryItem(id?: string) {
  return useQuery({
    queryKey: ['inventory', 'detail', id],
    enabled: Boolean(id),
    queryFn: async () => {
      const response = await api.get(`/v1/inventory/${id}`);
      return response.data as { item: InventoryItem };
    },
  });
}

export function useCreateInventoryItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (payload: InventoryItemRequest) => {
      const response = await api.post('/v1/inventory', payload);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['inventory'] });
    },
  });
}

export function useUpdateInventoryItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, payload }: { id: string; payload: InventoryItemRequest }) => {
      const response = await api.put(`/v1/inventory/${id}`, payload);
      return response.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['inventory'] });
      queryClient.invalidateQueries({ queryKey: ['inventory', 'detail', variables.id] });
    },
  });
}

export function useAdjustInventoryStock() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      adjustmentType,
      quantity,
      reason,
    }: {
      id: string;
      adjustmentType: 'ADD' | 'REMOVE' | 'SET' | 'COUNT';
      quantity: string;
      reason?: string;
    }) => {
      const response = await api.post(`/v1/inventory/${id}/adjust`, {
        adjustment_type: adjustmentType,
        quantity,
        reason,
      });
      return response.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['inventory'] });
      queryClient.invalidateQueries({ queryKey: ['inventory', 'detail', variables.id] });
    },
  });
}
