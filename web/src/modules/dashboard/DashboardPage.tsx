import { useQuery } from '@tanstack/react-query';
import {
  LayoutDashboard,
  Users,
  Package,
  ShoppingCart,
  TrendingUp,
  AlertTriangle,
  Clock,
  Plus,
  Wrench,
} from 'lucide-react';
import { Link } from 'react-router-dom';
import api from '@/api/client';
import { PageHeader } from '@/components/shared/page-header';
import { LoadingSpinner } from '@/components/shared/loading';
import { Button } from '@/components/ui/button';

interface DashboardStats {
  open_jobs: number;
  jobs_change: number;
  stock_alerts: number;
  alerts_change: number;
  bays_occupied: number;
  total_bays: number;
  goods_in_transit: number;
  transit_change: number;
}

interface RecentActivity {
  id: string;
  action: string;
  description: string;
  performed_by: string;
  created_at: string;
}

interface Bay {
  id: string;
  name: string;
  status: 'FREE' | 'OCCUPIED' | 'RESERVED' | 'MAINTENANCE';
  job_number: string | null;
}

function useDashboardStats() {
  return useQuery({
    queryKey: ['dashboard', 'stats'],
    queryFn: async () => {
      const response = await api.get<DashboardStats>('/v1/dashboard/stats');
      return response.data;
    },
    staleTime: 30000,
  });
}

function useRecentActivity() {
  return useQuery({
    queryKey: ['dashboard', 'activity'],
    queryFn: async () => {
      const response = await api.get<{ data: RecentActivity[] }>('/v1/jobs/activities/recent');
      return response.data.data;
    },
    staleTime: 30000,
  });
}

function useBays() {
  return useQuery({
    queryKey: ['dashboard', 'bays'],
    queryFn: async () => {
      const response = await api.get<{ data: Bay[] }>('/v1/bays/board');
      return response.data.data;
    },
    staleTime: 30000,
  });
}

function KPICard({
  title,
  value,
  change,
  icon: Icon,
  href,
  alert,
}: {
  title: string;
  value: number | string;
  change?: number;
  icon: React.ElementType;
  href?: string;
  alert?: boolean;
}) {
  const Card = (
    <div
      className={`p-4 rounded-lg border border-border bg-surface ${
        alert ? 'border-l-4 border-l-warning' : ''
      }`}
    >
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm text-muted-foreground">{title}</p>
          <p className="text-3xl font-bold text-foreground mt-1">{value}</p>
          {change !== undefined && (
            <p
              className={`text-sm mt-1 ${
                change >= 0 ? 'text-success' : 'text-destructive'
              }`}
            >
              {change >= 0 ? '+' : ''}
              {change}% from last week
            </p>
          )}
        </div>
        <div className="p-2 rounded-lg bg-surface-raised">
          <Icon className="h-5 w-5 text-accent" />
        </div>
      </div>
      {href && (
        <Link
          to={href}
          className="text-sm text-accent mt-3 inline-flex items-center hover:underline"
        >
          View all <TrendingUp className="h-3 w-3 ml-1" />
        </Link>
      )}
    </div>
  );

  return Card;
}

function BayWidget() {
  const { data: bays, isLoading } = useBays();

  if (isLoading) return <LoadingSpinner />;
  if (!bays) return null;

  const statusColors = {
    FREE: 'bg-success',
    OCCUPIED: 'bg-warning',
    RESERVED: 'bg-muted-foreground',
    MAINTENANCE: 'bg-destructive',
  };

  return (
    <div className="rounded-lg border border-border bg-surface p-4">
      <h3 className="text-lg font-semibold text-foreground mb-3">Service Bays</h3>
      <div className="grid grid-cols-4 sm:grid-cols-6 gap-2">
        {bays.map((bay) => (
          <div
            key={bay.id}
            className={`aspect-square rounded-lg p-2 flex flex-col items-center justify-center ${
              statusColors[bay.status]
            }`}
          >
            <span className="text-xs font-medium text-surface-foreground">
              {bay.name}
            </span>
            {bay.job_number && (
              <span className="text-[10px] text-surface-foreground truncate w-full text-center">
                {bay.job_number}
              </span>
            )}
          </div>
        ))}
      </div>
      <div className="flex gap-3 mt-3 text-xs text-muted-foreground">
        <span className="flex items-center gap-1">
          <span className="w-2 h-2 rounded-full bg-success" /> Free
        </span>
        <span className="flex items-center gap-1">
          <span className="w-2 h-2 rounded-full bg-warning" /> Occupied
        </span>
        <span className="flex items-center gap-1">
          <span className="w-2 h-2 rounded-full bg-muted-foreground" /> Reserved
        </span>
      </div>
    </div>
  );
}

function ActivityFeed() {
  const { data: activities, isLoading } = useRecentActivity();

  if (isLoading) return <LoadingSpinner />;
  if (!activities || activities.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        No recent activity
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {activities.slice(0, 5).map((activity) => (
        <div
          key={activity.id}
          className="flex gap-3 pb-3 border-b border-border last:border-0"
        >
          <div className="p-2 rounded-full bg-surface-raised h-fit">
            <Clock className="h-4 w-4 text-muted-foreground" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm text-foreground">{activity.description}</p>
            <p className="text-xs text-muted-foreground mt-1">
              {activity.performed_by} •{' '}
              {new Date(activity.created_at).toLocaleString()}
            </p>
          </div>
        </div>
      ))}
    </div>
  );
}

export default function DashboardPage() {
  const { data: stats, isLoading } = useDashboardStats();

  if (isLoading) {
    return (
      <div className="p-6">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Dashboard"
        description="Welcome back! Here's what's happening today."
      />

      {/* KPI Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {KPICard({
          title: 'Open Jobs',
          value: stats?.open_jobs ?? 0,
          change: stats?.jobs_change,
          icon: Wrench,
          href: '/jobs',
        })}
        {KPICard({
          title: 'Stock Alerts',
          value: stats?.stock_alerts ?? 0,
          change: stats?.alerts_change,
          icon: AlertTriangle,
          href: '/inventory',
          alert: (stats?.stock_alerts ?? 0) > 0,
        })}
        {KPICard({
          title: 'Bays Occupied',
          value: `${stats?.bays_occupied ?? 0}/${stats?.total_bays ?? 0}`,
          icon: LayoutDashboard,
        })}
        {KPICard({
          title: 'Goods In Transit',
          value: stats?.goods_in_transit ?? 0,
          change: stats?.transit_change,
          icon: ShoppingCart,
          href: '/purchases',
        })}
      </div>

      {/* Quick Actions */}
      <div className="flex flex-wrap gap-2">
        <Button asChild>
          <Link to="/jobs/new">
            <Plus className="h-4 w-4 mr-1" /> New Job
          </Link>
        </Button>
        <Button variant="secondary" asChild>
          <Link to="/customers/new">
            <Users className="h-4 w-4 mr-1" /> Add Customer
          </Link>
        </Button>
        <Button variant="secondary" asChild>
          <Link to="/inventory/new">
            <Package className="h-4 w-4 mr-1" /> Add Inventory
          </Link>
        </Button>
      </div>

      {/* Main Content Grid */}
      <div className="grid lg:grid-cols-2 gap-6">
        {/* Bay Board */}
        <BayWidget />

        {/* Recent Activity */}
        <div className="rounded-lg border border-border bg-surface p-4">
          <h3 className="text-lg font-semibold text-foreground mb-3">
            Recent Activity
          </h3>
          <ActivityFeed />
        </div>
      </div>
    </div>
  );
}