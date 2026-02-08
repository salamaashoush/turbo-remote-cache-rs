import { apiFetch } from './client';
import type {
  AdminOrg,
  AdminOrgDetail,
  AdminUser,
  AdminUserDetail,
  AnalyticsOverview,
  Organization,
  OrgAnalytics,
  PlatformStats,
  TimelinePoint,
  UpdateOrgLimitsRequest,
  User,
} from '../types';

export async function listAllOrgs(limit = 50, offset = 0): Promise<AdminOrg[]> {
  return apiFetch<AdminOrg[]>(`/admin/orgs?limit=${limit}&offset=${offset}`);
}

export async function listAllUsers(limit = 50, offset = 0): Promise<AdminUser[]> {
  return apiFetch<AdminUser[]>(`/admin/users?limit=${limit}&offset=${offset}`);
}

export async function getPlatformStats(): Promise<PlatformStats> {
  return apiFetch<PlatformStats>('/admin/stats');
}

export async function updateUserRole(userId: string, role: string): Promise<User> {
  return apiFetch<User>(`/admin/users/${userId}`, {
    method: 'PATCH',
    body: JSON.stringify({ role }),
  });
}

export async function deleteOrg(orgId: string): Promise<void> {
  await apiFetch(`/admin/orgs/${orgId}`, { method: 'DELETE' });
}

export async function getPlatformOverview(period: string): Promise<AnalyticsOverview> {
  return apiFetch<AnalyticsOverview>(`/admin/analytics/overview?period=${period}`);
}

export async function getPlatformTimeline(
  period: string,
  granularity = 'day',
): Promise<TimelinePoint[]> {
  return apiFetch<TimelinePoint[]>(
    `/admin/analytics/timeline?period=${period}&granularity=${granularity}`,
  );
}

export async function getOrgBreakdown(period: string): Promise<OrgAnalytics[]> {
  return apiFetch<OrgAnalytics[]>(`/admin/analytics/orgs?period=${period}`);
}

export interface EmailStatus {
  configured: boolean;
  host: string | null;
}

export async function getEmailStatus(): Promise<EmailStatus> {
  return apiFetch<EmailStatus>('/admin/email/status');
}

// ─── New admin API functions ───

export async function getUser(userId: string): Promise<AdminUserDetail> {
  return apiFetch<AdminUserDetail>(`/admin/users/${userId}`);
}

export async function activateUser(userId: string): Promise<void> {
  await apiFetch(`/admin/users/${userId}/activate`, { method: 'POST' });
}

export async function deactivateUser(userId: string): Promise<void> {
  await apiFetch(`/admin/users/${userId}/deactivate`, { method: 'POST' });
}

export async function deleteUser(userId: string): Promise<void> {
  await apiFetch(`/admin/users/${userId}`, { method: 'DELETE' });
}

export async function forceLogout(userId: string): Promise<void> {
  await apiFetch(`/admin/users/${userId}/force-logout`, { method: 'POST' });
}

export async function adminPasswordReset(userId: string): Promise<void> {
  await apiFetch(`/admin/users/${userId}/reset-password`, { method: 'POST' });
}

export async function addUserToOrg(
  userId: string,
  orgId: string,
  role = 'member',
): Promise<void> {
  await apiFetch(`/admin/users/${userId}/orgs`, {
    method: 'POST',
    body: JSON.stringify({ org_id: orgId, role }),
  });
}

export async function removeUserFromOrg(userId: string, orgId: string): Promise<void> {
  await apiFetch(`/admin/users/${userId}/orgs/${orgId}`, { method: 'DELETE' });
}

export async function getOrg(orgId: string): Promise<AdminOrgDetail> {
  return apiFetch<AdminOrgDetail>(`/admin/orgs/${orgId}`);
}

export async function createOrg(
  name: string,
  slug: string,
  ownerEmail: string,
): Promise<Organization> {
  return apiFetch<Organization>('/admin/orgs', {
    method: 'POST',
    body: JSON.stringify({ name, slug, owner_email: ownerEmail }),
  });
}

export async function updateOrgLimits(
  orgId: string,
  limits: UpdateOrgLimitsRequest,
): Promise<void> {
  await apiFetch(`/admin/orgs/${orgId}/limits`, {
    method: 'PATCH',
    body: JSON.stringify(limits),
  });
}

export async function transferOrgOwnership(orgId: string, newOwnerId: string): Promise<void> {
  await apiFetch(`/admin/orgs/${orgId}/transfer`, {
    method: 'POST',
    body: JSON.stringify({ new_owner_id: newOwnerId }),
  });
}

export async function purgeOrgCache(orgId: string): Promise<{ count: number }> {
  return apiFetch<{ count: number }>(`/admin/orgs/${orgId}/purge-cache`, { method: 'POST' });
}
