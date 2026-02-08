import { useRef, useState } from 'react';
import { verifyTwofaLogin } from '~/api/auth';
import { Button } from '~/components/ui/button';
import { Input } from '~/components/ui/input';
import { Label } from '~/components/ui/label';
import { Alert, AlertDescription } from '~/components/ui/alert';

interface Props {
  pendingToken: string;
  method: string;
  onSuccess: () => void;
  onBack: () => void;
}

export function TwofaVerify({ pendingToken, method, onSuccess, onBack }: Props) {
  const [digits, setDigits] = useState(['', '', '', '', '', '']);
  const [recoveryMode, setRecoveryMode] = useState(false);
  const [recoveryCode, setRecoveryCode] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const inputRefs = useRef<(HTMLInputElement | null)[]>([]);

  const handleDigitChange = (index: number, value: string) => {
    if (!/^\d*$/.test(value)) return;
    const newDigits = [...digits];
    newDigits[index] = value.slice(-1);
    setDigits(newDigits);

    // Auto-advance to next input
    if (value && index < 5) {
      inputRefs.current[index + 1]?.focus();
    }

    // Auto-submit when all digits filled
    const code = newDigits.join('');
    if (code.length === 6 && newDigits.every((d) => d)) {
      submitCode(code);
    }
  };

  const handleKeyDown = (index: number, e: React.KeyboardEvent) => {
    if (e.key === 'Backspace' && !digits[index] && index > 0) {
      inputRefs.current[index - 1]?.focus();
    }
  };

  const handlePaste = (e: React.ClipboardEvent) => {
    e.preventDefault();
    const text = e.clipboardData.getData('text').replace(/\D/g, '').slice(0, 6);
    if (text.length === 6) {
      const newDigits = text.split('');
      setDigits(newDigits);
      inputRefs.current[5]?.focus();
      submitCode(text);
    }
  };

  const submitCode = async (code: string) => {
    setError('');
    setLoading(true);
    try {
      await verifyTwofaLogin(pendingToken, code);
      onSuccess();
    } catch (err: any) {
      setError(err.message);
      setDigits(['', '', '', '', '', '']);
      inputRefs.current[0]?.focus();
    } finally {
      setLoading(false);
    }
  };

  const handleRecoverySubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!recoveryCode.trim()) return;
    setError('');
    setLoading(true);
    try {
      await verifyTwofaLogin(pendingToken, recoveryCode.trim());
      onSuccess();
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-lg font-semibold">Two-factor authentication</h3>
        <p className="text-sm text-muted-foreground">
          {method === 'totp'
            ? 'Enter the code from your authenticator app.'
            : 'Enter the code sent to your email.'}
        </p>
      </div>

      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {!recoveryMode ? (
        <>
          <div className="flex gap-2 justify-center" onPaste={handlePaste}>
            {digits.map((digit, i) => (
              <Input
                key={i}
                ref={(el) => { inputRefs.current[i] = el; }}
                type="text"
                inputMode="numeric"
                maxLength={1}
                value={digit}
                onChange={(e) => handleDigitChange(i, e.target.value)}
                onKeyDown={(e) => handleKeyDown(i, e)}
                className="w-12 h-12 text-center text-lg font-mono"
                disabled={loading}
                autoFocus={i === 0}
              />
            ))}
          </div>

          <div className="text-center">
            <button
              type="button"
              className="text-sm text-muted-foreground hover:text-primary hover:underline"
              onClick={() => setRecoveryMode(true)}
            >
              Use a recovery code
            </button>
          </div>
        </>
      ) : (
        <form onSubmit={handleRecoverySubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="recovery-code">Recovery Code</Label>
            <Input
              id="recovery-code"
              value={recoveryCode}
              onChange={(e) => setRecoveryCode(e.target.value)}
              placeholder="xxxx-xxxx"
              className="font-mono"
              disabled={loading}
              autoFocus
            />
          </div>
          <Button type="submit" className="w-full" disabled={loading || !recoveryCode.trim()}>
            {loading ? 'Verifying...' : 'Verify'}
          </Button>
          <div className="text-center">
            <button
              type="button"
              className="text-sm text-muted-foreground hover:text-primary hover:underline"
              onClick={() => {
                setRecoveryMode(false);
                setError('');
              }}
            >
              Use {method === 'totp' ? 'authenticator' : 'email'} code instead
            </button>
          </div>
        </form>
      )}

      <div className="pt-2">
        <Button variant="ghost" className="w-full" onClick={onBack} disabled={loading}>
          Back to login
        </Button>
      </div>
    </div>
  );
}
