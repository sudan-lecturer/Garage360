import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export type JobStatus = 
  | 'INTAKE'
  | 'AUDIT'
  | 'QUOTE'
  | 'APPROVAL'
  | 'IN_SERVICE'
  | 'QA'
  | 'BILLING'
  | 'COMPLETED'
  | 'CANCELLED';

export interface Job {
  id: string;
  job_number: string;
  status: JobStatus;
  customer_id: string;
  customer_name: string;
  vehicle_id: string;
  vehicle_plate: string;
  vehicle_make: string;
  vehicle_model: string;
  mechanic_id: string | null;
  mechanic_name: string | null;
  bay_id: string | null;
  bay_name: string | null;
  complaint: string | null;
  created_at: string;
}

interface JobsResponse {
  data: Job[];
  total: number;
  page: number;
  limit: number;
}

export interface JobDetail extends Job {
  diagnosis: string | null;
  bay_id: string | null;
  estimated_completion: string | null;
  items?: JobItem[];
  activities?: Activity[];
  approvals?: Approval[];
  change_requests?: ChangeRequest[];
}

export interface JobItem {
  id: string;
  type: 'PART' | 'LABOUR';
  description: string;
  quantity: number;
  unit_price: number;
  total: number;
}

export interface Activity {
  id: string;
  action: string;
  description: string;
  performed_by: string;
  performed_by_role: string;
  created_at: string;
}

export interface Approval {
  id: string;
  channel: string;
  approved_by: string;
  created_at: string;
}

export interface ChangeRequest {
  id: string;
  status: 'PENDING' | 'APPROVED' | 'DECLINED';
  items: JobItem[];
  reason: string;
  created_at: string;
}

export function useJobs(params?: {
  status?: JobStatus;
  mechanic_id?: string;
  bay_id?: string;
  search?: string;
  page?: number;
  limit?: number;
}) {
  return useQuery({
    queryKey: ['jobs', params],
    queryFn: async () => {
      const response = await api.get<JobsResponse>('/v1/jobs', { params });
      return response.data;
    },
  });
}

export function useJob(id: string) {
  return useQuery({
    queryKey: ['job', id],
    queryFn: async () => {
      const response = await api.get<JobDetail>(`/v1/jobs/${id}`);
      return response.data;
    },
    enabled: !!id,
  });
}

export function useCreateJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: Partial<Job>) => {
      const response = await api.post<Job>('/v1/jobs', data);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['jobs'] });
    },
  });
}

export function useUpdateJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, ...data }: Partial<Job> & { id: string }) => {
      const response = await api.put<Job>(`/v1/jobs/${id}`, data);
      return response.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['jobs'] });
      queryClient.invalidateQueries({ queryKey: ['job', variables.id] });
    },
  });
}

export function useTransitionJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, status }: { id: string; status: JobStatus }) => {
      await api.post(`/v1/jobs/${id}/transition`, { status });
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['jobs'] });
      queryClient.invalidateQueries({ queryKey: ['job', variables.id] });
    },
  });
}

export function useAssignMechanic() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ job_id, mechanic_id }: { job_id: string; mechanic_id: string }) => {
      await api.put(`/v1/jobs/${job_id}/assign-mechanic`, { mechanic_id });
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['jobs'] });
      queryClient.invalidateQueries({ queryKey: ['job', variables.job_id] });
    },
  });
}

export function useAssignBay() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ job_id, bay_id }: { job_id: string; bay_id: string }) => {
      await api.put(`/v1/jobs/${job_id}/assign-bay`, { bay_id });
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['jobs'] });
      queryClient.invalidateQueries({ queryKey: ['job', variables.job_id] });
    },
  });
}

export function useAddJobNote() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ job_id, note }: { job_id: string; note: string }) => {
      await api.post(`/v1/jobs/${job_id}/activities/note`, { note });
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['job', variables.job_id] });
    },
  });
}