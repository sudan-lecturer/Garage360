import api from '@/api/client';
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { Button } from '@/components/ui/button';
import { Plus, ClipboardCheck, Edit, Trash2 } from 'lucide-react';

interface DviTemplate {
  id: string;
  name: string;
  is_active: boolean;
  sections: { section: string; items: { key: string; label: string; type: string }[] }[];
  created_at: string;
}

function useDVITemplates() {
  return useQuery({
    queryKey: ['dvi-templates'],
    queryFn: async () => {
      const response = await api.get<{ data: DviTemplate[] }>('/v1/dvi/templates');
      return response.data;
    },
  });
}

export default function DVITemplateListPage() {
  const { data, isLoading, error } = useDVITemplates();

  return (
    <div className="space-y-4">
      <PageHeader
        title="DVI Templates"
        description="Digital Vehicle Inspection templates"
        actions={
          <Button asChild>
            <Link to="/dvi/templates/new">
              <Plus className="h-4 w-4 mr-1" /> New Template
            </Link>
          </Button>
        }
      />

      {isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {error && <EmptyState icon="default" title="Error loading templates" description="Please try again later" />}
      {!isLoading && !error && (!data?.data || data.data.length === 0) && (
        <EmptyState icon="default" title="No DVI templates" description="Create your first template" action={{ label: 'New Template', onClick: () => {} }} />
      )}

      {!isLoading && !error && data?.data && data.data.length > 0 && (
        <div className="rounded-lg border border-border bg-surface overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Template Name</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Sections</th>
                <th className="text-left p-3 text-sm font-medium text-muted-foreground">Created</th>
                <th className="text-right p-3 text-sm font-medium text-muted-foreground">Actions</th>
              </tr>
            </thead>
            <tbody>
              {data.data.map(template => (
                <tr key={template.id} className="border-b border-border last:border-0 hover:bg-surface-raised">
                  <td className="p-3">
                    <Link to={`/dvi/templates/${template.id}`} className="flex items-center gap-2 hover:text-accent">
                      <ClipboardCheck className="h-4 w-4 text-muted-foreground" />
                      <span className="font-medium">{template.name}</span>
                    </Link>
                  </td>
                  <td className="p-3 text-sm">
                    <span className={template.is_active ? 'text-success' : 'text-muted-foreground'}>
                      {template.is_active ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td className="p-3 text-sm">{template.sections?.length || 0} sections</td>
                  <td className="p-3 text-sm text-muted-foreground">{new Date(template.created_at).toLocaleDateString()}</td>
                  <td className="p-3 text-right">
                    <div className="flex gap-2 justify-end">
                      <Button variant="ghost" size="sm">
                        <Edit className="h-4 w-4" />
                      </Button>
                      <Button variant="ghost" size="sm">
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
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