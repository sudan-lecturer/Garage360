import { Outlet, NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  Users,
  Car,
  Wrench,
  Package,
  ShoppingCart,
  Receipt,
  ClipboardCheck,
  Box,
  UsersRound,
  BarChart3,
  Settings,
  LogOut,
  Menu,
  X,
} from 'lucide-react';
import { useState } from 'react';
import { useAuthStore } from '@/store/auth';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Customers', href: '/customers', icon: Users },
  { name: 'Vehicles', href: '/vehicles', icon: Car },
  { name: 'Jobs', href: '/jobs', icon: Wrench },
  { name: 'Inventory', href: '/inventory', icon: Package },
  { name: 'Purchases', href: '/purchases', icon: ShoppingCart },
  { name: 'Billing', href: '/billing', icon: Receipt },
{ name: 'DVI', href: '/dvi/templates', icon: ClipboardCheck },
{ name: 'Assets', href: '/assets', icon: Box },
{ name: 'HR', href: '/hr/employees', icon: UsersRound },
  { name: 'Reports', href: '/reports', icon: BarChart3 },
  { name: 'Settings', href: '/settings', icon: Settings },
];

export function MainLayout() {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const { user, logout } = useAuthStore();

  return (
    <div className="min-h-screen bg-background">
      <nav className="fixed top-0 z-50 w-full bg-background border-b border-border">
        <div className="flex items-center justify-between px-4 h-16">
          <div className="flex items-center gap-4">
            <button
              className="lg:hidden"
              onClick={() => setSidebarOpen(!sidebarOpen)}
            >
              {sidebarOpen ? <X /> : <Menu />}
            </button>
            <h1 className="text-xl font-bold text-accent">Garage360</h1>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm text-muted-foreground hidden sm:block">
              {user?.name}
            </span>
            <Button variant="ghost" size="sm" onClick={logout}>
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </nav>

      <aside
        className={cn(
          'fixed top-16 left-0 z-40 w-64 h-[calc(100vh-4rem)] bg-background border-r border-border transition-transform lg:translate-x-0',
          sidebarOpen ? 'translate-x-0' : '-translate-x-full'
        )}
      >
        <nav className="p-4 space-y-1">
          {navigation.map((item) => (
            <NavLink
              key={item.href}
              to={item.href}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors',
                  isActive
                    ? 'bg-accent text-accent-foreground'
                    : 'text-muted-foreground hover:bg-surface-raised hover:text-foreground'
                )
              }
              onClick={() => setSidebarOpen(false)}
            >
              <item.icon className="h-5 w-5" />
              {item.name}
            </NavLink>
          ))}
        </nav>
      </aside>

      {sidebarOpen && (
        <div
          className="fixed inset-0 z-30 bg-background/50 lg:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      <main className="pt-16 lg:pl-64">
        <div className="p-4 lg:p-6">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
