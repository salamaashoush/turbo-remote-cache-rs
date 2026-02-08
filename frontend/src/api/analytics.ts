import { apiFetch } from './client';
import type { AnalyticsOverview, TimelinePoint, TeamAnalytics, ArtifactEntry } from '../types';

export async function getOverview(orgId: string, period = '30d'): Promise<AnalyticsOverview> {
  return apiFetch<AnalyticsOverview>(`/orgs/${orgId}/analytics/overview?period=${period}`);
}

export async function getTimeline(
  orgId: string,
  period = '30d',
  granularity = 'day',
): Promise<TimelinePoint[]> {
  return apiFetch<TimelinePoint[]>(
    `/orgs/${orgId}/analytics/timeline?period=${period}&granularity=${granularity}`,
  );
}

export async function getTeamBreakdown(orgId: string, period = '30d'): Promise<TeamAnalytics[]> {
  return apiFetch<TeamAnalytics[]>(`/orgs/${orgId}/analytics/teams?period=${period}`);
}

export async function listArtifacts(
  orgId: string,
  limit = 50,
  offset = 0,
): Promise<ArtifactEntry[]> {
  return apiFetch<ArtifactEntry[]>(
    `/orgs/${orgId}/cache/artifacts?limit=${limit}&offset=${offset}`,
  );
}

export async function purgeArtifact(orgId: string, hash: string): Promise<void> {
  await apiFetch(`/orgs/${orgId}/cache/artifacts/${hash}`, { method: 'DELETE' });
}

export async function purgeAll(orgId: string, teamId?: string): Promise<void> {
  const params = teamId ? `?team_id=${teamId}` : '';
  await apiFetch(`/orgs/${orgId}/cache/purge${params}`, { method: 'POST' });
}
