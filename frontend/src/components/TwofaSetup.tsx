import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  confirmTotp,
  disableTwofa,
  enableEmailTwofa,
  getTwofaStatus,
  regenerateRecoveryCodes,
  setupTotp,
} from '~/api/auth';
import { toast } from 'sonner';
import type { TotpSetupResponse, User } from '~/types';
import { Button } from '~/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '~/components/ui/card';
import { Input } from '~/components/ui/input';
import { Label } from '~/components/ui/label';
import { Badge } from '~/components/ui/badge';
import { RecoveryCodesDisplay } from './RecoveryCodesDisplay';

interface Props {
  user: User;
  onUpdate: () => void;
}

type Step = 'idle' | 'totp-setup' | 'recovery-codes';

export function TwofaSetup({ user, onUpdate }: Props) {
  const queryClient = useQueryClient();
  const [step, setStep] = useState<Step>('idle');
  const [totpData, setTotpData] = useState<TotpSetupResponse | null>(null);
  const [totpCode, setTotpCode] = useState('');
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);
  const [password, setPassword] = useState('');
  const [showDisable, setShowDisable] = useState(false);
  const [showRegenerate, setShowRegenerate] = useState(false);

  const { data: status } = useQuery({
    queryKey: ['twofa-status'],
    queryFn: getTwofaStatus,
  });

  const setupTotpMutation = useMutation({
    mutationFn: setupTotp,
    onSuccess: (data) => {
      setTotpData(data);
      setStep('totp-setup');
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const confirmTotpMutation = useMutation({
    mutationFn: (code: string) => confirmTotp(code),
    onSuccess: (data) => {
      setRecoveryCodes(data.recovery_codes);
      setStep('recovery-codes');
      setTotpCode('');
      queryClient.invalidateQueries({ queryKey: ['twofa-status'] });
      onUpdate();
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const enableEmailMutation = useMutation({
    mutationFn: enableEmailTwofa,
    onSuccess: (data) => {
      setRecoveryCodes(data.recovery_codes);
      setStep('recovery-codes');
      queryClient.invalidateQueries({ queryKey: ['twofa-status'] });
      onUpdate();
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const disableMutation = useMutation({
    mutationFn: (pw: string) => disableTwofa(pw),
    onSuccess: () => {
      toast.success('Two-factor authentication disabled');
      setShowDisable(false);
      setPassword('');
      queryClient.invalidateQueries({ queryKey: ['twofa-status'] });
      onUpdate();
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const regenerateMutation = useMutation({
    mutationFn: (pw: string) => regenerateRecoveryCodes(pw),
    onSuccess: (data) => {
      setRecoveryCodes(data.recovery_codes);
      setStep('recovery-codes');
      setShowRegenerate(false);
      setPassword('');
      queryClient.invalidateQueries({ queryKey: ['twofa-status'] });
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const handleDone = () => {
    setStep('idle');
    setTotpData(null);
    setRecoveryCodes([]);
  };

  const isEnabled = status?.enabled ?? user.twofa_enabled;
  const method = status?.method ?? user.twofa_method;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Two-Factor Authentication</CardTitle>
            <CardDescription>Add an extra layer of security to your account</CardDescription>
          </div>
          {isEnabled && (
            <Badge variant="secondary">
              {method === 'totp' ? 'Authenticator App' : 'Email OTP'}
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {step === 'recovery-codes' && (
          <RecoveryCodesDisplay codes={recoveryCodes} onDone={handleDone} />
        )}

        {step === 'totp-setup' && totpData && (
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Scan this QR code with your authenticator app (1Password, Google Authenticator,
              Authy), then enter the code below.
            </p>
            <div className="flex justify-center">
              <img
                src={`data:image/png;base64,${totpData.qr_code}`}
                alt="TOTP QR Code"
                className="w-48 h-48"
              />
            </div>
            <details className="text-sm">
              <summary className="cursor-pointer text-muted-foreground hover:text-primary">
                Can't scan? Enter manually
              </summary>
              <code className="block mt-2 p-2 bg-muted rounded text-xs break-all">
                {totpData.secret}
              </code>
            </details>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                confirmTotpMutation.mutate(totpCode);
              }}
              className="space-y-3"
            >
              <div className="space-y-2">
                <Label htmlFor="totp-code">Verification Code</Label>
                <Input
                  id="totp-code"
                  value={totpCode}
                  onChange={(e) => setTotpCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  placeholder="000000"
                  className="font-mono text-center text-lg"
                  maxLength={6}
                  autoFocus
                />
              </div>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  onClick={() => {
                    setStep('idle');
                    setTotpData(null);
                  }}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={totpCode.length !== 6 || confirmTotpMutation.isPending}
                >
                  {confirmTotpMutation.isPending ? 'Verifying...' : 'Verify & Enable'}
                </Button>
              </div>
            </form>
          </div>
        )}

        {step === 'idle' && !isEnabled && (
          <div className="space-y-3">
            <p className="text-sm text-muted-foreground">
              Protect your account by requiring a second factor when you sign in.
            </p>
            <div className="flex gap-2">
              <Button
                onClick={() => setupTotpMutation.mutate()}
                disabled={setupTotpMutation.isPending}
              >
                {setupTotpMutation.isPending ? 'Setting up...' : 'Authenticator App'}
              </Button>
              <Button
                variant="outline"
                onClick={() => enableEmailMutation.mutate()}
                disabled={enableEmailMutation.isPending}
              >
                {enableEmailMutation.isPending ? 'Enabling...' : 'Email OTP'}
              </Button>
            </div>
          </div>
        )}

        {step === 'idle' && isEnabled && (
          <div className="space-y-4">
            <div className="text-sm text-muted-foreground">
              {status && (
                <p>
                  You have{' '}
                  <span className="font-semibold text-foreground">
                    {status.recovery_codes_remaining}
                  </span>{' '}
                  recovery codes remaining.
                </p>
              )}
            </div>

            {showRegenerate ? (
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  regenerateMutation.mutate(password);
                }}
                className="space-y-3"
              >
                <div className="space-y-2">
                  <Label htmlFor="regen-password">Confirm Password</Label>
                  <Input
                    id="regen-password"
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    autoFocus
                  />
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    onClick={() => {
                      setShowRegenerate(false);
                      setPassword('');
                    }}
                  >
                    Cancel
                  </Button>
                  <Button type="submit" disabled={!password || regenerateMutation.isPending}>
                    {regenerateMutation.isPending ? 'Regenerating...' : 'Regenerate Codes'}
                  </Button>
                </div>
              </form>
            ) : showDisable ? (
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  disableMutation.mutate(password);
                }}
                className="space-y-3"
              >
                <div className="space-y-2">
                  <Label htmlFor="disable-password">Confirm Password</Label>
                  <Input
                    id="disable-password"
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    autoFocus
                  />
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    onClick={() => {
                      setShowDisable(false);
                      setPassword('');
                    }}
                  >
                    Cancel
                  </Button>
                  <Button
                    type="submit"
                    variant="destructive"
                    disabled={!password || disableMutation.isPending}
                  >
                    {disableMutation.isPending ? 'Disabling...' : 'Disable 2FA'}
                  </Button>
                </div>
              </form>
            ) : (
              <div className="flex gap-2">
                <Button variant="outline" onClick={() => setShowRegenerate(true)}>
                  Regenerate Recovery Codes
                </Button>
                <Button variant="destructive" onClick={() => setShowDisable(true)}>
                  Disable 2FA
                </Button>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
