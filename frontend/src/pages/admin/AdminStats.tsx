import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  getPlatformStats,
  getPlatformOverview,
  getPlatformTimeline,
  getOrgBreakdown,
  getEmailStatus,
} from '~/api/admin';
import { StatsCard } from '~/components/StatsCard';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui/card';
import { Tabs, TabsList, TabsTrigger } from '~/components/ui/tabs';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '~/components/ui/table';
import { Skeleton } from '~/components/ui/skeleton';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

const PERIODS = [
  { label: '24h', value: '1d' },
  { label: '7d', value: '7d' },
  { label: '30d', value: '30d' },
  { label: '90d', value: '90d' },
];

export function AdminStats() {
  const [period, setPeriod] = useState('30d');

  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['admin-stats'],
    queryFn: getPlatformStats,
  });

  const { data: overview } = useQuery({
    queryKey: ['admin-overview', period],
    queryFn: () => getPlatformOverview(period),
  });

  const { data: timeline } = useQuery({
    queryKey: ['admin-timeline', period],
    queryFn: () => getPlatformTimeline(period),
  });

  const { data: orgBreakdown } = useQuery({
    queryKey: ['admin-org-breakdown', period],
    queryFn: () => getOrgBreakdown(period),
  });

  const { data: emailStatus } = useQuery({
    queryKey: ['admin-email-status'],
    queryFn: getEmailStatus,
  });

  return (
    <div>
      <div className="flex items-center justify-between mb-2">
        <h1 className="text-2xl font-bold">Platform Stats</h1>
        <Tabs value={period} onValueChange={setPeriod}>
          <TabsList>
            {PERIODS.map((p) => (
              <TabsTrigger key={p.value} value={p.value}>
                {p.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </div>

      {statsLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4 mb-8">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className="h-28" />
          ))}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4 mb-8">
          <StatsCard title="Total Users" value={stats?.total_users ?? 0} />
          <StatsCard title="Total Orgs" value={stats?.total_orgs ?? 0} />
          <StatsCard title="Active Tokens" value={stats?.total_active_tokens ?? 0} />
          <StatsCard title="Active Sessions" value={stats?.total_active_sessions ?? 0} />
          <StatsCard
            title="Hit Rate"
            value={overview ? `${(overview.hit_rate * 100).toFixed(1)}%` : '-'}
          />
          <StatsCard
            title="Bandwidth Saved"
            value={overview ? formatBytes(overview.total_bytes_saved) : '-'}
          />
        </div>
      )}

      <Card className="mb-8">
        <CardHeader>
          <CardTitle className="text-base">Email</CardTitle>
        </CardHeader>
        <CardContent>
          {emailStatus?.configured ? (
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-green-500" />
              <span className="text-sm">
                SMTP configured — <span className="font-mono">{emailStatus.host}</span>
              </span>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-yellow-500" />
              <span className="text-sm text-muted-foreground">
                SMTP not configured — email features disabled
              </span>
            </div>
          )}
        </CardContent>
      </Card>

      <Card className="mb-8">
        <CardHeader>
          <CardTitle className="text-base">Cache Events Over Time</CardTitle>
        </CardHeader>
        <CardContent>
          {timeline && timeline.length > 0 ? (
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={timeline}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis
                  dataKey="date"
                  tickFormatter={(v) =>
                    new Date(v).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
                  }
                  fontSize={12}
                  className="fill-muted-foreground"
                />
                <YAxis fontSize={12} className="fill-muted-foreground" />
                <Tooltip
                  labelFormatter={(v) => new Date(v).toLocaleDateString()}
                  contentStyle={{
                    backgroundColor: 'hsl(var(--card))',
                    border: '1px solid hsl(var(--border))',
                    borderRadius: '0.5rem',
                    color: 'hsl(var(--foreground))',
                  }}
                />
                <Legend />
                <Line
                  type="monotone"
                  dataKey="hits"
                  stroke="hsl(142, 71%, 45%)"
                  strokeWidth={2}
                  dot={false}
                />
                <Line
                  type="monotone"
                  dataKey="misses"
                  stroke="hsl(0, 84%, 60%)"
                  strokeWidth={2}
                  dot={false}
                />
                <Line
                  type="monotone"
                  dataKey="puts"
                  stroke="hsl(217, 91%, 60%)"
                  strokeWidth={2}
                  dot={false}
                />
              </LineChart>
            </ResponsiveContainer>
          ) : (
            <p className="text-muted-foreground text-center py-12">No data yet</p>
          )}
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        <Card>
          <CardContent className="pt-6">
            <p className="text-lg font-semibold text-green-600 dark:text-green-400">
              {overview?.hits ?? 0}
            </p>
            <p className="text-sm text-muted-foreground">Cache Hits</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <p className="text-lg font-semibold text-red-600 dark:text-red-400">
              {overview?.misses ?? 0}
            </p>
            <p className="text-sm text-muted-foreground">Cache Misses</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <p className="text-lg font-semibold text-blue-600 dark:text-blue-400">
              {overview?.puts ?? 0}
            </p>
            <p className="text-sm text-muted-foreground">Cache Puts</p>
          </CardContent>
        </Card>
      </div>

      {orgBreakdown && orgBreakdown.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Org Breakdown</CardTitle>
          </CardHeader>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Organization</TableHead>
                <TableHead>Slug</TableHead>
                <TableHead className="text-right">Hits</TableHead>
                <TableHead className="text-right">Misses</TableHead>
                <TableHead className="text-right">Puts</TableHead>
                <TableHead className="text-right">Bandwidth</TableHead>
                <TableHead className="text-right">Hit Rate</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {orgBreakdown.map((org) => {
                const total = org.hits + org.misses + org.puts;
                const hitRate = total > 0 ? ((org.hits / total) * 100).toFixed(1) : '0.0';
                return (
                  <TableRow key={org.org_id}>
                    <TableCell className="font-medium">{org.org_name}</TableCell>
                    <TableCell className="font-mono text-muted-foreground">{org.org_slug}</TableCell>
                    <TableCell className="text-right text-green-600 dark:text-green-400">
                      {org.hits}
                    </TableCell>
                    <TableCell className="text-right text-red-600 dark:text-red-400">
                      {org.misses}
                    </TableCell>
                    <TableCell className="text-right text-blue-600 dark:text-blue-400">
                      {org.puts}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      {formatBytes(org.total_bytes)}
                    </TableCell>
                    <TableCell className="text-right">{hitRate}%</TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </Card>
      )}
    </div>
  );
}
