import { Outlet } from 'react-router-dom';

export function AuthLayout() {
  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <p className="text-xs uppercase tracking-[0.2em] text-accent">Operator Access</p>
          <h1 className="mt-2 text-3xl font-bold uppercase tracking-[0.08em] text-primary">Garage360</h1>
          <p className="text-muted-foreground mt-2">Workshop Command Console</p>
        </div>
        <div className="bg-surface rounded-sm border border-border p-6">
          <Outlet />
        </div>
      </div>
    </div>
  );
}
