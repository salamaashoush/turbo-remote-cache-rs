import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getOrg, listMembers } from '~/api/orgs';
import { listTeams } from '~/api/teams';
import { getOverview } from '~/api/analytics';
import { StatsCard } from '~/components/StatsCard';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui/card';
import { Skeleton } from '~/components/ui/skeleton';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function OrgOverview() {
  const { orgId } = useParams<{ orgId: string }>();
  const { data: org } = useQuery({ queryKey: ['org', orgId], queryFn: () => getOrg(orgId!) });
  const { data: members } = useQuery({
    queryKey: ['members', orgId],
    queryFn: () => listMembers(orgId!),
  });
  const { data: teams } = useQuery({
    queryKey: ['teams', orgId],
    queryFn: () => listTeams(orgId!),
  });
  const { data: overview, isLoading } = useQuery({
    queryKey: ['overview', orgId],
    queryFn: () => getOverview(orgId!),
  });

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">{org?.name ?? 'Organization'}</h1>

      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-28" />
          ))}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <StatsCard title="Members" value={members?.length ?? 0} />
          <StatsCard title="Teams" value={teams?.length ?? 0} />
          <StatsCard
            title="Hit Rate (30d)"
            value={overview ? `${(overview.hit_rate * 100).toFixed(1)}%` : '-'}
          />
          <StatsCard
            title="Bandwidth Saved"
            value={overview ? formatBytes(overview.total_bytes_saved) : '-'}
          />
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Cache Events (30d)</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Hits</span>
              <span className="font-medium">{overview?.hits ?? 0}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Misses</span>
              <span className="font-medium">{overview?.misses ?? 0}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Puts</span>
              <span className="font-medium">{overview?.puts ?? 0}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total</span>
              <span className="font-medium">{overview?.total_events ?? 0}</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Organization Details</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Slug</span>
              <span className="font-mono">{org?.slug}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Created</span>
              <span>{org?.created_at ? new Date(org.created_at).toLocaleDateString() : '-'}</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
