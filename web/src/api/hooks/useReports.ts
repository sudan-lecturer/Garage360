import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export type ReportType = 'revenue' | 'jobs' | 'customers';
export type ExportFormat = 'json' | 'csv';

export interface SavedReport {
  id: string;
  name: string;
  reportType: string;
  config: unknown;
  createdAt?: string;
}

export interface ReportConfig {
  fromDate?: string;
  toDate?: string;
}

export function useSavedReports() {
  return useQuery({
    queryKey: ['reports', 'saved'],
    queryFn: async () => {
      const response = await api.get<SavedReport[]>('/v1/reports/saved');
      return response.data;
    },
  });
}

export function useGenerateReport() {
  return useMutation({
    mutationFn: async ({ reportType, config }: { reportType: ReportType; config: ReportConfig }) => {
      const response = await api.post('/v1/reports/generate', { reportType, config });
      return response.data;
    },
  });
}

export function useExportReport() {
  return useMutation({
    mutationFn: async ({
      reportType,
      config,
      format,
    }: {
      reportType: ReportType;
      config: ReportConfig;
      format: ExportFormat;
    }) => {
      const response = await api.post('/v1/reports/export', { reportType, config, format });
      return response.data;
    },
  });
}

export function useCreateSavedReport() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      name,
      reportType,
      config,
    }: {
      name: string;
      reportType: ReportType;
      config: ReportConfig;
    }) => {
      const response = await api.post('/v1/reports/saved', { name, reportType, config });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['reports', 'saved'] });
    },
  });
}

export function useDeleteSavedReport() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/v1/reports/saved/${id}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['reports', 'saved'] });
    },
  });
}
