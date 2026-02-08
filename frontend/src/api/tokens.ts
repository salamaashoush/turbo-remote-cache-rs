import { apiFetch } from './client';
import type { ApiToken, CreateTokenResponse } from '../types';

export async function listTokens(orgId: string): Promise<ApiToken[]> {
  return apiFetch<ApiToken[]>(`/orgs/${orgId}/tokens`);
}

export async function createToken(
  orgId: string,
  name: string,
  teamId?: string,
  expiresInDays?: number,
): Promise<CreateTokenResponse> {
  return apiFetch<CreateTokenResponse>(`/orgs/${orgId}/tokens`, {
    method: 'POST',
    body: JSON.stringify({
      name,
      team_id: teamId || null,
      expires_in_days: expiresInDays || null,
    }),
  });
}

export async function revokeToken(orgId: string, tokenId: string): Promise<void> {
  await apiFetch(`/orgs/${orgId}/tokens/${tokenId}`, { method: 'DELETE' });
}
