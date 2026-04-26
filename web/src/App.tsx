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
const VehicleForm = lazy(() => import('@/modules/vehicles/pages/VehicleForm'));
const JobList = lazy(() => import('@/modules/jobs/pages/JobList'));
const JobDetail = lazy(() => import('@/modules/jobs/pages/JobDetail'));
const JobForm = lazy(() => import('@/modules/jobs/pages/JobForm'));
const IntakeFlow = lazy(() => import('@/modules/jobs/pages/IntakeFlow'));
const InventoryList = lazy(() => import('@/modules/inventory/pages/InventoryList'));
const InventoryDetail = lazy(() => import('@/modules/inventory/pages/InventoryDetail'));
const InventoryForm = lazy(() => import('@/modules/inventory/pages/InventoryForm'));
const POList = lazy(() => import('@/modules/purchases/pages/POList'));
const POCreate = lazy(() => import('@/modules/purchases/pages/POCreate'));
const PODetail = lazy(() => import('@/modules/purchases/pages/PODetail'));
const InvoiceList = lazy(() => import('@/modules/billing/pages/InvoiceList'));
const InvoiceDetail = lazy(() => import('@/modules/billing/pages/InvoiceDetail'));
const InvoiceCreate = lazy(() => import('@/modules/billing/pages/InvoiceCreate'));
const CouponManagement = lazy(() => import('@/modules/billing/pages/CouponManagement'));
const PaymentConfirmation = lazy(() => import('@/modules/billing/pages/PaymentConfirmation'));
const DVITemplateList = lazy(() => import('@/modules/dvi/pages/DVITemplateList'));
const DVITemplateEditor = lazy(() => import('@/modules/dvi/pages/DVITemplateEditor'));
const DVIResultCreate = lazy(() => import('@/modules/dvi/pages/DVIResultCreate'));
const DVIResultDetail = lazy(() => import('@/modules/dvi/pages/DVIResultDetail'));
const AssetList = lazy(() => import('@/modules/assets/pages/AssetList'));
const AssetCreate = lazy(() => import('@/modules/assets/pages/AssetCreate'));
const EmployeeList = lazy(() => import('@/modules/hr/pages/EmployeeList'));
const EmployeeDetail = lazy(() => import('@/modules/hr/pages/EmployeeDetail'));
const Settings = lazy(() => import('@/modules/settings/pages/Settings'));
const UserManagement = lazy(() => import('@/modules/settings/pages/UserManagement'));
const RoleManagement = lazy(() => import('@/modules/settings/pages/RoleManagement'));
const SuperAdminTenants = lazy(() => import('@/modules/super-admin/pages/Tenants'));
const ReportsDashboard = lazy(() => import('@/modules/reports/pages/ReportsDashboard'));
const QuoteCreate = lazy(() => import('@/modules/jobs/pages/QuoteCreate'));

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
          <Route path="/forgot-password" element={<Navigate to="/login" replace />} />
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
          <Route path="/vehicles/new" element={<VehicleForm />} />
          <Route path="/vehicles/:id" element={<VehicleDetail />} />
          <Route path="/vehicles/:id/edit" element={<VehicleForm />} />
          <Route path="/jobs" element={<JobList />} />
          <Route path="/jobs/new" element={<JobForm />} />
          <Route path="/jobs/:id" element={<JobDetail />} />
          <Route path="/jobs/:id/edit" element={<JobForm />} />
          <Route path="/jobs/:id/approve" element={<QuoteCreate />} />
          <Route path="/jobs/:id/qa" element={<Navigate to="/jobs" replace />} />
          <Route path="/jobs/:id/intake" element={<IntakeFlow />} />
          <Route path="/inventory" element={<InventoryList />} />
          <Route path="/inventory/:id" element={<InventoryDetail />} />
          <Route path="/inventory/new" element={<InventoryForm />} />
          <Route path="/purchases" element={<POList />} />
          <Route path="/purchases/new" element={<POCreate />} />
          <Route path="/purchases/:id" element={<PODetail />} />
          <Route path="/billing" element={<InvoiceList />} />
          <Route path="/billing/:id" element={<InvoiceDetail />} />
          <Route path="/billing/new" element={<InvoiceCreate />} />
          <Route path="/billing/coupons" element={<CouponManagement />} />
          <Route path="/billing/payment-confirmation" element={<PaymentConfirmation />} />
          <Route path="/dvi/templates" element={<DVITemplateList />} />
          <Route path="/dvi/templates/:id" element={<DVITemplateEditor />} />
          <Route path="/dvi/templates/new" element={<DVITemplateEditor />} />
          <Route path="/dvi/results/new" element={<DVIResultCreate />} />
          <Route path="/dvi/results/:id" element={<DVIResultDetail />} />
          <Route path="/assets" element={<AssetList />} />
          <Route path="/assets/new" element={<AssetCreate />} />
          <Route path="/hr/employees" element={<EmployeeList />} />
          <Route path="/hr/employees/:id" element={<EmployeeDetail />} />
          <Route path="/reports" element={<ReportsDashboard />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/settings/users" element={<UserManagement />} />
          <Route path="/settings/roles" element={<RoleManagement />} />
          <Route path="/control/tenants" element={<SuperAdminTenants />} />
        </Route>

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Suspense>
  );
}