import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { PageHeader } from '@/components/shared/page-header';
import { FormField, FormTextarea } from '@/components/shared/form-field';
import { Button } from '@/components/ui/button';
import { Save, Settings as SettingsIcon, Users, MapPin, Bell, Shield } from 'lucide-react';

const settingsSchema = z.object({
  workshop_name: z.string().min(1, 'Workshop name is required'),
  address: z.string().optional(),
  phone: z.string().optional(),
  email: z.string().email('Invalid email').optional(),
});

type SettingsForm = z.infer<typeof settingsSchema>;

const tabs = [
  { id: 'profile', label: 'Workshop', icon: SettingsIcon },
  { id: 'users', label: 'Users & Roles', icon: Users },
  { id: 'locations', label: 'Locations', icon: MapPin },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'flags', label: 'Feature Flags', icon: Shield },
];

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState('profile');
  const { register, handleSubmit, formState: { errors } } = useForm<SettingsForm>({
    resolver: zodResolver(settingsSchema),
    defaultValues: { workshop_name: 'Demo Workshop', address: '', phone: '', email: '' },
  });

  const onSubmit = async (data: SettingsForm) => {
    console.log('Saving settings:', data);
  };

  return (
    <div className="space-y-4">
      <PageHeader title="Settings" description="Configure your workshop" />

      <div className="flex flex-col lg:flex-row gap-6">
        {/* Sidebar Tabs */}
        <div className="lg:w-48 flex-shrink-0">
          <nav className="space-y-1">
            {tabs.map(tab => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`w-full flex items-center gap-2 px-3 py-2 text-sm font-medium rounded-md transition-colors ${
                  activeTab === tab.id
                    ? 'bg-accent text-accent-foreground'
                    : 'text-muted-foreground hover:bg-surface-raised hover:text-foreground'
                }`}
              >
                <tab.icon className="h-4 w-4" />
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        {/* Content */}
        <div className="flex-1 rounded-lg border border-border bg-surface p-6">
          {activeTab === 'profile' && (
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-6 max-w-lg">
              <h3 className="text-lg font-semibold">Workshop Profile</h3>
              <FormField name="workshop_name" label="Workshop Name" register={register} errors={errors} required />
              <FormTextarea name="address" label="Address" register={register} errors={errors} rows={3} />
              <div className="grid grid-cols-2 gap-4">
                <FormField name="phone" label="Phone" register={register} errors={errors} />
                <FormField name="email" label="Email" type="email" register={register} errors={errors} />
              </div>
              <Button type="submit">
                <Save className="h-4 w-4 mr-1" /> Save Changes
              </Button>
            </form>
          )}

          {activeTab === 'users' && (
            <div>
              <h3 className="text-lg font-semibold mb-4">Users & Roles</h3>
              <p className="text-muted-foreground">Manage user accounts and role assignments</p>
            </div>
          )}

          {activeTab === 'locations' && (
            <div>
              <h3 className="text-lg font-semibold mb-4">Locations</h3>
              <p className="text-muted-foreground">Manage workshop locations</p>
            </div>
          )}

          {activeTab === 'notifications' && (
            <div>
              <h3 className="text-lg font-semibold mb-4">Notification Preferences</h3>
              <p className="text-muted-foreground">Configure email and SMS notifications</p>
            </div>
          )}

          {activeTab === 'flags' && (
            <div>
              <h3 className="text-lg font-semibold mb-4">Feature Flags</h3>
              <p className="text-muted-foreground">Toggle features on/off for your workshop</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}