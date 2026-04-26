import { useMemo, useState } from 'react';
import { AxiosError } from 'axios';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { EmptyState } from '@/components/shared/empty-state';
import { Button } from '@/components/ui/button';
import {
  useCreateSavedReport,
  useDeleteSavedReport,
  useExportReport,
  useGenerateReport,
  useSavedReports,
  type ReportType,
  type ExportFormat,
} from '@/api/hooks/useReports';


export default function ReportsDashboardPage() {
  const [reportType, setReportType] = useState<ReportType>('revenue');
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');
  const [saveName, setSaveName] = useState('');
  const [latestResult, setLatestResult] = useState<string>('');
  const [errorMessage, setErrorMessage] = useState('');

  const savedReportsQuery = useSavedReports();

  const config = useMemo(
    () => ({
      fromDate,
      toDate,
    }),
    [fromDate, toDate]
  );

  const generateMutation = useGenerateReport();
  const exportMutation = useExportReport();
  const saveMutation = useCreateSavedReport();
  const deleteMutation = useDeleteSavedReport();

  const handleMutationError = (error: unknown, fallback: string) => {
    const typedError = error as AxiosError<{ error?: { message?: string } }>;
    const message = typedError.response?.data?.error?.message ?? fallback;
    setErrorMessage(message);
  };

  const onGenerate = () => {
    generateMutation.mutate(
      { reportType, config },
      {
        onSuccess: (data) => {
          setErrorMessage('');
          setLatestResult(JSON.stringify(data, null, 2));
        },
        onError: (error) => handleMutationError(error, 'Failed to generate report.'),
      }
    );
  };

  const onExport = (format: ExportFormat) => {
    exportMutation.mutate(
      { reportType, config, format },
      {
        onSuccess: (data) => {
          setErrorMessage('');
          setLatestResult(JSON.stringify(data, null, 2));
        },
        onError: (error) => handleMutationError(error, 'Failed to export report.'),
      }
    );
  };

  const onSavePreset = () => {
    saveMutation.mutate(
      { name: saveName, reportType, config },
      {
        onSuccess: () => {
          setErrorMessage('');
          setSaveName('');
        },
        onError: (error) => handleMutationError(error, 'Failed to save report preset.'),
      }
    );
  };

  const onDeletePreset = (id: string) => {
    deleteMutation.mutate(id, {
      onSuccess: () => {
        setErrorMessage('');
      },
      onError: (error) => handleMutationError(error, 'Failed to delete saved report.'),
    });
  };

  const reportTypeOptions = [
    { value: 'revenue', label: 'Revenue Summary' },
    { value: 'jobs', label: 'Job Status Summary' },
    { value: 'customers', label: 'Customer Activity' },
  ] as const;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Reports Dashboard"
        description="Generate, export, and save commonly used reports."
      />

      <div className="grid gap-6 lg:grid-cols-2">
        <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
          <h2 className="text-lg font-semibold text-foreground">Generate Report</h2>

          <div className="space-y-2">
            <label htmlFor="reportType" className="text-sm font-medium">
              Report Type
            </label>
            <select
              id="reportType"
              value={reportType}
              onChange={(event) => setReportType(event.target.value as ReportType)}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
            >
              {reportTypeOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <div className="space-y-2">
              <label htmlFor="fromDate" className="text-sm font-medium">
                From Date
              </label>
              <input
                id="fromDate"
                type="date"
                value={fromDate}
                onChange={(event) => setFromDate(event.target.value)}
                className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
              />
            </div>
            <div className="space-y-2">
              <label htmlFor="toDate" className="text-sm font-medium">
                To Date
              </label>
              <input
                id="toDate"
                type="date"
                value={toDate}
                onChange={(event) => setToDate(event.target.value)}
                className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
              />
            </div>
          </div>

          <div className="flex flex-wrap gap-2">
            <Button type="button" onClick={onGenerate} disabled={generateMutation.isPending}>
              {generateMutation.isPending ? 'Generating...' : 'Generate'}
            </Button>
            <Button
              type="button"
              variant="secondary"
              onClick={() => onExport('json')}
              disabled={exportMutation.isPending}
            >
              Export JSON
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => onExport('csv')}
              disabled={exportMutation.isPending}
            >
              Export CSV
            </Button>
          </div>
        </section>

        <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
          <h2 className="text-lg font-semibold text-foreground">Save Preset</h2>
          <div className="space-y-2">
            <label htmlFor="saveName" className="text-sm font-medium">
              Preset Name
            </label>
            <input
              id="saveName"
              value={saveName}
              onChange={(event) => setSaveName(event.target.value)}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
              placeholder="Monthly revenue snapshot"
            />
          </div>
          <Button
            type="button"
            onClick={onSavePreset}
            disabled={saveMutation.isPending || saveName.trim().length === 0}
          >
            {saveMutation.isPending ? 'Saving...' : 'Save Preset'}
          </Button>

          {savedReportsQuery.isLoading && (
            <div className="py-8">
              <LoadingSpinner />
            </div>
          )}

          {!savedReportsQuery.isLoading && (savedReportsQuery.data?.length ?? 0) === 0 && (
            <EmptyState
              icon="default"
              title="No saved reports"
              description="Save frequently used report filters for quick access."
            />
          )}

          {!savedReportsQuery.isLoading && (savedReportsQuery.data?.length ?? 0) > 0 && (
            <div className="space-y-2">
              {savedReportsQuery.data?.map((report) => (
                <div
                  key={report.id}
                  className="flex items-center justify-between rounded-md border border-border p-3"
                >
                  <div>
                    <p className="font-medium text-sm">{report.name}</p>
                    <p className="text-xs text-muted-foreground uppercase tracking-wide">
                      {report.reportType}
                    </p>
                  </div>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => onDeletePreset(report.id)}
                    disabled={deleteMutation.isPending}
                  >
                    Delete
                  </Button>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>

      {errorMessage && (
        <div className="rounded-md border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
          {errorMessage}
        </div>
      )}

      {latestResult && (
        <section className="rounded-lg border border-border bg-surface p-4 space-y-2">
          <h2 className="text-lg font-semibold text-foreground">Latest Output</h2>
          <pre className="max-h-96 overflow-auto rounded-md bg-background p-3 text-xs text-foreground">
            {latestResult}
          </pre>
        </section>
      )}
    </div>
  );
}
