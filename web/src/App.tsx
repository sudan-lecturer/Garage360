import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from '@/store/auth';
import { MainLayout } from '@/layouts/MainLayout';
import { AuthLayout } from '@/layouts/AuthLayout';
import { LoginPage } from '@/modules/auth/pages/LoginPage';

export default function App() {
  const { isAuthenticated } = useAuthStore();

  return (
    <Routes>
      <Route element={<AuthLayout />}>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/forgot-password" element={<div>Forgot Password</div>} />
      </Route>

      <Route
        element={
          isAuthenticated ? <MainLayout /> : <Navigate to="/login" replace />
        }
      >
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="/dashboard" element={<DashboardPage />} />
        <Route path="/customers/*" element={<CustomersPage />} />
        <Route path="/vehicles/*" element={<VehiclesPage />} />
        <Route path="/jobs/*" element={<JobsPage />} />
        <Route path="/inventory/*" element={<InventoryPage />} />
        <Route path="/purchases/*" element={<PurchasesPage />} />
        <Route path="/billing/*" element={<BillingPage />} />
        <Route path="/dvi/*" element={<DVIPage />} />
        <Route path="/assets/*" element={<AssetsPage />} />
        <Route path="/hr/*" element={<HRPage />} />
        <Route path="/reports/*" element={<ReportsPage />} />
        <Route path="/settings/*" element={<SettingsPage />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

function DashboardPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Dashboard</h1></div>;
}
function CustomersPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Customers</h1></div>;
}
function VehiclesPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Vehicles</h1></div>;
}
function JobsPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Jobs</h1></div>;
}
function InventoryPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Inventory</h1></div>;
}
function PurchasesPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Purchases</h1></div>;
}
function BillingPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Billing</h1></div>;
}
function DVIPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">DVI</h1></div>;
}
function AssetsPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Assets</h1></div>;
}
function HRPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">HR</h1></div>;
}
function ReportsPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Reports</h1></div>;
}
function SettingsPage() {
  return <div className="p-6"><h1 className="text-2xl font-bold">Settings</h1></div>;
}
