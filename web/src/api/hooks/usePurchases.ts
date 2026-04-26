import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface PurchaseOrderSummary {
  id: string;
  po_no: number | null;
  supplier_id: string;
  supplier_name: string;
  status: string;
  total_amount: string;
  expected_delivery: string | null;
  created_at: string;
}

interface PurchaseListResponse {
  data: PurchaseOrderSummary[];
  total: number;
  page: number;
  limit: number;
}

export interface CreatePurchaseOrderPayload {
  supplierId: string;
  expectedDelivery?: string;
  notes?: string;
  items: Array<{
    description: string;
    quantity: number;
    unitPrice: number;
  }>;
}

export function usePurchaseOrders(params?: { search?: string; status?: string; page?: number; limit?: number }) {
  return useQuery({
    queryKey: ['purchases', params],
    queryFn: async () => {
      const response = await api.get<PurchaseListResponse>('/v1/purchases', { params });
      return response.data;
    },
  });
}

export function useCreatePurchaseOrder() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (payload: CreatePurchaseOrderPayload) => {
      const response = await api.post('/v1/purchases', payload);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['purchases'] });
    },
  });
}
