import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useJob } from '@/api/hooks/useJobs';
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
  const [photos, setPhotos] = useState<string[]>([]);
  const [signature, setSignature] = useState<string | null>(null);

  const { data: job, isLoading } = useJob(id!);

  const handleChecklistChange = (key: string, value: boolean) => {
    setChecklist(prev => ({ ...prev, [key]: value }));
  };

  const handlePhotoCapture = () => {
    // Simulated photo capture - in real app would use react-webcam
    const newPhoto = `photo_${Date.now()}.jpg`;
    setPhotos(prev => [...prev, newPhoto]);
  };

  const handleSubmit = async () => {
    console.log('Submitting intake:', { checklist, odometer, photos, signature });
    // TODO: API call
    navigate(`/jobs/${id}`);
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
            {photos.map((_, idx) => (
              <div key={idx} className="aspect-video rounded-md bg-surface-raised flex items-center justify-center">
                <Camera className="h-8 w-8 text-muted-foreground" />
              </div>
            ))}
            <button
              type="button"
              onClick={handlePhotoCapture}
              className="aspect-video rounded-md border-2 border-dashed border-border hover:border-accent flex items-center justify-center"
            >
              <Camera className="h-8 w-8 text-muted-foreground" />
            </button>
          </div>

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
            {signature ? (
              <p className="text-success">Signature captured</p>
            ) : (
              <p className="text-muted-foreground">Customer signature area</p>
            )}
          </div>

          <div className="flex gap-2 mb-4">
            <Button variant="outline" size="sm" onClick={() => setSignature('signed')}>
              Sign Here
            </Button>
            <Button variant="outline" size="sm" onClick={() => setSignature(null)}>
              Clear
            </Button>
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
                {signature ? 'Signed' : 'Not signed'}
              </p>
            </div>
          </div>

          <div className="flex justify-between mt-6">
            <Button variant="outline" onClick={() => setStep(3)}>← Back</Button>
            <Button onClick={handleSubmit}>
              <Save className="h-4 w-4 mr-1" /> Complete Intake
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}