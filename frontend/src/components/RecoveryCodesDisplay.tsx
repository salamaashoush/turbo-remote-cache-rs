import { Button } from '~/components/ui/button';
import { toast } from 'sonner';

interface Props {
  codes: string[];
  onDone: () => void;
}

export function RecoveryCodesDisplay({ codes, onDone }: Props) {
  const copyAll = () => {
    navigator.clipboard.writeText(codes.join('\n'));
    toast.success('Recovery codes copied to clipboard');
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="font-semibold text-lg">Recovery Codes</h3>
        <p className="text-sm text-muted-foreground">
          Save these codes in a safe place. Each code can only be used once to sign in if you lose
          access to your authenticator.
        </p>
      </div>

      <div className="grid grid-cols-2 gap-2 rounded-lg border bg-muted/50 p-4">
        {codes.map((code) => (
          <code key={code} className="text-sm font-mono text-center py-1">
            {code}
          </code>
        ))}
      </div>

      <div className="flex gap-2">
        <Button variant="outline" onClick={copyAll}>
          Copy all
        </Button>
        <Button onClick={onDone}>I've saved these codes</Button>
      </div>
    </div>
  );
}
