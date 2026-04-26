import { useEffect, useState } from 'react';
import { AxiosError } from 'axios';
import { useNavigate, useParams, useSearchParams } from 'react-router-dom';
import { Check, Upload, X } from 'lucide-react';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { Button } from '@/components/ui/button';
import { useCreateVehicle, useUpdateVehicle, useVehicle } from '@/api/hooks/useVehicles';
import { useCustomers } from '@/api/hooks/useCustomers';

const initialState = {
  registrationNo: '',
  brand: '',
  model: '',
  year: '',
  color: '',
  customerId: '',
  photoBase64: '',
  photoMimeType: '',
};

export default function VehicleFormPage() {
  const { id } = useParams();
  const [searchParams] = useSearchParams();
  const editing = Boolean(id);
  const navigate = useNavigate();
  const [errorMessage, setErrorMessage] = useState('');
  const [formState, setFormState] = useState(initialState);
  const [customerQuery, setCustomerQuery] = useState('');
  const [isCustomerMenuOpen, setIsCustomerMenuOpen] = useState(false);
  const [photoPreview, setPhotoPreview] = useState('');
  const [photoName, setPhotoName] = useState('');

  const vehicleQuery = useVehicle(id || '');
  const createMutation = useCreateVehicle();
  const updateMutation = useUpdateVehicle();
  const customersQuery = useCustomers({
    search: customerQuery || undefined,
    limit: 20,
  });

  useEffect(() => {
    if (editing) return;
    const customerId = searchParams.get('customerId');
    if (!customerId) return;
    setFormState((state) => ({ ...state, customerId }));
  }, [editing, searchParams]);

  useEffect(() => {
    if (!vehicleQuery.data || !editing) return;
    setFormState({
      registrationNo: vehicleQuery.data.registration_no || '',
      brand: vehicleQuery.data.brand || '',
      model: vehicleQuery.data.model || '',
      year: vehicleQuery.data.year?.toString() || '',
      color: vehicleQuery.data.color || '',
      customerId: vehicleQuery.data.customer_id || '',
      photoBase64: '',
      photoMimeType: '',
    });
    setCustomerQuery(vehicleQuery.data.customer_name || '');
    setPhotoPreview('');
    setPhotoName('');
  }, [vehicleQuery.data, editing]);

  useEffect(() => {
    if (editing) return;
    const selected = (customersQuery.data?.data ?? []).find((c) => c.id === formState.customerId);
    if (selected) {
      setCustomerQuery(selected.name);
    }
  }, [customersQuery.data, formState.customerId, editing]);

  const getApiErrorMessage = (error: unknown, fallback: string) => {
    const typed = error as AxiosError<{ error?: { message?: string } | string; message?: string }>;
    const payload = typed.response?.data;
    if (!payload) return fallback;
    if (typeof payload.error === 'string' && payload.error.trim()) return payload.error;
    if (typeof payload.error === 'object' && payload.error?.message?.trim()) return payload.error.message;
    if (payload.message?.trim()) return payload.message;
    return fallback;
  };

  const onSubmit = () => {
    setErrorMessage('');
    if (!formState.registrationNo.trim() || !formState.customerId.trim() || !formState.brand.trim() || !formState.model.trim()) {
      setErrorMessage('License plate, customer, brand and model are required.');
      return;
    }

    const payload = {
      registration_no: formState.registrationNo.trim(),
      make: formState.brand.trim(),
      model: formState.model.trim(),
      year: formState.year ? Number(formState.year) : null,
      color: formState.color.trim() || null,
      customer_id: formState.customerId.trim(),
      photo_base64: formState.photoBase64 || undefined,
      photo_mime_type: formState.photoMimeType || undefined,
    };

    if (editing && id) {
      updateMutation.mutate(
        { id, ...payload },
        {
          onSuccess: () => navigate(`/vehicles/${id}`),
          onError: (error) => {
            setErrorMessage(getApiErrorMessage(error, 'Failed to update vehicle.'));
          },
        }
      );
      return;
    }

    createMutation.mutate(payload as any, {
      onSuccess: () => navigate('/vehicles'),
      onError: (error) => {
        setErrorMessage(getApiErrorMessage(error, 'Failed to create vehicle.'));
      },
    });
  };

  if (editing && vehicleQuery.isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={editing ? 'Edit Vehicle' : 'Create Vehicle'}
        description="Manage vehicle registration and profile information."
        breadcrumbs={[
          { label: 'Vehicles', href: '/vehicles' },
          { label: editing ? 'Edit Vehicle' : 'Create Vehicle' },
        ]}
      />

      <section className="rounded-sm border border-border bg-surface p-4 space-y-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <input
            value={formState.registrationNo}
            onChange={(e) => setFormState((s) => ({ ...s, registrationNo: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="License Plate *"
          />
          <div className="sm:col-span-2 relative">
            <input
              value={customerQuery}
              onFocus={() => setIsCustomerMenuOpen(true)}
              onBlur={() => {
                window.setTimeout(() => setIsCustomerMenuOpen(false), 150);
              }}
              onChange={(e) => {
                setCustomerQuery(e.target.value);
                setFormState((s) => ({ ...s, customerId: '' }));
                setIsCustomerMenuOpen(true);
              }}
              className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm"
              placeholder="Search and select customer by name / phone / email *"
            />
            {isCustomerMenuOpen && (
              <div className="absolute z-20 mt-1 max-h-56 w-full overflow-auto rounded-sm border border-border bg-background shadow-lg">
                {(customersQuery.data?.data ?? []).slice(0, 8).map((customer) => (
                  <button
                    key={customer.id}
                    type="button"
                    className="flex w-full items-center justify-between border-b border-border px-3 py-2 text-left text-sm last:border-b-0 hover:bg-surface-raised"
                    onMouseDown={(event) => {
                      event.preventDefault();
                      setFormState((s) => ({ ...s, customerId: customer.id }));
                      setCustomerQuery(customer.name);
                      setErrorMessage('');
                      setIsCustomerMenuOpen(false);
                    }}
                  >
                    <span>{customer.name}</span>
                    <span className="inline-flex items-center gap-2 text-xs text-muted-foreground">
                      {formState.customerId === customer.id && <Check className="h-3.5 w-3.5 text-accent" />}
                      {customer.type === 'ORGANIZATION' ? 'Corporate' : 'Individual'}
                    </span>
                  </button>
                ))}
                {(customersQuery.data?.data ?? []).length === 0 && (
                  <div className="px-3 py-2 text-sm text-muted-foreground">No customers found.</div>
                )}
              </div>
            )}
          </div>
          <input
            value={formState.brand}
            onChange={(e) => setFormState((s) => ({ ...s, brand: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Brand"
          />
          <input
            value={formState.model}
            onChange={(e) => setFormState((s) => ({ ...s, model: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Model"
          />
          <input
            type="number"
            value={formState.year}
            onChange={(e) => setFormState((s) => ({ ...s, year: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Year"
          />
          <input
            value={formState.color}
            onChange={(e) => setFormState((s) => ({ ...s, color: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Color"
          />
        </div>
        <div>
          <label className="mb-2 block text-xs uppercase tracking-wide text-muted-foreground">
            Vehicle Picture
          </label>
          <label className="flex cursor-pointer items-center justify-center gap-2 rounded-sm border border-dashed border-input bg-background px-4 py-6 text-sm text-muted-foreground hover:bg-surface-raised">
            <Upload className="h-4 w-4" />
            <span>{photoName ? 'Replace image' : 'Upload image'}</span>
            <input
              type="file"
              accept="image/*"
              className="hidden"
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (!file) return;
                const reader = new FileReader();
                reader.onload = () => {
                  const result = String(reader.result);
                  setFormState((s) => ({
                    ...s,
                    photoBase64: result,
                    photoMimeType: file.type || 'image/jpeg',
                  }));
                  setPhotoPreview(result);
                  setPhotoName(file.name);
                };
                reader.readAsDataURL(file);
              }}
            />
          </label>
          <p className="mt-2 text-xs text-muted-foreground">Optional. Best results under 5MB.</p>
          {photoPreview && (
            <div className="mt-3 rounded-sm border border-border bg-background p-2">
              <div className="mb-2 flex items-center justify-between text-xs text-muted-foreground">
                <span className="truncate pr-3">{photoName}</span>
                <button
                  type="button"
                  className="inline-flex items-center gap-1 text-destructive hover:underline"
                  onClick={() => {
                    setFormState((s) => ({ ...s, photoBase64: '', photoMimeType: '' }));
                    setPhotoPreview('');
                    setPhotoName('');
                  }}
                >
                  <X className="h-3.5 w-3.5" /> Remove
                </button>
              </div>
              <img src={photoPreview} alt="Vehicle preview" className="h-44 w-full rounded-sm object-cover" />
            </div>
          )}
        </div>
      </section>

      {errorMessage && (
        <div className="rounded-sm border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
          {errorMessage}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" onClick={() => navigate('/vehicles')}>
          Cancel
        </Button>
        <Button
          type="button"
          onClick={onSubmit}
          disabled={createMutation.isPending || updateMutation.isPending}
        >
          {editing ? 'Save Vehicle' : 'Create Vehicle'}
        </Button>
      </div>
    </div>
  );
}
