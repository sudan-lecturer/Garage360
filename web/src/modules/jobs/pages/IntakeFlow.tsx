import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  useIntakeSnapshot,
  useJob,
  useSaveCustomerSignature,
  useSaveIntakeChecklist,
  useUpdateJob,
  useUploadIntakePhoto,
} from '@/api/hooks/useJobs';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { Button } from '@/components/ui/button';
import { Save, Camera, FileSignature, CheckCircle } from 'lucide-react';

const checklistItems = [
  { key: 'keys', label: 'Keys received', required: true },
  { key: 'tyres', label: 'Tyres condition', required: true },
  { key: 'engine', label: 'Engine noise', required: true },
  { key: 'odometer', label: 'Odometer reading', required: true },
  { key: 'lights', label: 'Lights functional', required: true },
  { key: 'belongings', label: 'Personal belongings noted', required: false },
  { key: 'damage', label: 'Existing damage noted', required: false },
];

export default function IntakeFlowPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  
  const [step, setStep] = useState(1);
  const [checklist, setChecklist] = useState<Record<string, boolean>>({});
  const [odometer, setOdometer] = useState('');
  const [photos, setPhotos] = useState<Array<{ file: File; preview: string }>>([]);
  const [signatureFile, setSignatureFile] = useState<File | null>(null);
  const [signedBy, setSignedBy] = useState('');
  const [submitError, setSubmitError] = useState('');

  const { data: job, isLoading } = useJob(id!);
  const intakeSnapshot = useIntakeSnapshot(id);
  const updateJobMutation = useUpdateJob();
  const checklistMutation = useSaveIntakeChecklist();
  const uploadPhotoMutation = useUploadIntakePhoto();
  const signatureMutation = useSaveCustomerSignature();

  const handleChecklistChange = (key: string, value: boolean) => {
    setChecklist(prev => ({ ...prev, [key]: value }));
  };

  const handleSubmit = async () => {
    if (!id) return;
    setSubmitError('');
    try {
      await updateJobMutation.mutateAsync({
        id,
        odometer_in: odometer ? Number(odometer) : undefined,
      } as any);

      await checklistMutation.mutateAsync({
        id,
        data: checklist,
        completed: true,
      });

      for (const entry of photos) {
        await uploadPhotoMutation.mutateAsync({
          id,
          photoType: 'VEHICLE',
          fileName: entry.file.name,
          mimeType: entry.file.type || 'image/jpeg',
          imageBase64: await fileToBase64(entry.file),
        });
      }

      if (signatureFile) {
        await signatureMutation.mutateAsync({
          id,
          signatureType: 'CUSTOMER',
          signedBy: signedBy.trim() || 'Customer',
          mimeType: signatureFile.type || 'image/png',
          imageBase64: await fileToBase64(signatureFile),
        });
      }

      navigate(`/jobs/${id}`);
    } catch {
      setSubmitError('Failed to save intake details. Please try again.');
    }
  };

  if (isLoading) {
    return (
      <div className="py-12">
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <PageHeader
        title="Intake Inspection"
        description={`Job #${job?.job_number}`}
        breadcrumbs={[
          { label: 'Jobs', href: '/jobs' },
          { label: job?.job_number || '', href: `/jobs/${id}` },
          { label: 'Intake' },
        ]}
      />

      {/* Progress Steps */}
      <div className="flex items-center justify-center gap-2 mb-6">
        {[1, 2, 3, 4].map(s => (
          <div key={s} className="flex items-center">
            <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
              step >= s ? 'bg-accent text-accent-foreground' : 'bg-surface-raised text-muted-foreground'
            }`}>
              {s}
            </div>
            {s < 4 && <div className={`w-8 h-px ${step > s ? 'bg-accent' : 'bg-border'}`} />}
          </div>
        ))}
      </div>

      {/* Step 1: Checklist */}
      {step === 1 && (
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4">Step 1: Intake Checklist</h3>
          <div className="space-y-3">
            {checklistItems.map(item => (
              <label
                key={item.key}
                className="flex items-center gap-3 p-3 rounded-md border border-border hover:border-accent cursor-pointer"
              >
                <input
                  type="checkbox"
                  checked={checklist[item.key] || false}
                  onChange={(e) => handleChecklistChange(item.key, e.target.checked)}
                  className="h-5 w-5 rounded border-input"
                />
                <span className="flex-1">
                  {item.label}
                  {item.required && <span className="text-destructive ml-1">*</span>}
                </span>
              </label>
            ))}
            
            <div className="mt-4">
              <label className="block text-sm font-medium mb-2">Odometer Reading</label>
              <input
                type="number"
                value={odometer}
                onChange={(e) => setOdometer(e.target.value)}
                placeholder="Enter odometer reading"
                className="h-10 w-full rounded-md border border-input bg-background px-3"
              />
            </div>
          </div>

          <div className="flex justify-end mt-6">
            <Button onClick={() => setStep(2)}>Next: Photos →</Button>
          </div>
        </div>
      )}

      {/* Step 2: Photos */}
      {step === 2 && (
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Camera className="h-5 w-5" /> Step 2: Photos
          </h3>
          
          <div className="grid grid-cols-3 gap-4 mb-4">
            {photos.map((photo, idx) => (
              <div key={idx} className="aspect-video rounded-md bg-surface-raised flex items-center justify-center">
                <img src={photo.preview} alt={`Vehicle ${idx + 1}`} className="h-full w-full object-cover rounded-md" />
              </div>
            ))}
          </div>
          <input
            type="file"
            accept="image/*"
            multiple
            onChange={(event) => {
              const files = Array.from(event.target.files ?? []);
              const mapped = files.map((file) => ({
                file,
                preview: URL.createObjectURL(file),
              }));
              setPhotos((prev) => [...prev, ...mapped].slice(0, 10));
            }}
            className="mb-3 block w-full text-sm text-muted-foreground"
          />

          <p className="text-sm text-muted-foreground mb-4">
            Take photos of vehicle condition (max 10 photos)
          </p>

          <div className="flex justify-between">
            <Button variant="outline" onClick={() => setStep(1)}>← Back</Button>
            <Button onClick={() => setStep(3)}>Next: Signature →</Button>
          </div>
        </div>
      )}

      {/* Step 3: Signature */}
      {step === 3 && (
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <FileSignature className="h-5 w-5" /> Step 3: Customer Signature
          </h3>
          
          <div className="border-2 border-dashed border-border rounded-lg h-48 flex items-center justify-center mb-4">
            {signatureFile || intakeSnapshot.data?.signature ? (
              <p className="text-success">Signature ready</p>
            ) : (
              <p className="text-muted-foreground">Upload customer signature image</p>
            )}
          </div>

          <div className="mb-4 space-y-2">
            <input
              type="text"
              value={signedBy}
              onChange={(e) => setSignedBy(e.target.value)}
              placeholder="Signed by"
              className="h-10 w-full rounded-md border border-input bg-background px-3"
            />
            <input
              type="file"
              accept="image/*"
              onChange={(event) => {
                const file = event.target.files?.[0] ?? null;
                setSignatureFile(file);
              }}
              className="block w-full text-sm text-muted-foreground"
            />
            {intakeSnapshot.data?.signature && (
              <p className="text-xs text-muted-foreground">
                Existing signature already on file.
              </p>
            )}
          </div>

          <div className="flex justify-between">
            <Button variant="outline" onClick={() => setStep(2)}>← Back</Button>
            <Button onClick={() => setStep(4)}>Next: Review →</Button>
          </div>
        </div>
      )}

      {/* Step 4: Confirm */}
      {step === 4 && (
        <div className="rounded-lg border border-border bg-surface p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <CheckCircle className="h-5 w-5" /> Step 4: Review & Confirm
          </h3>
          
          <div className="space-y-4">
            <div className="p-4 rounded-md bg-surface-raised">
              <h4 className="font-medium mb-2">Checklist</h4>
              <p className="text-sm text-muted-foreground">
                {Object.values(checklist).filter(Boolean).length} / {checklistItems.length} completed
              </p>
            </div>
            
            <div className="p-4 rounded-md bg-surface-raised">
              <h4 className="font-medium mb-2">Photos</h4>
              <p className="text-sm text-muted-foreground">{photos.length} photos captured</p>
            </div>
            
            <div className="p-4 rounded-md bg-surface-raised">
              <h4 className="font-medium mb-2">Signature</h4>
              <p className="text-sm text-muted-foreground">
                {signatureFile || intakeSnapshot.data?.signature ? 'Signed' : 'Not signed'}
              </p>
            </div>
          </div>

          {submitError && (
            <div className="mt-4 rounded-sm border border-destructive bg-destructive-muted p-3 text-sm text-destructive">
              {submitError}
            </div>
          )}
          <div className="flex justify-between mt-6">
            <Button variant="outline" onClick={() => setStep(3)}>← Back</Button>
            <Button
              onClick={handleSubmit}
              disabled={
                updateJobMutation.isPending ||
                checklistMutation.isPending ||
                uploadPhotoMutation.isPending ||
                signatureMutation.isPending
              }
            >
              <Save className="h-4 w-4 mr-1" /> Complete Intake
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

async function fileToBase64(file: File): Promise<string> {
  return await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error('File read failed'));
    reader.readAsDataURL(file);
  });
}