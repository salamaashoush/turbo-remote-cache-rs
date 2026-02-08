import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getOrg, updateOrg, listMembers, addMember, removeMember } from '~/api/orgs';
import { toast } from 'sonner';
import { Trash2, Plus } from 'lucide-react';
import { Button } from '~/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '~/components/ui/card';
import { Input } from '~/components/ui/input';
import { Badge } from '~/components/ui/badge';

export function OrgSettings() {
  const { orgId } = useParams<{ orgId: string }>();
  const queryClient = useQueryClient();
  const { data: org } = useQuery({ queryKey: ['org', orgId], queryFn: () => getOrg(orgId!) });
  const { data: members } = useQuery({
    queryKey: ['members', orgId],
    queryFn: () => listMembers(orgId!),
  });

  const [name, setName] = useState('');
  const [memberEmail, setMemberEmail] = useState('');

  const updateMutation = useMutation({
    mutationFn: () => updateOrg(orgId!, { name: name || undefined }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['org', orgId] });
      setName('');
      toast.success('Organization updated');
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const addMemberMutation = useMutation({
    mutationFn: () => addMember(orgId!, memberEmail),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['members', orgId] });
      setMemberEmail('');
      toast.success('Member added');
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const removeMemberMutation = useMutation({
    mutationFn: (userId: string) => removeMember(orgId!, userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['members', orgId] });
      toast.success('Member removed');
    },
  });

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Settings</h1>

      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>Organization Name</CardTitle>
            <CardDescription>Update your organization's display name</CardDescription>
          </CardHeader>
          <CardContent>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                updateMutation.mutate();
              }}
              className="flex gap-3"
            >
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={org?.name}
                className="flex-1"
              />
              <Button type="submit" disabled={updateMutation.isPending}>
                Update
              </Button>
            </form>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Members</CardTitle>
            <CardDescription>Manage organization members</CardDescription>
          </CardHeader>
          <CardContent>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                addMemberMutation.mutate();
              }}
              className="flex gap-3 mb-4"
            >
              <Input
                value={memberEmail}
                onChange={(e) => setMemberEmail(e.target.value)}
                placeholder="user@example.com"
                type="email"
                className="flex-1"
              />
              <Button type="submit" disabled={addMemberMutation.isPending}>
                <Plus className="mr-1 h-4 w-4" /> Add
              </Button>
            </form>
            <div className="space-y-2">
              {members?.map((m) => (
                <div
                  key={m.user_id}
                  className="flex items-center justify-between py-2 border-b last:border-0"
                >
                  <div>
                    <p className="text-sm font-medium">{m.name || m.email}</p>
                    <p className="text-xs text-muted-foreground">
                      {m.email} <Badge variant="secondary" className="ml-1">{m.role}</Badge>
                    </p>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => removeMemberMutation.mutate(m.user_id)}
                  >
                    <Trash2 className="h-4 w-4 text-destructive" />
                  </Button>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
