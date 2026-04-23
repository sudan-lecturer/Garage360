import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from '@/store/auth';
import { MainLayout } from '@/layouts/MainLayout';
import { AuthLayout } from '@/layouts/AuthLayout';
import { LoginPage } from '@/modules/auth/pages/LoginPage';
import DashboardPage from '@/modules/dashboard/DashboardPage';
import CustomerListPage from '@/modules/customers/pages/CustomerList';
import CustomerDetailPage from '@/modules/customers/pages/CustomerDetail';
import CustomerFormPage from '@/modules/customers/pages/CustomerForm';
import VehicleListPage from '@/modules/vehicles/pages/VehicleList';
import VehicleDetailPage from '@/modules/vehicles/pages/VehicleDetail';

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
        <Route path="/customers" element={<CustomerListPage />} />
        <Route path="/customers/new" element={<CustomerFormPage />} />
        <Route path="/customers/:id" element={<CustomerDetailPage />} />
        <Route path="/customers/:id/edit" element={<CustomerFormPage />} />
        <Route path="/vehicles" element={<VehicleListPage />} />
        <Route path="/vehicles/new" element={<div>New Vehicle</div>} />
        <Route path="/vehicles/:id" element={<VehicleDetailPage />} />
        <Route path="/vehicles/:id/edit" element={<div>Edit Vehicle</div>} />
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
