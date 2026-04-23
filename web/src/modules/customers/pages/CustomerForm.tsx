import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useCreateCustomer, useUpdateCustomer } from '@/api/hooks/useCustomers';
import { PageHeader } from '@/components/shared/page-header';
import { FormField, FormTextarea } from '@/components/shared/form-field';
import { Button } from '@/components/ui/button';
import { ArrowLeft, Save } from 'lucide-react';

const customerSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  type: z.enum(['INDIVIDUAL', 'ORGANISATION']),
  email: z.string().email('Invalid email').optional().or(z.literal('')),
  phone: z.string().optional(),
  address: z.string().optional(),
});

type CustomerForm = z.infer<typeof customerSchema>;

export default function CustomerFormPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const isEdit = Boolean(id);

  const [type, setType] = useState<'INDIVIDUAL' | 'ORGANISATION'>('INDIVIDUAL');

  const { register, handleSubmit, formState: { errors } } = useForm<CustomerForm>({
    resolver: zodResolver(customerSchema),
    defaultValues: {
      name: '',
      type: 'INDIVIDUAL',
      email: '',
      phone: '',
      address: '',
    },
  });

const createMutation = useCreateCustomer();
  const updateMutation = useUpdateCustomer();

  const onSubmit = async (data: CustomerForm) => {
    try {
      const payload = {
        ...data,
        email: data.email || null,
      };
      if (isEdit && id) {
        await updateMutation.mutateAsync({ id, ...payload });
      } else {
        await createMutation.mutateAsync(payload);
      }
      navigate('/customers');
    } catch (error) {
      console.error('Failed to save customer:', error);
    }
  };

  return (
    <div className="space-y-4">
      <PageHeader
        title={isEdit ? 'Edit Customer' : 'Add Customer'}
        description={isEdit ? 'Update customer information' : 'Add a new customer to your database'}
        breadcrumbs={[
          { label: 'Customers', href: '/customers' },
          { label: isEdit ? 'Edit' : 'New' },
        ]}
      />

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-lg">
        {/* Customer Type Toggle */}
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => setType('INDIVIDUAL')}
            className={`flex-1 py-2 rounded-md border text-sm font-medium transition-colors ${
              type === 'INDIVIDUAL'
                ? 'bg-accent text-accent-foreground border-accent'
                : 'border-border bg-surface hover:border-accent'
            }`}
          >
            Individual
          </button>
          <button
            type="button"
            onClick={() => setType('ORGANISATION')}
            className={`flex-1 py-2 rounded-md border text-sm font-medium transition-colors ${
              type === 'ORGANISATION'
                ? 'bg-accent text-accent-foreground border-accent'
                : 'border-border bg-surface hover:border-accent'
            }`}
          >
            Organisation
          </button>
        </div>

        <input type="hidden" {...register('type')} value={type} />

        <FormField
          name="name"
          label={type === 'ORGANISATION' ? 'Organisation Name' : 'Full Name'}
          placeholder={type === 'ORGANISATION' ? 'Enter organisation name' : 'Enter full name'}
          register={register}
          errors={errors}
          required
        />

        <div className="grid grid-cols-2 gap-4">
          <FormField
            name="phone"
            label="Phone"
            type="tel"
            placeholder="+977 98XXXXXXXX"
            register={register}
            errors={errors}
          />

          <FormField
            name="email"
            label="Email"
            type="email"
            placeholder="email@example.com"
            register={register}
            errors={errors}
          />
        </div>

        <FormTextarea
          name="address"
          label="Address"
          placeholder="Enter address (optional)"
          register={register}
          errors={errors}
          rows={3}
        />

        <div className="flex gap-3">
          <Button
            type="button"
            variant="outline"
            onClick={() => navigate('/customers')}
          >
            <ArrowLeft className="h-4 w-4 mr-1" /> Cancel
          </Button>
          <Button type="submit" disabled={createMutation.isPending || updateMutation.isPending}>
            <Save className="h-4 w-4 mr-1" />
            {createMutation.isPending || updateMutation.isPending ? 'Saving...' : 'Save'}
          </Button>
        </div>
      </form>
    </div>
  );
}