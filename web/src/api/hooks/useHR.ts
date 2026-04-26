import { useQuery } from '@tanstack/react-query';
import api from '@/api/client';

export function useEmployeeDetail(id?: string) {
  return useQuery({
    queryKey: ['hr', 'employee', id],
    enabled: Boolean(id),
    queryFn: async () => {
      const response = await api.get(`/v1/hr/employees/${id}`);
      return response.data as {
        id: string;
        employeeNo: string;
        firstName: string;
        lastName: string;
        email?: string | null;
        phone: string;
        employmentType: string;
        department?: string | null;
        designation?: string | null;
        joinDate?: string | null;
        salary?: string | null;
        isActive: boolean;
      };
    },
  });
}
