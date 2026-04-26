import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { Button } from '@/components/ui/button';
import { useCreateDviResult, useDviTemplates } from '@/api/hooks/useDvi';

export default function DVIResultCreatePage() {
  const navigate = useNavigate();
  const { data: templatesData } = useDviTemplates();
  const createMutation = useCreateDviResult();
  const [jobCardId, setJobCardId] = useState('');
  const [templateId, setTemplateId] = useState('');
  const [jsonData, setJsonData] = useState('{\n  "summary": "Inspection completed"\n}');

  const templates = useMemo(() => templatesData ?? [], [templatesData]);

  const onSubmit = () => {
    let parsed: unknown;
    try {
      parsed = JSON.parse(jsonData);
    } catch {
      return;
    }

    createMutation.mutate(
      { job_card_id: jobCardId.trim(), template_id: templateId || undefined, data: parsed },
      {
        onSuccess: (response) => {
          if (response?.id) navigate(`/dvi/results/${response.id}`);
        },
      }
    );
  };

  return (
    <div className="space-y-4">
      <PageHeader title="DVI Result Entry" description="Create inspection result and submit to job card." />
      <div className="rounded-sm border border-border bg-surface p-4 space-y-3">
        <input className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm" placeholder="Job Card ID" value={jobCardId} onChange={(e) => setJobCardId(e.target.value)} />
        <select className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm" value={templateId} onChange={(e) => setTemplateId(e.target.value)}>
          <option value="">Select template (optional)</option>
          {templates.map((template) => (
            <option key={template.id} value={template.id}>{template.name}</option>
          ))}
        </select>
        <textarea className="min-h-40 w-full rounded-sm border border-input bg-background p-3 font-mono text-sm" value={jsonData} onChange={(e) => setJsonData(e.target.value)} />
        <div className="flex justify-end">
          <Button onClick={onSubmit} disabled={createMutation.isPending || !jobCardId.trim()}>Submit Result</Button>
        </div>
      </div>
    </div>
  );
}
