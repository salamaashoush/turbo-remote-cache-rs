import { apiFetch } from './client';
import type { Team, TeamMember } from '../types';

export async function listTeams(orgId: string): Promise<Team[]> {
  return apiFetch<Team[]>(`/orgs/${orgId}/teams`);
}

export async function createTeam(orgId: string, name: string, slug: string): Promise<Team> {
  return apiFetch<Team>(`/orgs/${orgId}/teams`, {
    method: 'POST',
    body: JSON.stringify({ name, slug }),
  });
}

export async function deleteTeam(orgId: string, teamId: string): Promise<void> {
  await apiFetch(`/orgs/${orgId}/teams/${teamId}`, { method: 'DELETE' });
}

export async function listTeamMembers(orgId: string, teamId: string): Promise<TeamMember[]> {
  return apiFetch<TeamMember[]>(`/orgs/${orgId}/teams/${teamId}/members`);
}

export async function addTeamMember(
  orgId: string,
  teamId: string,
  userId: string,
  role?: string,
): Promise<TeamMember> {
  return apiFetch<TeamMember>(`/orgs/${orgId}/teams/${teamId}/members`, {
    method: 'POST',
    body: JSON.stringify({ user_id: userId, role }),
  });
}

export async function removeTeamMember(
  orgId: string,
  teamId: string,
  userId: string,
): Promise<void> {
  await apiFetch(`/orgs/${orgId}/teams/${teamId}/members/${userId}`, { method: 'DELETE' });
}
