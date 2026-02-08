import { Navigate } from 'react-router-dom';
import { Skeleton } from '~/components/ui/skeleton';
import type { User } from '~/types';

interface Props {
  user: User | null;
  loading: boolean;
  children: React.ReactNode;
}

export function ProtectedRoute({ user, loading, children }: Props) {
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="space-y-4 w-full max-w-md">
          <Skeleton className="h-8 w-48 mx-auto" />
          <Skeleton className="h-4 w-64 mx-auto" />
          <Skeleton className="h-4 w-56 mx-auto" />
        </div>
      </div>
    );
  }

  if (!user) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}
