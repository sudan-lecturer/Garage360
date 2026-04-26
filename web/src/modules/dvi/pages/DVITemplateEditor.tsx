import { useEffect, useState } from 'react';
import { AxiosError } from 'axios';
import { useNavigate, useParams } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { Button } from '@/components/ui/button';
import { useCreateDviTemplate, useDviTemplates, useUpdateDviTemplate } from '@/api/hooks/useDvi';

interface SectionDraft {
  section: string;
  items: Array<{ key: string; label: string; type: string }>;
}

const blankSection = (): SectionDraft => ({ section: '', items: [{ key: '', label: '', type: 'PASS_FAIL' }] });

export default function DVITemplateEditorPage() {
  const { id } = useParams();
  const editing = Boolean(id);
  const navigate = useNavigate();
  const [name, setName] = useState('');
  const [sections, setSections] = useState<SectionDraft[]>([blankSection()]);
  const [errorMessage, setErrorMessage] = useState('');

  const templatesQuery = useDviTemplates();
  const createMutation = useCreateDviTemplate();
  const updateMutation = useUpdateDviTemplate();

  useEffect(() => {
    if (!editing || !id || !templatesQuery.data || name.length > 0) return;
    const current = templatesQuery.data.find((t) => t.id === id);
    if (!current) return;
    setName(current.name);
    const loaded = Array.isArray(current.sections) ? (current.sections as SectionDraft[]) : [blankSection()];
    setSections(loaded.length > 0 ? loaded : [blankSection()]);
  }, [editing, id, templatesQuery.data, name.length]);

  const onSubmit = () => {
    setErrorMessage('');
    if (!name.trim()) {
      setErrorMessage('Template name is required.');
      return;
    }
    const payload = {
      name: name.trim(),
      sections,
    };

    if (editing && id) {
      updateMutation.mutate(
        { id, payload },
        {
          onSuccess: () => navigate('/dvi/templates'),
          onError: (error) => {
            const typed = error as AxiosError<{ error?: { message?: string } }>;
            setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to update template.');
          },
        }
      );
      return;
    }

    createMutation.mutate(payload, {
      onSuccess: () => navigate('/dvi/templates'),
      onError: (error) => {
        const typed = error as AxiosError<{ error?: { message?: string } }>;
        setErrorMessage(typed.response?.data?.error?.message ?? 'Failed to create template.');
      },
    });
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title={editing ? 'Edit DVI Template' : 'Create DVI Template'}
        description="Configure inspection sections and checklist items."
      />

      <section className="rounded-sm border border-border bg-surface p-4 space-y-4">
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm"
          placeholder="Template name"
        />

        {sections.map((section, sectionIdx) => (
          <div key={sectionIdx} className="space-y-2 rounded-sm border border-border p-3">
            <input
              value={section.section}
              onChange={(e) =>
                setSections((current) =>
                  current.map((s, i) => (i === sectionIdx ? { ...s, section: e.target.value } : s))
                )
              }
              className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm"
              placeholder="Section name"
            />
            {section.items.map((item, itemIdx) => (
              <div key={itemIdx} className="grid gap-2 sm:grid-cols-12">
                <input
                  value={item.key}
                  onChange={(e) =>
                    setSections((current) =>
                      current.map((s, i) =>
                        i === sectionIdx
                          ? {
                              ...s,
                              items: s.items.map((it, j) =>
                                j === itemIdx ? { ...it, key: e.target.value } : it
                              ),
                            }
                          : s
                      )
                    )
                  }
                  className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-3"
                  placeholder="Key"
                />
                <input
                  value={item.label}
                  onChange={(e) =>
                    setSections((current) =>
                      current.map((s, i) =>
                        i === sectionIdx
                          ? {
                              ...s,
                              items: s.items.map((it, j) =>
                                j === itemIdx ? { ...it, label: e.target.value } : it
                              ),
                            }
                          : s
                      )
                    )
                  }
                  className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-6"
                  placeholder="Label"
                />
                <select
                  value={item.type}
                  onChange={(e) =>
                    setSections((current) =>
                      current.map((s, i) =>
                        i === sectionIdx
                          ? {
                              ...s,
                              items: s.items.map((it, j) =>
                                j === itemIdx ? { ...it, type: e.target.value } : it
                              ),
                            }
                          : s
                      )
                    )
                  }
                  className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-3"
                >
                  <option value="PASS_FAIL">Pass / Fail</option>
                  <option value="TEXT">Text</option>
                  <option value="NUMERIC">Numeric</option>
                </select>
              </div>
            ))}
            <Button
              variant="outline"
              onClick={() =>
                setSections((current) =>
                  current.map((s, i) =>
                    i === sectionIdx
                      ? { ...s, items: [...s.items, { key: '', label: '', type: 'PASS_FAIL' }] }
                      : s
                  )
                )
              }
            >
              Add Item
            </Button>
          </div>
        ))}

        <Button variant="outline" onClick={() => setSections((current) => [...current, blankSection()])}>
          Add Section
        </Button>
      </section>

      {errorMessage && (
        <div className="rounded-sm border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
          {errorMessage}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <Button variant="outline" onClick={() => navigate('/dvi/templates')}>
          Cancel
        </Button>
        <Button onClick={onSubmit} disabled={createMutation.isPending || updateMutation.isPending}>
          {editing ? 'Save Template' : 'Create Template'}
        </Button>
      </div>
    </div>
  );
}
