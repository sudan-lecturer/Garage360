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

export interface PurchaseOrderItem {
  id: string;
  purchaseOrderId: string;
  inventoryItemId: string | null;
  inventoryItemName: string | null;
  description: string;
  quantity: string;
  unitPrice: string;
  receivedQty: string;
  createdAt: string;
}

export interface GoodsReceiptSummary {
  id: string;
  grnNo: number | null;
  purchaseOrderId: string;
  status: string;
  receivedBy: string | null;
  receivedByName: string | null;
  receivedAt: string | null;
  createdAt: string;
}

export interface PurchaseOrderDetail {
  id: string;
  poNo: number | null;
  supplierId: string;
  supplierName: string;
  status: string;
  expectedDelivery: string | null;
  subtotal: string;
  discountPct: string;
  taxAmount: string;
  totalAmount: string;
  notes: string | null;
  currency: string;
  exchangeRate: string | null;
  createdBy: string | null;
  createdAt: string;
  items: PurchaseOrderItem[];
  grns: GoodsReceiptSummary[];
}

interface PurchaseStatusHistory {
  id: string;
  fromStatus: string | null;
  toStatus: string;
  notes: string | null;
  changedBy: string | null;
  changedByName: string | null;
  createdAt: string;
}

interface PurchaseApproval {
  id: string;
  approvedBy: string | null;
  approvedByName: string | null;
  isApproved: boolean;
  notes: string | null;
  createdAt: string;
}

export interface PurchaseHistory {
  statusHistory: PurchaseStatusHistory[];
  approvals: PurchaseApproval[];
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

export function usePurchaseOrder(id: string) {
  return useQuery({
    queryKey: ['purchase-order', id],
    queryFn: async () => {
      const response = await api.get<PurchaseOrderDetail>(`/v1/purchases/${id}`);
      return response.data;
    },
    enabled: !!id,
  });
}

export function usePurchaseOrderHistory(id: string) {
  return useQuery({
    queryKey: ['purchase-order-history', id],
    queryFn: async () => {
      const response = await api.get<PurchaseHistory>(`/v1/purchases/${id}/history`);
      return response.data;
    },
    enabled: !!id,
  });
}
