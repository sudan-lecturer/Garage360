import { lazy, Suspense } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from '@/store/auth';
import { MainLayout } from '@/layouts/MainLayout';
import { AuthLayout } from '@/layouts/AuthLayout';
import { LoginPage } from '@/modules/auth/pages/LoginPage';
import { LoadingSpinner } from '@/components/shared/loading';

const Dashboard = lazy(() => import('@/modules/dashboard/DashboardPage'));
const CustomerList = lazy(() => import('@/modules/customers/pages/CustomerList'));
const CustomerDetail = lazy(() => import('@/modules/customers/pages/CustomerDetail'));
const CustomerForm = lazy(() => import('@/modules/customers/pages/CustomerForm'));
const VehicleList = lazy(() => import('@/modules/vehicles/pages/VehicleList'));
const VehicleDetail = lazy(() => import('@/modules/vehicles/pages/VehicleDetail'));
const JobList = lazy(() => import('@/modules/jobs/pages/JobList'));
const JobDetail = lazy(() => import('@/modules/jobs/pages/JobDetail'));
const JobForm = lazy(() => import('@/modules/jobs/pages/JobForm'));
const IntakeFlow = lazy(() => import('@/modules/jobs/pages/IntakeFlow'));
const InventoryList = lazy(() => import('@/modules/inventory/pages/InventoryList'));
const POList = lazy(() => import('@/modules/purchases/pages/POList'));
const InvoiceList = lazy(() => import('@/modules/billing/pages/InvoiceList'));
const DVITemplateList = lazy(() => import('@/modules/dvi/pages/DVITemplateList'));
const AssetList = lazy(() => import('@/modules/assets/pages/AssetList'));
const EmployeeList = lazy(() => import('@/modules/hr/pages/EmployeeList'));
const Settings = lazy(() => import('@/modules/settings/pages/Settings'));
const SuperAdminTenants = lazy(() => import('@/modules/super-admin/pages/Tenants'));

function PageLoader() {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <LoadingSpinner size="lg" />
    </div>
  );
}

export default function App() {
  const { isAuthenticated } = useAuthStore();

  return (
    <Suspense fallback={<PageLoader />}>
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
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/customers" element={<CustomerList />} />
          <Route path="/customers/new" element={<CustomerForm />} />
          <Route path="/customers/:id" element={<CustomerDetail />} />
          <Route path="/customers/:id/edit" element={<CustomerForm />} />
          <Route path="/vehicles" element={<VehicleList />} />
          <Route path="/vehicles/new" element={<div>New Vehicle</div>} />
          <Route path="/vehicles/:id" element={<VehicleDetail />} />
          <Route path="/vehicles/:id/edit" element={<div>Edit Vehicle</div>} />
          <Route path="/jobs" element={<JobList />} />
          <Route path="/jobs/new" element={<JobForm />} />
          <Route path="/jobs/:id" element={<JobDetail />} />
          <Route path="/jobs/:id/edit" element={<JobForm />} />
          <Route path="/jobs/:id/approve" element={<div>Approve Job</div>} />
          <Route path="/jobs/:id/qa" element={<div>QA Job</div>} />
          <Route path="/jobs/:id/intake" element={<IntakeFlow />} />
          <Route path="/inventory" element={<InventoryList />} />
          <Route path="/inventory/new" element={<div>Add Inventory</div>} />
          <Route path="/purchases" element={<POList />} />
          <Route path="/purchases/new" element={<div>New PO</div>} />
          <Route path="/billing" element={<InvoiceList />} />
          <Route path="/billing/new" element={<div>New Invoice</div>} />
          <Route path="/dvi/templates" element={<DVITemplateList />} />
          <Route path="/dvi/templates/:id" element={<div>DVI Template Detail</div>} />
          <Route path="/dvi/templates/new" element={<div>DVI Template Editor</div>} />
          <Route path="/assets" element={<AssetList />} />
          <Route path="/assets/new" element={<div>New Asset</div>} />
          <Route path="/hr/employees" element={<EmployeeList />} />
          <Route path="/hr/employees/:id" element={<div>Employee Detail</div>} />
          <Route path="/reports" element={<div className="p-6"><h1 className="text-2xl font-bold">Reports</h1></div>} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/control/tenants" element={<SuperAdminTenants />} />
        </Route>

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Suspense>
  );
}