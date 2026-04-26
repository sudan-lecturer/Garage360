import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { PageHeader } from '@/components/shared/page-header';
import { Button } from '@/components/ui/button';
import { Save, Settings as SettingsIcon, Users, MapPin, Bell, Shield } from 'lucide-react';
import {
  useCreateLocation,
  useFeatureFlags,
  useLocations,
  useNotificationPreferences,
  useUpdateFeatureFlag,
  useUpdateNotificationPreferences,
  useUpdateWorkshopProfile,
  useWorkshopProfile,
} from '@/api/hooks/useSettings';

const tabs = [
  { id: 'profile', label: 'Workshop', icon: SettingsIcon },
  { id: 'users', label: 'Users & Roles', icon: Users },
  { id: 'locations', label: 'Locations', icon: MapPin },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'flags', label: 'Feature Flags', icon: Shield },
];

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState('profile');
  const profileQuery = useWorkshopProfile();
  const updateProfileMutation = useUpdateWorkshopProfile();
  const locationsQuery = useLocations();
  const createLocationMutation = useCreateLocation();
  const flagsQuery = useFeatureFlags();
  const updateFlagMutation = useUpdateFeatureFlag();
  const notificationsQuery = useNotificationPreferences();
  const updateNotificationsMutation = useUpdateNotificationPreferences();
  const [profileForm, setProfileForm] = useState({ name: '', address: '', phone: '', email: '' });
  const [newLocation, setNewLocation] = useState({ name: '', address: '', isPrimary: false });
  const [notificationForm, setNotificationForm] = useState({
    jobUpdatesEmail: true,
    jobUpdatesSms: false,
    pushEnabled: true,
    lowStockAlerts: true,
    approvalAlerts: true,
    dailySummaryEmail: '',
  });

  useEffect(() => {
    if (profileQuery.data) {
      setProfileForm({
        name: profileQuery.data.name ?? '',
        address: profileQuery.data.address ?? '',
        phone: profileQuery.data.phone ?? '',
        email: profileQuery.data.email ?? '',
      });
    }
  }, [profileQuery.data]);

  useEffect(() => {
    if (notificationsQuery.data) {
      setNotificationForm({
        jobUpdatesEmail: notificationsQuery.data.jobUpdatesEmail,
        jobUpdatesSms: notificationsQuery.data.jobUpdatesSms,
        pushEnabled: notificationsQuery.data.pushEnabled,
        lowStockAlerts: notificationsQuery.data.lowStockAlerts,
        approvalAlerts: notificationsQuery.data.approvalAlerts,
        dailySummaryEmail: notificationsQuery.data.dailySummaryEmail ?? '',
      });
    }
  }, [notificationsQuery.data]);

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
                className={`w-full flex items-center gap-2 px-3 py-2 text-sm font-semibold uppercase tracking-wide rounded-sm border-l-2 transition-colors ${
                  activeTab === tab.id
                    ? 'bg-primary text-primary-foreground border-l-accent'
                    : 'text-muted-foreground border-l-transparent hover:bg-surface-raised hover:text-foreground hover:border-l-border-hover'
                }`}
              >
                <tab.icon className="h-4 w-4" />
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        {/* Content */}
        <div className="flex-1 rounded-sm border border-border bg-surface p-6">
          {activeTab === 'profile' && (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                updateProfileMutation.mutate(profileForm);
              }}
              className="space-y-4 max-w-lg"
            >
              <h3 className="text-lg font-semibold">Workshop Profile</h3>
              <input className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm" placeholder="Workshop name" value={profileForm.name} onChange={(e) => setProfileForm((v) => ({ ...v, name: e.target.value }))} />
              <textarea className="min-h-24 w-full rounded-sm border border-input bg-background p-3 text-sm" placeholder="Address" value={profileForm.address} onChange={(e) => setProfileForm((v) => ({ ...v, address: e.target.value }))} />
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                <input className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm" placeholder="Phone" value={profileForm.phone} onChange={(e) => setProfileForm((v) => ({ ...v, phone: e.target.value }))} />
                <input className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm" placeholder="Email" type="email" value={profileForm.email} onChange={(e) => setProfileForm((v) => ({ ...v, email: e.target.value }))} />
              </div>
              <Button type="submit" disabled={updateProfileMutation.isPending}>
                <Save className="h-4 w-4 mr-1" /> Save Changes
              </Button>
            </form>
          )}

          {activeTab === 'users' && (
            <div className="space-y-4">
              <h3 className="text-lg font-semibold mb-4">Users & Roles</h3>
              <p className="text-muted-foreground">Manage user accounts and role assignments</p>
              <div className="flex flex-wrap gap-2">
                <Button asChild>
                  <Link to="/settings/users">Open User Management</Link>
                </Button>
                <Button variant="outline" asChild>
                  <Link to="/settings/roles">Open Role Management</Link>
                </Button>
              </div>
            </div>
          )}

          {activeTab === 'locations' && (
            <div className="space-y-4">
              <h3 className="text-lg font-semibold mb-4">Locations</h3>
              <div className="rounded-sm border border-border bg-background">
                {(locationsQuery.data ?? []).map((location) => (
                  <div key={location.id} className="flex items-center justify-between border-b border-border p-3 last:border-b-0">
                    <div>
                      <p className="font-medium">{location.name}</p>
                      <p className="text-sm text-muted-foreground">{location.address || '-'}</p>
                    </div>
                    <p className="text-xs uppercase tracking-wide text-muted-foreground">
                      {location.isPrimary ? 'Primary' : 'Secondary'} / {location.isActive ? 'Active' : 'Inactive'}
                    </p>
                  </div>
                ))}
              </div>
              <div className="grid gap-2 sm:grid-cols-12">
                <input className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-4" placeholder="Location name" value={newLocation.name} onChange={(e) => setNewLocation((v) => ({ ...v, name: e.target.value }))} />
                <input className="h-10 rounded-sm border border-input bg-background px-3 text-sm sm:col-span-6" placeholder="Address" value={newLocation.address} onChange={(e) => setNewLocation((v) => ({ ...v, address: e.target.value }))} />
                <label className="flex items-center gap-2 text-sm sm:col-span-2">
                  <input type="checkbox" checked={newLocation.isPrimary} onChange={(e) => setNewLocation((v) => ({ ...v, isPrimary: e.target.checked }))} />
                  Primary
                </label>
              </div>
              <Button
                onClick={() => {
                  createLocationMutation.mutate({
                    name: newLocation.name,
                    address: newLocation.address || undefined,
                    isPrimary: newLocation.isPrimary,
                    isActive: true,
                  });
                  setNewLocation({ name: '', address: '', isPrimary: false });
                }}
                disabled={createLocationMutation.isPending || !newLocation.name.trim()}
              >
                Add Location
              </Button>
            </div>
          )}

          {activeTab === 'notifications' && (
            <div className="space-y-4">
              <h3 className="text-lg font-semibold mb-4">Notification Preferences</h3>
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                {([
                  ['jobUpdatesEmail', 'Job Updates Email'],
                  ['jobUpdatesSms', 'Job Updates SMS'],
                  ['pushEnabled', 'Push Notifications'],
                  ['lowStockAlerts', 'Low Stock Alerts'],
                  ['approvalAlerts', 'Approval Alerts'],
                ] as const).map(([key, label]) => (
                  <label key={key} className="flex items-center gap-2 text-sm">
                    <input
                      type="checkbox"
                      checked={notificationForm[key]}
                      onChange={(e) => setNotificationForm((v) => ({ ...v, [key]: e.target.checked }))}
                    />
                    {label}
                  </label>
                ))}
              </div>
              <input className="h-10 w-full rounded-sm border border-input bg-background px-3 text-sm" placeholder="Daily summary email (optional)" value={notificationForm.dailySummaryEmail} onChange={(e) => setNotificationForm((v) => ({ ...v, dailySummaryEmail: e.target.value }))} />
              <Button onClick={() => updateNotificationsMutation.mutate({ ...notificationForm, dailySummaryEmail: notificationForm.dailySummaryEmail || undefined })} disabled={updateNotificationsMutation.isPending}>
                Save Notification Settings
              </Button>
            </div>
          )}

          {activeTab === 'flags' && (
            <div className="space-y-4">
              <h3 className="text-lg font-semibold mb-4">Feature Flags</h3>
              <div className="rounded-sm border border-border bg-background">
                {(flagsQuery.data ?? []).map((flag) => (
                  <div key={flag.key} className="flex items-center justify-between border-b border-border p-3 last:border-b-0">
                    <div>
                      <p className="font-medium">{flag.key}</p>
                      <p className="text-sm text-muted-foreground">{flag.description || 'No description'}</p>
                    </div>
                    <label className="flex items-center gap-2 text-sm">
                      <input
                        type="checkbox"
                        checked={flag.isEnabled}
                        onChange={(e) => updateFlagMutation.mutate({ key: flag.key, isEnabled: e.target.checked })}
                      />
                      Enabled
                    </label>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}