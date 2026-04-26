import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface DviTemplatePayload {
  name: string;
  sections: unknown;
}

export function useDviTemplates() {
  return useQuery({
    queryKey: ['dvi', 'templates'],
    queryFn: async () => {
      const response = await api.get('/v1/dvi/templates');
      return response.data as Array<{ id: string; name: string; sections: unknown }>;
    },
  });
}

export function useCreateDviTemplate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (payload: DviTemplatePayload) => {
      const response = await api.post('/v1/dvi/templates', payload);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dvi', 'templates'] });
    },
  });
}

export function useUpdateDviTemplate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, payload }: { id: string; payload: DviTemplatePayload }) => {
      const response = await api.put(`/v1/dvi/templates/${id}`, payload);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dvi', 'templates'] });
    },
  });
}

export function useCreateDviResult() {
  return useMutation({
    mutationFn: async (payload: { job_card_id: string; template_id?: string; data: unknown }) => {
      const response = await api.post('/v1/dvi/results', payload);
      return response.data;
    },
  });
}

export function useDviResult(id?: string) {
  return useQuery({
    queryKey: ['dvi', 'result', id],
    enabled: Boolean(id),
    queryFn: async () => {
      const response = await api.get(`/v1/dvi/results/${id}`);
      const data = response.data as {
        id: string;
        job_card_id: string;
        template_name?: string;
        submitted_by?: string;
        data: unknown;
        created_at?: string;
      };
      return {
        id: data.id,
        jobCardId: data.job_card_id,
        templateName: data.template_name,
        submittedBy: data.submitted_by,
        data: data.data,
        createdAt: data.created_at,
      };
    },
  });
}
