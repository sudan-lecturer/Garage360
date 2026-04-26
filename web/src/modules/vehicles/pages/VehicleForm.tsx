import { useEffect, useState } from 'react';
import { AxiosError } from 'axios';
import { useNavigate, useParams, useSearchParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { Button } from '@/components/ui/button';
import { useCreateVehicle, useUpdateVehicle, useVehicle } from '@/api/hooks/useVehicles';

const initialState = {
  licensePlate: '',
  make: '',
  model: '',
  year: '',
  color: '',
  vin: '',
  customerId: '',
};

export default function VehicleFormPage() {
  const { id } = useParams();
  const [searchParams] = useSearchParams();
  const editing = Boolean(id);
  const navigate = useNavigate();
  const [errorMessage, setErrorMessage] = useState('');
  const [formState, setFormState] = useState(initialState);

  const vehicleQuery = useVehicle(id || '');
  const createMutation = useCreateVehicle();
  const updateMutation = useUpdateVehicle();

  useEffect(() => {
    if (editing) return;
    const customerId = searchParams.get('customerId');
    if (!customerId) return;
    setFormState((state) => ({ ...state, customerId }));
  }, [editing, searchParams]);

  useEffect(() => {
    if (!vehicleQuery.data || !editing) return;
    setFormState({
      licensePlate: vehicleQuery.data.license_plate || '',
      make: vehicleQuery.data.make || '',
      model: vehicleQuery.data.model || '',
      year: vehicleQuery.data.year?.toString() || '',
      color: vehicleQuery.data.color || '',
      vin: vehicleQuery.data.vin || '',
      customerId: vehicleQuery.data.customer_id || '',
    });
  }, [vehicleQuery.data, editing]);

  const onSubmit = () => {
    setErrorMessage('');
    if (!formState.licensePlate.trim() || !formState.customerId.trim()) {
      setErrorMessage('License plate and customer ID are required.');
      return;
    }

    const payload = {
      license_plate: formState.licensePlate.trim(),
      make: formState.make.trim(),
      model: formState.model.trim(),
      year: formState.year ? Number(formState.year) : null,
      color: formState.color.trim() || null,
      vin: formState.vin.trim() || null,
      customer_id: formState.customerId.trim(),
    };

    if (editing && id) {
      updateMutation.mutate(
        { id, ...payload },
        {
          onSuccess: () => navigate(`/vehicles/${id}`),
          onError: (error) => {
            const typed = error as AxiosError<{ error?: { message?: string } }>;
            setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to update vehicle.');
          },
        }
      );
      return;
    }

    createMutation.mutate(payload as any, {
      onSuccess: () => navigate('/vehicles'),
      onError: (error) => {
        const typed = error as AxiosError<{ error?: { message?: string } }>;
        setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to create vehicle.');
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
            value={formState.licensePlate}
            onChange={(e) => setFormState((s) => ({ ...s, licensePlate: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="License Plate *"
          />
          <input
            value={formState.customerId}
            onChange={(e) => setFormState((s) => ({ ...s, customerId: e.target.value }))}
            className="h-10 rounded-sm border border-input bg-background px-3 text-sm"
            placeholder="Customer ID *"
          />
          <input
            value={formState.make}
            onChange={(e) => setFormState((s) => ({ ...s, make: e.target.value }))}
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
        <input
          value={formState.vin}
          onChange={(e) => setFormState((s) => ({ ...s, vin: e.target.value }))}
          className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm"
          placeholder="VIN"
        />
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
