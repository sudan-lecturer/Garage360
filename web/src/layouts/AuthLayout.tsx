import { Outlet } from 'react-router-dom';

export function AuthLayout() {
  return (
    <div className="min-h-screen bg-primary flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-accent">Garage360</h1>
          <p className="text-muted-foreground mt-2">Workshop Management System</p>
        </div>
        <div className="bg-card rounded-lg border border-border p-6">
          <Outlet />
        </div>
      </div>
    </div>
  );
}
