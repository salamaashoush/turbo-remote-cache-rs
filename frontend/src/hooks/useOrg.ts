import { useQuery } from '@tanstack/react-query';
import { listOrgs, getOrg } from '../api/orgs';

export function useOrgs() {
  return useQuery({ queryKey: ['orgs'], queryFn: listOrgs });
}

export function useOrg(orgId: string) {
  return useQuery({
    queryKey: ['org', orgId],
    queryFn: () => getOrg(orgId),
    enabled: !!orgId,
  });
}
