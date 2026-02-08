import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { updateProfile } from '~/api/auth';
import { toast } from 'sonner';
import type { User } from '~/types';
import { Button } from '~/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '~/components/ui/card';
import { Input } from '~/components/ui/input';
import { Label } from '~/components/ui/label';
import { TwofaSetup } from '~/components/TwofaSetup';

interface Props {
  user: User;
  onUpdate: () => void;
}

export function UserProfile({ user, onUpdate }: Props) {
  const [name, setName] = useState(user.name);
  const [password, setPassword] = useState('');

  const mutation = useMutation({
    mutationFn: () =>
      updateProfile({
        name: name !== user.name ? name : undefined,
        password: password || undefined,
      }),
    onSuccess: () => {
      toast.success('Profile updated');
      setPassword('');
      onUpdate();
    },
    onError: (err: Error) => {
      toast.error(err.message);
    },
  });

  return (
    <div className="max-w-lg space-y-6">
      <h1 className="text-2xl font-bold">Profile</h1>

      <Card>
        <CardHeader>
          <CardTitle>Account Details</CardTitle>
          <CardDescription>Manage your profile information</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>Email</Label>
            <Input value={user.email} disabled />
          </div>

          <div className="space-y-2">
            <Label>Role</Label>
            <Input value={user.role} disabled />
          </div>

          <form
            onSubmit={(e) => {
              e.preventDefault();
              mutation.mutate();
            }}
            className="space-y-4"
          >
            <div className="space-y-2">
              <Label htmlFor="name">Name</Label>
              <Input id="name" value={name} onChange={(e) => setName(e.target.value)} />
            </div>

            <div className="space-y-2">
              <Label htmlFor="new-password">New Password</Label>
              <Input
                id="new-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Leave blank to keep current"
                minLength={8}
              />
            </div>

            <Button type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? 'Saving...' : 'Save Changes'}
            </Button>
          </form>
        </CardContent>
      </Card>

      <TwofaSetup user={user} onUpdate={onUpdate} />
    </div>
  );
}
