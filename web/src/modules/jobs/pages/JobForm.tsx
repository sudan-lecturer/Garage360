import { useNavigate, useParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useCustomers } from '@/api/hooks/useCustomers';
import { useVehicles } from '@/api/hooks/useVehicles';
import { useCreateJob, useUpdateJob } from '@/api/hooks/useJobs';
import { PageHeader } from '@/components/shared/page-header';
import { FormSelect, FormTextarea } from '@/components/shared/form-field';
import { Button } from '@/components/ui/button';
import { ArrowLeft, Save, User, Car, FileText } from 'lucide-react';

const jobSchema = z.object({
  customer_id: z.string().min(1, 'Customer is required'),
  vehicle_id: z.string().min(1, 'Vehicle is required'),
  complaint: z.string().min(1, 'Complaint is required'),
  bay_id: z.string().optional(),
  mechanic_id: z.string().optional(),
});

type JobForm = z.infer<typeof jobSchema>;

export default function JobFormPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const isEdit = Boolean(id);

  const { register, handleSubmit, formState: { errors }, watch } = useForm<JobForm>({
    resolver: zodResolver(jobSchema),
    defaultValues: {
      customer_id: '',
      vehicle_id: '',
      complaint: '',
      bay_id: '',
      mechanic_id: '',
    },
  });

  const selectedCustomerId = watch('customer_id');
  const createJobMutation = useCreateJob();
  const updateJobMutation = useUpdateJob();

  // Fetch customers for dropdown
  const { data: customersData } = useCustomers({ limit: 100 });
  const customers = customersData?.data?.map((c: any) => ({
    value: c.id,
    label: c.name,
  })) || [];

  // Fetch vehicles for dropdown (filtered by customer)
  const { data: vehiclesData } = useVehicles({ 
    customer_id: selectedCustomerId || undefined,
  });
  const vehicles = vehiclesData?.data?.map((v: any) => ({
    value: v.id,
    label: `${v.license_plate ?? '-'} - ${v.make ?? ''} ${v.model ?? ''}`,
  })) || [];

  const onSubmit = async (data: JobForm) => {
    const payload = {
      customerId: data.customer_id,
      vehicleId: data.vehicle_id,
      complaint: data.complaint,
    };

    if (isEdit && id) {
      updateJobMutation.mutate(
        { id, ...payload } as any,
        {
          onSuccess: () => navigate(`/jobs/${id}`),
        }
      );
      return;
    }

    createJobMutation.mutate(payload as any, {
      onSuccess: () => navigate('/jobs'),
    });
  };

  return (
    <div className="space-y-4">
      <PageHeader
        title={isEdit ? 'Edit Job' : 'New Job'}
        description={isEdit ? 'Update job information' : 'Create a new job card'}
        breadcrumbs={[
          { label: 'Jobs', href: '/jobs' },
          { label: isEdit ? 'Edit' : 'New' },
        ]}
      />

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-2xl">
        {/* Step 1: Customer Selection */}
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <User className="h-5 w-5" /> Step 1: Select Customer
          </h3>
          
          <FormSelect
            name="customer_id"
            label="Customer"
            options={customers}
            placeholder="Select customer..."
            register={register}
            errors={errors}
            required
          />
        </div>

        {/* Step 2: Vehicle Selection */}
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Car className="h-5 w-5" /> Step 2: Select Vehicle
          </h3>
          
          {selectedCustomerId ? (
            <FormSelect
              name="vehicle_id"
              label="Vehicle"
              options={vehicles}
              placeholder="Select vehicle..."
              register={register}
              errors={errors}
              required
            />
          ) : (
            <p className="text-sm text-muted-foreground">Select a customer first</p>
          )}

          <div className="mt-4">
            <Button type="button" variant="outline" size="sm">
              + Add New Vehicle
            </Button>
          </div>
        </div>

        {/* Step 3: Job Details */}
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <FileText className="h-5 w-5" /> Step 3: Job Details
          </h3>
          
          <FormTextarea
            name="complaint"
            label="Customer Complaint"
            placeholder="Describe the issue reported by the customer..."
            register={register}
            errors={errors}
            rows={4}
            required
          />

          <div className="grid grid-cols-2 gap-4 mt-4">
            <FormSelect
              name="mechanic_id"
              label="Assign Mechanic (Optional)"
              options={[
                { value: '', label: 'Not assigned' },
                { value: 'mech1', label: 'Mechanic 1' },
                { value: 'mech2', label: 'Mechanic 2' },
              ]}
              register={register}
              errors={errors}
            />

            <FormSelect
              name="bay_id"
              label="Service Bay (Optional)"
              options={[
                { value: '', label: 'No bay assigned' },
                { value: 'bay1', label: 'Bay 1' },
                { value: 'bay2', label: 'Bay 2' },
                { value: 'bay3', label: 'Bay 3' },
              ]}
              register={register}
              errors={errors}
            />
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <Button
            type="button"
            variant="outline"
            onClick={() => navigate('/jobs')}
          >
            <ArrowLeft className="h-4 w-4 mr-1" /> Cancel
          </Button>
          <Button type="submit" disabled={createJobMutation.isPending || updateJobMutation.isPending}>
            <Save className="h-4 w-4 mr-1" />
            {isEdit ? (updateJobMutation.isPending ? 'Updating...' : 'Update Job') : (createJobMutation.isPending ? 'Creating...' : 'Create Job')}
          </Button>
        </div>
      </form>
    </div>
  );
}