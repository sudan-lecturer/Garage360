import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface Customer {
  id: string;
  name: string;
  type: 'INDIVIDUAL' | 'ORGANIZATION';
  email: string | null;
  phone: string | null;
  address: string | null;
  created_at: string;
  is_active?: boolean;
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
  tier?: string;
  financialSnapshot?: {
    totalInvoices: number;
    totalSpend: string;
    outstandingBalance: string;
    paidInvoices: number;
    lastInvoiceAt?: string | null;
  };
  serviceChronicle?: Array<{
    id: string;
    kind: string;
    referenceNo: string;
    status: string;
    occurredAt: string;
    summary: string;
  }>;
}

interface VehicleSummary {
  id: string;
  registration_no: string;
  brand: string;
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
  invoice_no: number | null;
  total_amount: string;
  status: string;
  created_at: string;
}

export function useCustomers(params?: {
  search?: string;
  customer_type?: 'INDIVIDUAL' | 'ORGANIZATION' | 'BOTH';
  page?: number;
  limit?: number;
}) {
  return useQuery({
    queryKey: ['customers', params],
    queryFn: async () => {
      const response = await api.get('/v1/customers', { params });
      const payload = response.data as {
        data: Array<{
          id: string;
          name: string;
          customerType?: string;
          customer_type?: string;
          email?: string | null;
          phone?: string | null;
          address?: string | null;
          createdAt?: string;
          created_at?: string;
          isActive?: boolean;
          is_active?: boolean;
        }>;
        meta?: { page: number; limit: number; total: number };
      };

      return {
        data: (payload.data ?? []).map((customer) => ({
          id: customer.id,
          name: customer.name,
          type: ((customer.customerType ?? customer.customer_type ?? 'INDIVIDUAL').toUpperCase() as 'INDIVIDUAL' | 'ORGANIZATION'),
          email: customer.email ?? null,
          phone: customer.phone ?? null,
          address: customer.address ?? null,
          created_at: customer.createdAt ?? customer.created_at ?? '',
          is_active: customer.isActive ?? customer.is_active,
        })),
        page: payload.meta?.page ?? 1,
        limit: payload.meta?.limit ?? 20,
        total: payload.meta?.total ?? 0,
      } as CustomersResponse;
    },
  });
}

export function useCustomer(id: string) {
  return useQuery({
    queryKey: ['customer', id],
    queryFn: async () => {
      const response = await api.get(`/v1/customers/${id}`);
      const customer = response.data as {
        id: string;
        name: string;
        customerType?: string;
        customer_type?: string;
        email?: string | null;
        phone?: string | null;
        address?: string | null;
        createdAt?: string;
        created_at?: string;
        tier?: string;
        financialSnapshot?: {
          totalInvoices: number;
          totalSpend: string;
          outstandingBalance: string;
          paidInvoices: number;
          lastInvoiceAt?: string | null;
        };
        financial_snapshot?: {
          total_invoices: number;
          total_spend: string;
          outstanding_balance: string;
          paid_invoices: number;
          last_invoice_at?: string | null;
        };
        serviceChronicle?: Array<{
          id: string;
          kind: string;
          referenceNo: string;
          status: string;
          occurredAt: string;
          summary: string;
        }>;
        service_chronicle?: Array<{
          id: string;
          kind: string;
          reference_no: string;
          status: string;
          occurred_at: string;
          summary: string;
        }>;
        vehicles?: Array<{
          id: string;
          registrationNo?: string;
          registration_no?: string;
          brand?: string;
          make?: string;
          model: string;
          year: number | null;
        }>;
        jobs?: Array<{
          id: string;
          jobNo?: number | null;
          job_no?: number | null;
          status: string;
          createdAt?: string;
          created_at?: string;
        }>;
        invoices?: Array<{
          id: string;
          invoiceNo?: number | null;
          invoice_no?: number | null;
          totalAmount?: string;
          total_amount?: string;
          status: string;
          createdAt?: string;
          created_at?: string;
        }>;
      };

      return {
        id: customer.id,
        name: customer.name,
        type: ((customer.customerType ?? customer.customer_type ?? 'INDIVIDUAL').toUpperCase() as 'INDIVIDUAL' | 'ORGANIZATION'),
        email: customer.email ?? null,
        phone: customer.phone ?? null,
        address: customer.address ?? null,
        created_at: customer.createdAt ?? customer.created_at ?? '',
        tier: customer.tier ?? 'BRONZE',
        financialSnapshot: customer.financialSnapshot
          ? customer.financialSnapshot
          : customer.financial_snapshot
          ? {
              totalInvoices: customer.financial_snapshot.total_invoices,
              totalSpend: customer.financial_snapshot.total_spend,
              outstandingBalance: customer.financial_snapshot.outstanding_balance,
              paidInvoices: customer.financial_snapshot.paid_invoices,
              lastInvoiceAt: customer.financial_snapshot.last_invoice_at,
            }
          : undefined,
        serviceChronicle: customer.serviceChronicle
          ? customer.serviceChronicle
          : (customer.service_chronicle ?? []).map((entry) => ({
              id: entry.id,
              kind: entry.kind,
              referenceNo: entry.reference_no,
              status: entry.status,
              occurredAt: entry.occurred_at,
              summary: entry.summary,
            })),
        vehicles: (customer.vehicles ?? []).map((vehicle) => ({
          id: vehicle.id,
          registration_no: vehicle.registrationNo ?? vehicle.registration_no ?? '',
          brand: vehicle.brand ?? vehicle.make ?? '',
          model: vehicle.model,
          year: vehicle.year,
        })),
        jobs: (customer.jobs ?? []).map((job) => ({
          id: job.id,
          job_number: `JOB-${job.jobNo ?? job.job_no ?? '-'}`,
          status: job.status,
          created_at: job.createdAt ?? job.created_at ?? '',
        })),
        invoices: (customer.invoices ?? []).map((invoice) => ({
          id: invoice.id,
          invoice_no: invoice.invoiceNo ?? invoice.invoice_no ?? null,
          total_amount: invoice.totalAmount ?? invoice.total_amount ?? '0.00',
          status: invoice.status,
          created_at: invoice.createdAt ?? invoice.created_at ?? '',
        })),
      } as CustomerDetail;
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