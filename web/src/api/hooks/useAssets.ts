import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '@/api/client';

export interface AssetRequest {
  assetTag: string;
  name: string;
  category?: string;
  locationId?: string;
  purchaseDate?: string;
  purchaseCost?: string;
  usefulLifeYears?: number;
  status?: string;
  notes?: string;
}

export function useCreateAsset() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (payload: AssetRequest) => {
      const response = await api.post('/v1/assets', payload);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['assets'] });
    },
  });
}

export function useAssetDetail(id?: string) {
  return useQuery({
    queryKey: ['assets', 'detail', id],
    enabled: Boolean(id),
    queryFn: async () => {
      const response = await api.get(`/v1/assets/${id}`);
      return response.data;
    },
  });
}
