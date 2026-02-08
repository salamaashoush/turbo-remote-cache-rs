import { useEffect, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { verifyEmail } from '~/api/auth';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui/card';
import { Alert, AlertDescription } from '~/components/ui/alert';

export function VerifyEmail() {
  const [searchParams] = useSearchParams();
  const token = searchParams.get('token') ?? '';
  const hasToken = token.length > 0;

  const [status, setStatus] = useState<'loading' | 'success' | 'error'>(
    hasToken ? 'loading' : 'error',
  );
  const [error, setError] = useState(hasToken ? '' : 'Invalid verification link');

  useEffect(() => {
    if (!hasToken) return;

    let cancelled = false;
    verifyEmail(token)
      .then(() => {
        if (!cancelled) setStatus('success');
      })
      .catch((err: Error) => {
        if (!cancelled) {
          setStatus('error');
          setError(err.message || 'Verification failed');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [token, hasToken]);

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="w-full max-w-md">
        <h1 className="text-2xl font-bold text-center mb-8">Turbo Remote Cache</h1>
        <Card>
          <CardHeader>
            <CardTitle>Email Verification</CardTitle>
          </CardHeader>
          <CardContent>
            {status === 'loading' && (
              <p className="text-center text-muted-foreground">Verifying your email...</p>
            )}
            {status === 'success' && (
              <div className="space-y-4">
                <Alert>
                  <AlertDescription>Your email has been verified successfully.</AlertDescription>
                </Alert>
                <p className="text-sm text-center">
                  <Link to="/" className="text-primary hover:underline">
                    Go to dashboard
                  </Link>
                </p>
              </div>
            )}
            {status === 'error' && (
              <div className="space-y-4">
                <Alert variant="destructive">
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
                <p className="text-sm text-center">
                  <Link to="/" className="text-primary hover:underline">
                    Go to dashboard
                  </Link>
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
