import { useState } from 'react';
import { Link } from 'react-router-dom';
import { login } from '~/api/auth';
import { Button } from '~/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '~/components/ui/card';
import { Input } from '~/components/ui/input';
import { Label } from '~/components/ui/label';
import { Alert, AlertDescription } from '~/components/ui/alert';
import { TwofaVerify } from '~/components/TwofaVerify';

interface Props {
  onLogin: () => void;
}

export function Login({ onLogin }: Props) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [twofaPending, setTwofaPending] = useState<{
    token: string;
    method: string;
  } | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const res = await login(email, password);
      if ('twofa_required' in res && res.twofa_required) {
        setTwofaPending({
          token: res.pending_token,
          method: res.twofa_method,
        });
      } else {
        await onLogin();
      }
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleTwofaSuccess = async () => {
    await onLogin();
  };

  const handleTwofaBack = () => {
    setTwofaPending(null);
    setPassword('');
  };

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="w-full max-w-md">
        <h1 className="text-2xl font-bold text-center mb-8">Turbo Remote Cache</h1>
        <Card>
          {twofaPending ? (
            <CardContent className="pt-6">
              <TwofaVerify
                pendingToken={twofaPending.token}
                method={twofaPending.method}
                onSuccess={handleTwofaSuccess}
                onBack={handleTwofaBack}
              />
            </CardContent>
          ) : (
            <>
              <CardHeader>
                <CardTitle>Sign in</CardTitle>
                <CardDescription>Enter your credentials to access your account</CardDescription>
              </CardHeader>
              <CardContent>
                <form onSubmit={handleSubmit} className="space-y-4">
                  {error && (
                    <Alert variant="destructive">
                      <AlertDescription>{error}</AlertDescription>
                    </Alert>
                  )}
                  <div className="space-y-2">
                    <Label htmlFor="email">Email</Label>
                    <Input
                      id="email"
                      type="email"
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      required
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="password">Password</Label>
                    <Input
                      id="password"
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      required
                    />
                  </div>
                  <div className="text-right">
                    <Link
                      to="/forgot-password"
                      className="text-sm text-muted-foreground hover:text-primary hover:underline"
                    >
                      Forgot password?
                    </Link>
                  </div>
                  <Button type="submit" className="w-full" disabled={loading}>
                    {loading ? 'Signing in...' : 'Sign in'}
                  </Button>
                  <p className="text-sm text-center text-muted-foreground">
                    Don't have an account?{' '}
                    <Link to="/register" className="text-primary hover:underline">
                      Sign up
                    </Link>
                  </p>
                  {import.meta.env.DEV && (
                    <Button
                      type="button"
                      variant="outline"
                      className="w-full"
                      disabled={loading}
                      onClick={async () => {
                        setError('');
                        setLoading(true);
                        try {
                          const res = await login(import.meta.env.SUPER_ADMIN_EMAIL, import.meta.env.SUPER_ADMIN_PASSWORD);
                          if ('twofa_required' in res && res.twofa_required) {
                            setTwofaPending({ token: res.pending_token, method: res.twofa_method });
                          } else {
                            await onLogin();
                          }
                        } catch (err: any) {
                          setError(err.message);
                        } finally {
                          setLoading(false);
                        }
                      }}
                    >
                      Dev: Login as Super Admin
                    </Button>
                  )}
                </form>
              </CardContent>
            </>
          )}
        </Card>
      </div>
    </div>
  );
}
