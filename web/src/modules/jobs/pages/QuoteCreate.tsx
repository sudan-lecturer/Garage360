import { useMemo, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { AxiosError } from 'axios';
import { useNavigate, useParams } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { SearchInput } from '@/components/shared/search-input';
import api from '@/api/client';

interface JobSummary {
  id: string;
  job_no: number | null;
  vehicle_registration_no: string;
  customer_name: string;
  status: string;
}

interface JobsResponse {
  data: JobSummary[];
}

interface QuoteItemDraft {
  itemType: 'PART' | 'LABOUR';
  description: string;
  quantity: string;
  unitPrice: string;
  discountPct: string;
}

interface CreateJobItemPayload {
  itemType: string;
  description: string;
  quantity: string;
  unitPrice: string;
  discountPct?: string;
}

function useJobs(search: string) {
  return useQuery({
    queryKey: ['jobs', 'quote-picker', search],
    queryFn: async () => {
      const response = await api.get<JobsResponse>('/v1/jobs', {
        params: { page: 1, limit: 50, search: search || undefined },
      });
      return response.data.data;
    },
  });
}

export default function QuoteCreatePage() {
  const navigate = useNavigate();
  const { id } = useParams();
  const [search, setSearch] = useState('');
  const [selectedJobId, setSelectedJobId] = useState(id ?? '');
  const [errorMessage, setErrorMessage] = useState('');
  const [items, setItems] = useState<QuoteItemDraft[]>([
    { itemType: 'PART', description: '', quantity: '1', unitPrice: '0', discountPct: '0' },
  ]);

  const jobsQuery = useJobs(search);

  const createItemMutation = useMutation({
    mutationFn: async (payload: { jobId: string; item: CreateJobItemPayload }) => {
      const response = await api.post(`/v1/jobs/${payload.jobId}/items`, payload.item);
      return response.data;
    },
  });

  const total = useMemo(() => {
    return items.reduce((sum, item) => {
      const quantity = Number(item.quantity) || 0;
      const unitPrice = Number(item.unitPrice) || 0;
      const discountPct = Number(item.discountPct) || 0;
      const lineTotal = quantity * unitPrice;
      const discount = lineTotal * (discountPct / 100);
      return sum + (lineTotal - discount);
    }, 0);
  }, [items]);

  const updateItem = (index: number, field: keyof QuoteItemDraft, value: string) => {
    setItems((current) =>
      current.map((item, itemIndex) =>
        itemIndex === index
          ? {
              ...item,
              [field]: value,
            }
          : item
      )
    );
  };

  const submitQuote = async () => {
    setErrorMessage('');
    if (selectedJobId.trim().length === 0) {
      setErrorMessage('Please select a job card.');
      return;
    }

    const validItems = items.filter(
      (item) => item.description.trim().length > 0 && Number(item.quantity) > 0
    );

    if (validItems.length === 0) {
      setErrorMessage('Add at least one valid quote item.');
      return;
    }

    try {
      for (const item of validItems) {
        await createItemMutation.mutateAsync({
          jobId: selectedJobId,
          item: {
            itemType: item.itemType,
            description: item.description.trim(),
            quantity: item.quantity,
            unitPrice: item.unitPrice,
            discountPct: item.discountPct,
          },
        });
      }

      navigate(`/jobs/${selectedJobId}`);
    } catch (error) {
      const axiosError = error as AxiosError<{ error?: { message?: string } }>;
      const message = axiosError.response?.data?.error?.message ?? 'Failed to save quote.';
      setErrorMessage(message);
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="Quote Creation"
        description="Create quote line items and attach them to a job card."
        breadcrumbs={[
          { label: 'Jobs', href: '/jobs' },
          { label: 'Quote Creation' },
        ]}
      />

      <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
        <h2 className="text-lg font-semibold">Select Job Card</h2>
        <div className="max-w-md">
          <SearchInput
            value={search}
            onChange={setSearch}
            placeholder="Search by job number, customer, vehicle..."
          />
        </div>

        {jobsQuery.isLoading ? (
          <LoadingSpinner />
        ) : (
          <div className="max-h-64 overflow-auto rounded-md border border-border">
            {jobsQuery.data?.map((job) => (
              <button
                key={job.id}
                type="button"
                onClick={() => setSelectedJobId(job.id)}
                className={`w-full border-b border-border p-3 text-left last:border-0 ${
                  selectedJobId === job.id ? 'bg-surface-raised' : 'hover:bg-surface-raised'
                }`}
              >
                <p className="font-medium">Job #{job.job_no ?? '-'} • {job.customer_name}</p>
                <p className="text-xs text-muted-foreground">
                  {job.vehicle_registration_no} • {job.status}
                </p>
              </button>
            ))}
          </div>
        )}
      </section>

      <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Quote Items</h2>
          <Button
            type="button"
            variant="outline"
            onClick={() =>
              setItems((current) => [
                ...current,
                {
                  itemType: 'PART',
                  description: '',
                  quantity: '1',
                  unitPrice: '0',
                  discountPct: '0',
                },
              ])
            }
          >
            Add Item
          </Button>
        </div>

        <div className="space-y-3">
          {items.map((item, index) => (
            <div key={`quote-item-${index}`} className="grid gap-2 sm:grid-cols-12">
              <select
                value={item.itemType}
                onChange={(event) => updateItem(index, 'itemType', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-2"
              >
                <option value="PART">PART</option>
                <option value="LABOUR">LABOUR</option>
              </select>
              <input
                value={item.description}
                onChange={(event) => updateItem(index, 'description', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-4"
                placeholder="Description"
              />
              <input
                value={item.quantity}
                onChange={(event) => updateItem(index, 'quantity', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-2"
                type="number"
                min="0.001"
                step="0.001"
                placeholder="Qty"
              />
              <input
                value={item.unitPrice}
                onChange={(event) => updateItem(index, 'unitPrice', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-2"
                type="number"
                min="0"
                step="0.01"
                placeholder="Unit Price"
              />
              <input
                value={item.discountPct}
                onChange={(event) => updateItem(index, 'discountPct', event.target.value)}
                className="h-10 rounded-md border border-input bg-background px-3 text-sm sm:col-span-1"
                type="number"
                min="0"
                max="100"
                step="0.01"
                placeholder="%"
              />
              <Button
                type="button"
                variant="ghost"
                className="sm:col-span-1"
                onClick={() => setItems((current) => current.filter((_, i) => i !== index))}
                disabled={items.length === 1}
              >
                X
              </Button>
            </div>
          ))}
        </div>

        <div className="rounded-md border border-border bg-background p-3 text-sm">
          Quote Total: <span className="font-semibold">Rs. {total.toLocaleString()}</span>
        </div>
      </section>

      {errorMessage && (
        <div className="rounded-md border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
          {errorMessage}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" onClick={() => navigate('/jobs')}>
          Cancel
        </Button>
        <Button type="button" onClick={() => void submitQuote()} disabled={createItemMutation.isPending}>
          {createItemMutation.isPending ? 'Saving...' : 'Save Quote'}
        </Button>
      </div>
    </div>
  );
}
