import { useState } from 'react';
import { Link } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import api from '@/api/client';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { FormField, FormSelect } from '@/components/shared/form-field';
import { Button } from '@/components/ui/button';
import { Plus, Users, Edit, Trash2 } from 'lucide-react';

interface Employee {
  id: string;
  employee_no: string;
  first_name: string;
  last_name: string;
  email: string | null;
  phone: string;
  employment_type: string;
  department: string | null;
  designation: string | null;
  is_active: boolean;
}

const employeeSchema = z.object({
  first_name: z.string().min(1, 'First name is required'),
  last_name: z.string().min(1, 'Last name is required'),
  employee_no: z.string().min(1, 'Employee ID is required'),
  phone: z.string().optional(),
  email: z.string().email('Invalid email').optional(),
  employment_type: z.enum(['FULL_TIME', 'PART_TIME', 'CONTRACT', 'INTERN']),
  department: z.string().optional(),
  designation: z.string().optional(),
});

type EmployeeForm = z.infer<typeof employeeSchema>;

function useEmployees() {
  return useQuery({
    queryKey: ['employees'],
    queryFn: async () => {
      const response = await api.get<{ data: Employee[] }>('/v1/hr/employees');
      return response.data;
    },
  });
}

export default function EmployeeListPage() {
  const [showForm, setShowForm] = useState(false);
  const { data, isLoading, error } = useEmployees();
  const queryClient = useQueryClient();

  const { register, handleSubmit, formState: { errors }, reset } = useForm<EmployeeForm>({
    resolver: zodResolver(employeeSchema),
    defaultValues: { first_name: '', last_name: '', employee_no: '', phone: '', email: '', employment_type: 'FULL_TIME', department: '', designation: '' },
  });

  const createMutation = useMutation({
    mutationFn: async (data: EmployeeForm) => {
      const response = await api.post('/v1/hr/employees', data);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['employees'] });
      setShowForm(false);
      reset();
    },
  });

  const onSubmit = async (data: EmployeeForm) => {
    await createMutation.mutateAsync(data);
  };

  return (
    <div className="space-y-4">
      <PageHeader
        title="Employees"
        description="Manage staff and employees"
        actions={
          <Button onClick={() => setShowForm(!showForm)}>
            <Plus className="h-4 w-4 mr-1" /> Add Employee
          </Button>
        }
      />

      {showForm && (
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4">New Employee</h3>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 max-w-lg">
            <div className="grid grid-cols-2 gap-4">
              <FormField name="first_name" label="First Name" register={register} errors={errors} required />
              <FormField name="last_name" label="Last Name" register={register} errors={errors} required />
            </div>
            <FormField name="employee_no" label="Employee ID" register={register} errors={errors} required />
            <div className="grid grid-cols-2 gap-4">
              <FormField name="phone" label="Phone" register={register} errors={errors} />
              <FormField name="email" label="Email" type="email" register={register} errors={errors} />
            </div>
            <FormSelect
              name="employment_type"
              label="Employment Type"
              options={[
                { value: 'FULL_TIME', label: 'Full Time' },
                { value: 'PART_TIME', label: 'Part Time' },
                { value: 'CONTRACT', label: 'Contract' },
                { value: 'INTERN', label: 'Intern' },
              ]}
              register={register}
              errors={errors}
              required
            />
            <div className="grid grid-cols-2 gap-4">
              <FormField name="department" label="Department" register={register} errors={errors} />
              <FormField name="designation" label="Designation" register={register} errors={errors} />
            </div>
            <div className="flex gap-2">
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending ? 'Saving...' : 'Save Employee'}
              </Button>
              <Button type="button" variant="outline" onClick={() => setShowForm(false)}>Cancel</Button>
            </div>
          </form>
        </div>
      )}

      {isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {error && <EmptyState icon="default" title="Error loading employees" description="Please try again later" />}
      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState icon="default" title="No employees" description="Add your first employee" />
      )}

      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-lg border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Employee</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">ID</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Type</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Department</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map(emp => (
                <tr key={emp.id} className="border-b border-border last:border-0 hover:bg-surface-raised">
                  <td className="p-3">
                    <Link to={`/hr/employees/${emp.id}`} className="flex items-center gap-2 hover:text-accent">
                      <Users className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">{emp.first_name} {emp.last_name}</span>
                    </Link>
                  </td>
                  <td className="p-3 text-sm">{emp.employee_no}</td>
                  <td className="p-3 text-sm">{emp.employment_type}</td>
                  <td className="p-3 text-sm">{emp.department || '-'}</td>
                  <td className="p-3 text-sm">
                    <span className={emp.is_active ? 'text-success' : 'text-muted-foreground'}>
                      {emp.is_active ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td className="p-3 text-right">
                    <div className="flex gap-2 justify-end">
                      <Button variant="ghost" size="sm"><Edit className="h-4 w-4" /></Button>
                      <Button variant="ghost" size="sm"><Trash2 className="h-4 w-4 text-destructive" /></Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}