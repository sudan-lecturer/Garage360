import { useParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { useDviResult } from '@/api/hooks/useDvi';

export default function DVIResultDetailPage() {
  const { id } = useParams();
  const query = useDviResult(id);

  return (
    <div className="space-y-4">
      <PageHeader title="DVI Result Detail" description="Inspection output and metadata." />
      {query.isLoading && <div className="py-12"><LoadingSpinner /></div>}
      {!query.isLoading && !query.data && <EmptyState title="DVI result not found" description="Unable to load result details." />}

      {query.data && <section className="rounded-sm border border-border bg-surface p-4">
        <dl className="grid gap-3 sm:grid-cols-2">
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Job Card ID</dt><dd>{query.data.jobCardId}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Template</dt><dd>{query.data.templateName || '-'}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Submitted By</dt><dd>{query.data.submittedBy || '-'}</dd></div>
          <div><dt className="text-xs uppercase tracking-wide text-muted-foreground">Created At</dt><dd>{query.data.createdAt ? new Date(query.data.createdAt).toLocaleString() : '-'}</dd></div>
        </dl>
      </section>}
      {query.data && <section className="rounded-sm border border-border bg-surface p-4">
        <h2 className="mb-3 text-lg font-semibold">Result Data</h2>
        <pre className="overflow-x-auto rounded-sm border border-border bg-background p-3 text-xs">{JSON.stringify(query.data.data, null, 2)}</pre>
      </section>}
    </div>
  );
}
