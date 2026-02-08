import { useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { listOrgs } from '~/api/orgs';
import { Skeleton } from '~/components/ui/skeleton';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '~/components/ui/card';
import type { User } from '~/types';

interface Props {
  user: User;
}

export function Dashboard({ user }: Props) {
  const navigate = useNavigate();
  const { data: orgs, isLoading } = useQuery({ queryKey: ['orgs'], queryFn: listOrgs });

  useEffect(() => {
    if (isLoading) return;
    if (orgs && orgs.length > 0) {
      navigate(`/orgs/${orgs[0].id}`, { replace: true });
    } else if (user.role === 'super_admin') {
      navigate('/admin/stats', { replace: true });
    }
  }, [isLoading, orgs, navigate, user.role]);

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <Skeleton className="h-8 w-40" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Skeleton className="h-28" />
          <Skeleton className="h-28" />
          <Skeleton className="h-28" />
        </div>
      </div>
    );
  }

  if (!orgs || orgs.length === 0) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <Card className="max-w-md">
          <CardHeader>
            <CardTitle>No Organizations</CardTitle>
            <CardDescription>
              You are not a member of any organization yet. Ask an admin to add you to one.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return null;
}
