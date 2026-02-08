import { apiFetch } from './client';
import type { Organization, OrgMember } from '../types';

export async function listOrgs(): Promise<Organization[]> {
  return apiFetch<Organization[]>('/orgs');
}

export async function getOrg(orgId: string): Promise<Organization> {
  return apiFetch<Organization>(`/orgs/${orgId}`);
}

export async function updateOrg(
  orgId: string,
  data: { name?: string; slug?: string },
): Promise<Organization> {
  return apiFetch<Organization>(`/orgs/${orgId}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function deleteOrg(orgId: string): Promise<void> {
  await apiFetch(`/orgs/${orgId}`, { method: 'DELETE' });
}

export async function listMembers(orgId: string): Promise<OrgMember[]> {
  return apiFetch<OrgMember[]>(`/orgs/${orgId}/members`);
}

export async function addMember(orgId: string, email: string, role?: string): Promise<OrgMember> {
  return apiFetch<OrgMember>(`/orgs/${orgId}/members`, {
    method: 'POST',
    body: JSON.stringify({ email, role }),
  });
}

export async function removeMember(orgId: string, userId: string): Promise<void> {
  await apiFetch(`/orgs/${orgId}/members/${userId}`, { method: 'DELETE' });
}
