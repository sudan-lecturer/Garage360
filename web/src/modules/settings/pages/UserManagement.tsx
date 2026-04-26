import { useMemo, useState } from 'react';
import { AxiosError } from 'axios';
import { Button } from '@/components/ui/button';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { SearchInput } from '@/components/shared/search-input';
import {
  useCreateSettingsUser,
  useDeleteSettingsUser,
  useSettingsUsers,
  useUpdateSettingsUser,
} from '@/api/hooks/useSettings';

const roleOptions = ['OWNER', 'ADMIN', 'MANAGER', 'ACCOUNT_MGR', 'MECHANIC', 'CASHIER', 'HR_OFFICER'];

const emptyFormState = {
  id: '',
  email: '',
  name: '',
  role: 'MANAGER',
  password: '',
  isActive: true,
};

export default function UserManagementPage() {
  const usersQuery = useSettingsUsers();
  const [search, setSearch] = useState('');
  const [formState, setFormState] = useState(emptyFormState);
  const [errorMessage, setErrorMessage] = useState('');

  const filteredUsers = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (query.length === 0) {
      return usersQuery.data ?? [];
    }

    return (usersQuery.data ?? []).filter((user) => {
      return (
        user.name.toLowerCase().includes(query) ||
        user.email.toLowerCase().includes(query) ||
        user.role.toLowerCase().includes(query)
      );
    });
  }, [search, usersQuery.data]);

  const createMutation = useCreateSettingsUser();
  const updateMutation = useUpdateSettingsUser();
  const deleteMutation = useDeleteSettingsUser();

  const submitForm = () => {
    setErrorMessage('');
    if (formState.email.trim().length === 0 || formState.name.trim().length === 0) {
      setErrorMessage('Name and email are required.');
      return;
    }

    if (formState.id.length === 0 && formState.password.trim().length < 8) {
      setErrorMessage('Password must be at least 8 characters for new users.');
      return;
    }

    if (formState.id.length > 0) {
      updateMutation.mutate(
        {
          id: formState.id,
          data: {
            email: formState.email.trim(),
            name: formState.name.trim(),
            role: formState.role,
            password: formState.password.trim().length > 0 ? formState.password : undefined,
            isActive: formState.isActive,
          },
        },
        {
          onSuccess: () => {
            setErrorMessage('');
            setFormState(emptyFormState);
          },
          onError: (error) => {
            const typed = error as AxiosError<{ error?: { message?: string } }>;
            const message = typed.response?.data?.error?.message ?? 'Failed to update user.';
            setErrorMessage(message);
          },
        }
      );
      return;
    }

    createMutation.mutate(
      {
        email: formState.email.trim(),
        name: formState.name.trim(),
        role: formState.role,
        password: formState.password,
        isActive: formState.isActive,
      },
      {
        onSuccess: () => {
          setErrorMessage('');
          setFormState(emptyFormState);
        },
        onError: (error) => {
          const typed = error as AxiosError<{ error?: { message?: string } }>;
          const message = typed.response?.data?.error?.message ?? 'Failed to create user.';
          setErrorMessage(message);
        },
      }
    );
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="User Management"
        description="Create, update, and deactivate workshop users."
        breadcrumbs={[
          { label: 'Settings', href: '/settings' },
          { label: 'User Management' },
        ]}
      />

      <div className="grid gap-6 lg:grid-cols-2">
        <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
          <h2 className="text-lg font-semibold">{formState.id ? 'Edit User' : 'Create User'}</h2>
          <div className="space-y-2">
            <label htmlFor="name" className="text-sm font-medium">
              Full Name
            </label>
            <input
              id="name"
              value={formState.name}
              onChange={(event) => setFormState((current) => ({ ...current, name: event.target.value }))}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="email" className="text-sm font-medium">
              Email
            </label>
            <input
              id="email"
              type="email"
              value={formState.email}
              onChange={(event) => setFormState((current) => ({ ...current, email: event.target.value }))}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="role" className="text-sm font-medium">
              Role
            </label>
            <select
              id="role"
              value={formState.role}
              onChange={(event) => setFormState((current) => ({ ...current, role: event.target.value }))}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
            >
              {roleOptions.map((role) => (
                <option key={role} value={role}>
                  {role}
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-2">
            <label htmlFor="password" className="text-sm font-medium">
              Password {formState.id ? '(optional for update)' : ''}
            </label>
            <input
              id="password"
              type="password"
              value={formState.password}
              onChange={(event) =>
                setFormState((current) => ({ ...current, password: event.target.value }))
              }
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
            />
          </div>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={formState.isActive}
              onChange={(event) =>
                setFormState((current) => ({ ...current, isActive: event.target.checked }))
              }
            />
            Active
          </label>

          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              onClick={submitForm}
              disabled={createMutation.isPending || updateMutation.isPending}
            >
              {formState.id ? 'Update User' : 'Create User'}
            </Button>
            <Button type="button" variant="outline" onClick={() => setFormState(emptyFormState)}>
              Reset
            </Button>
          </div>
        </section>

        <section className="rounded-lg border border-border bg-surface p-4 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Users</h2>
            <div className="w-56">
              <SearchInput value={search} onChange={setSearch} placeholder="Search users..." />
            </div>
          </div>

          {usersQuery.isLoading && <LoadingSpinner />}

          {!usersQuery.isLoading && filteredUsers.length === 0 && (
            <p className="text-sm text-muted-foreground">No users found.</p>
          )}

          {!usersQuery.isLoading && filteredUsers.length > 0 && (
            <div className="space-y-2">
              {filteredUsers.map((user) => (
                <div
                  key={user.id}
                  className="rounded-md border border-border p-3 flex items-center justify-between gap-3"
                >
                  <div>
                    <p className="font-medium text-sm">{user.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {user.email} • {user.role} • {user.isActive ? 'Active' : 'Inactive'}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        setFormState({
                          id: user.id,
                          email: user.email,
                          name: user.name,
                          role: user.role,
                          password: '',
                          isActive: user.isActive,
                        })
                      }
                    >
                      Edit
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        deleteMutation.mutate(user.id, {
                          onSuccess: () => {
                            setErrorMessage('');
                            if (formState.id.length > 0) setFormState(emptyFormState);
                          },
                          onError: (error) => {
                            const typed = error as AxiosError<{ error?: { message?: string } }>;
                            const message =
                              typed.response?.data?.error?.message ?? 'Failed to delete user.';
                            setErrorMessage(message);
                          },
                        })
                      }
                      disabled={deleteMutation.isPending}
                    >
                      Delete
                    </Button>
                  </div>
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
    </div>
  );
}
