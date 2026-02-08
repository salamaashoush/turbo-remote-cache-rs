import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listTeams,
  createTeam,
  deleteTeam,
  listTeamMembers,
  addTeamMember,
  removeTeamMember,
} from '~/api/teams';
import { listMembers } from '~/api/orgs';
import { toast } from 'sonner';
import { Plus, Trash2, ChevronDown, ChevronRight, Users, Settings } from 'lucide-react';
import { Button } from '~/components/ui/button';
import { Card, CardContent } from '~/components/ui/card';
import { Input } from '~/components/ui/input';
import { Label } from '~/components/ui/label';
import { Skeleton } from '~/components/ui/skeleton';
import { Badge } from '~/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '~/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select';

function toSlug(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

export function TeamManagement() {
  const { orgId } = useParams<{ orgId: string }>();
  const queryClient = useQueryClient();
  const { data: teams, isLoading } = useQuery({
    queryKey: ['teams', orgId],
    queryFn: () => listTeams(orgId!),
  });
  const [showCreate, setShowCreate] = useState(false);
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [slugTouched, setSlugTouched] = useState(false);
  const [expandedTeam, setExpandedTeam] = useState<string | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  const createMutation = useMutation({
    mutationFn: () => createTeam(orgId!, name, slug),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['teams', orgId] });
      setShowCreate(false);
      setName('');
      setSlug('');
      setSlugTouched(false);
      toast.success('Team created');
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const deleteMutation = useMutation({
    mutationFn: (teamId: string) => deleteTeam(orgId!, teamId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['teams', orgId] });
      setConfirmDeleteId(null);
      toast.success('Team deleted');
    },
    onError: (err: Error) => toast.error(err.message),
  });

  function handleNameChange(value: string) {
    setName(value);
    if (!slugTouched) {
      setSlug(toSlug(value));
    }
  }

  function handleSlugChange(value: string) {
    setSlugTouched(true);
    setSlug(value);
  }

  return (
    <div>
      <h1 className="text-2xl font-bold mb-4">Teams</h1>

      <div className="flex justify-between items-center mb-4">
        <p className="text-sm text-muted-foreground">{teams?.length ?? 0} teams</p>
        <Dialog
          open={showCreate}
          onOpenChange={(open) => {
            setShowCreate(open);
            if (!open) {
              setName('');
              setSlug('');
              setSlugTouched(false);
            }
          }}
        >
          <DialogTrigger asChild>
            <Button size="sm">
              <Plus className="mr-1 h-4 w-4" /> New Team
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create Team</DialogTitle>
              <DialogDescription>Add a new team to this organization.</DialogDescription>
            </DialogHeader>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                createMutation.mutate();
              }}
              className="space-y-4"
            >
              <div className="space-y-2">
                <Label>Name</Label>
                <Input value={name} onChange={(e) => handleNameChange(e.target.value)} required />
              </div>
              <div className="space-y-2">
                <Label>Slug</Label>
                <Input
                  value={slug}
                  onChange={(e) => handleSlugChange(e.target.value)}
                  required
                  placeholder="auto-generated-from-name"
                />
                <p className="text-xs text-muted-foreground">
                  Used in Turborepo config as the team identifier.
                </p>
              </div>
              <DialogFooter>
                <Button type="button" variant="outline" onClick={() => setShowCreate(false)}>
                  Cancel
                </Button>
                <Button type="submit" disabled={createMutation.isPending}>
                  Create
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 3 }).map((_, i) => (
            <Skeleton key={i} className="h-14" />
          ))}
        </div>
      ) : teams?.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center py-8">
            <Users className="h-8 w-8 text-muted-foreground mb-3" />
            <p className="text-muted-foreground mb-1">No teams yet</p>
            <p className="text-sm text-muted-foreground">
              Create a team to organize your projects and tokens.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2">
          {teams?.map((team) => (
            <Card key={team.id}>
              <div className="flex items-center justify-between px-4 py-3">
                <button
                  onClick={() => setExpandedTeam(expandedTeam === team.id ? null : team.id)}
                  className="flex items-center gap-2 text-sm font-medium hover:text-primary"
                >
                  {expandedTeam === team.id ? (
                    <ChevronDown className="h-4 w-4" />
                  ) : (
                    <ChevronRight className="h-4 w-4" />
                  )}
                  {team.name}
                  <span className="font-mono text-muted-foreground text-xs">({team.slug})</span>
                </button>
                <div className="flex items-center gap-3">
                  <TeamMemberCount orgId={orgId!} teamId={team.id} />
                  <span className="text-xs text-muted-foreground">
                    {new Date(team.created_at).toLocaleDateString()}
                  </span>
                  <Dialog
                    open={confirmDeleteId === team.id}
                    onOpenChange={(open) => setConfirmDeleteId(open ? team.id : null)}
                  >
                    <DialogTrigger asChild>
                      <Button variant="ghost" size="icon">
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </DialogTrigger>
                    <DialogContent>
                      <DialogHeader>
                        <DialogTitle>Delete team "{team.name}"?</DialogTitle>
                        <DialogDescription>
                          This action cannot be undone.
                        </DialogDescription>
                      </DialogHeader>
                      <DialogFooter>
                        <Button variant="outline" onClick={() => setConfirmDeleteId(null)}>
                          Cancel
                        </Button>
                        <Button
                          variant="destructive"
                          onClick={() => deleteMutation.mutate(team.id)}
                        >
                          Delete
                        </Button>
                      </DialogFooter>
                    </DialogContent>
                  </Dialog>
                </div>
              </div>
              {expandedTeam === team.id && <TeamMembersPanel orgId={orgId!} teamId={team.id} />}
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}

function TeamMemberCount({ orgId, teamId }: { orgId: string; teamId: string }) {
  const { data: members } = useQuery({
    queryKey: ['team-members', teamId],
    queryFn: () => listTeamMembers(orgId, teamId),
  });

  if (!members) return null;

  return (
    <Badge variant="secondary" className="text-xs font-normal">
      <Users className="h-3 w-3 mr-1" />
      {members.length}
    </Badge>
  );
}

function TeamMembersPanel({ orgId, teamId }: { orgId: string; teamId: string }) {
  const queryClient = useQueryClient();
  const { data: members, isLoading } = useQuery({
    queryKey: ['team-members', teamId],
    queryFn: () => listTeamMembers(orgId, teamId),
  });
  const { data: orgMembers } = useQuery({
    queryKey: ['members', orgId],
    queryFn: () => listMembers(orgId),
  });
  const [selectedUserId, setSelectedUserId] = useState('');
  const [selectedRole, setSelectedRole] = useState('member');

  const addMutation = useMutation({
    mutationFn: () => addTeamMember(orgId, teamId, selectedUserId, selectedRole),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['team-members', teamId] });
      setSelectedUserId('');
      setSelectedRole('member');
      toast.success('Member added to team');
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const removeMutation = useMutation({
    mutationFn: (userId: string) => removeTeamMember(orgId, teamId, userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['team-members', teamId] });
      toast.success('Member removed from team');
    },
    onError: (err: Error) => toast.error(err.message),
  });

  const memberIds = new Set(members?.map((m) => m.user_id) ?? []);
  const availableMembers = orgMembers?.filter((m) => !memberIds.has(m.user_id)) ?? [];

  return (
    <div className="border-t px-4 py-3 bg-muted/50">
      <div className="flex items-center justify-between mb-3">
        <h4 className="text-sm font-medium">Members</h4>
      </div>

      {availableMembers.length > 0 ? (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (selectedUserId) addMutation.mutate();
          }}
          className="flex items-center gap-2 mb-3"
        >
          <Select value={selectedUserId} onValueChange={setSelectedUserId}>
            <SelectTrigger className="w-[180px] h-8 text-xs">
              <SelectValue placeholder="Select member..." />
            </SelectTrigger>
            <SelectContent>
              {availableMembers.map((m) => (
                <SelectItem key={m.user_id} value={m.user_id}>
                  {m.name || m.email}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={selectedRole} onValueChange={setSelectedRole}>
            <SelectTrigger className="w-[110px] h-8 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="member">Member</SelectItem>
              <SelectItem value="admin">Admin</SelectItem>
            </SelectContent>
          </Select>
          <Button type="submit" size="sm" disabled={!selectedUserId || addMutation.isPending}>
            Add
          </Button>
        </form>
      ) : (
        <div className="flex items-center gap-2 mb-3 text-xs text-muted-foreground">
          <span>
            {orgMembers && orgMembers.length <= 1
              ? 'No other org members to add.'
              : 'All org members are already in this team.'}
          </span>
          <Link
            to={`/orgs/${orgId}/settings`}
            className="inline-flex items-center gap-1 text-primary hover:underline"
          >
            <Settings className="h-3 w-3" />
            Manage org members
          </Link>
        </div>
      )}

      {isLoading ? (
        <Skeleton className="h-8" />
      ) : members?.length === 0 ? (
        <p className="text-xs text-muted-foreground">No members yet.</p>
      ) : (
        <div className="space-y-1">
          {members?.map((m) => (
            <div
              key={m.user_id}
              className="flex items-center justify-between bg-background rounded-md px-3 py-2 text-sm"
            >
              <div className="flex items-center gap-2">
                <span className="font-medium">{m.name || m.email}</span>
                {m.name && (
                  <span className="text-muted-foreground text-xs">{m.email}</span>
                )}
                <Badge variant={m.role === 'admin' ? 'default' : 'secondary'} className="text-[10px] px-1.5 py-0">
                  {m.role}
                </Badge>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={() => removeMutation.mutate(m.user_id)}
                disabled={removeMutation.isPending}
              >
                <Trash2 className="h-3 w-3 text-destructive" />
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
