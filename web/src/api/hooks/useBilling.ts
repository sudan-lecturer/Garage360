import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface InvoiceLineItemRequest {
  description: string;
  quantity: number;
  unitPrice: number;
  discountPct?: number;
}

export interface CreateInvoiceRequest {
  jobCardId?: string;
  customerId: string;
  discountPct?: number;
  taxAmount?: number;
  notes?: string;
  lineItems: InvoiceLineItemRequest[];
}

export function useInvoices(params?: { page?: number; limit?: number; search?: string; status?: string }) {
  return useQuery({
    queryKey: ['billing', 'invoices', params],
    queryFn: async () => {
      const response = await api.get('/v1/invoices', { params });
      return response.data;
    },
  });
}

export function useCreateInvoice() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (payload: CreateInvoiceRequest) => {
      const response = await api.post('/v1/invoices', payload);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['billing', 'invoices'] });
    },
  });
}

export function useInvoiceDetail(id?: string) {
  return useQuery({
    queryKey: ['billing', 'invoice', id],
    enabled: Boolean(id),
    queryFn: async () => {
      const response = await api.get(`/v1/invoices/${id}`);
      const data = response.data as {
        id: string;
        invoice_no?: number | null;
        status: string;
        customer_name: string;
        total_amount: string;
        amount_paid: string;
        balance_due: string;
        line_items: Array<{
          id: string;
          description: string;
          quantity: string;
          unit_price: string;
          line_total: string;
        }>;
      };
      return {
        id: data.id,
        invoiceNo: data.invoice_no,
        status: data.status,
        customerName: data.customer_name,
        totalAmount: data.total_amount,
        amountPaid: data.amount_paid,
        balanceDue: data.balance_due,
        lineItems: (data.line_items ?? []).map((line) => ({
          id: line.id,
          description: line.description,
          quantity: line.quantity,
          unitPrice: line.unit_price,
          lineTotal: line.line_total,
        })),
      };
    },
  });
}

export function useRecordInvoicePayment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      amount,
      paymentMethod,
      notes,
    }: {
      id: string;
      amount: number;
      paymentMethod?: string;
      notes?: string;
    }) => {
      const response = await api.post(`/v1/invoices/${id}/payment`, {
        amount,
        paymentMethod,
        notes,
      });
      return response.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['billing', 'invoices'] });
      queryClient.invalidateQueries({ queryKey: ['billing', 'invoice', variables.id] });
    },
  });
}
