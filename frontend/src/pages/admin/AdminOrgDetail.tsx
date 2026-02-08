import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getOrg,
  deleteOrg,
  updateOrgLimits,
  transferOrgOwnership,
  purgeOrgCache,
  listAllUsers,
} from '~/api/admin';
import { toast } from 'sonner';
import {
  ArrowLeft,
  Trash2,
  ArrowRightLeft,
  Database,
  Save,
} from 'lucide-react';
import { Button } from '~/components/ui/button';
import { Badge } from '~/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui/card';
import { Input } from '~/components/ui/input';
import { Label } from '~/components/ui/label';
import { Skeleton } from '~/components/ui/skeleton';
import { Separator } from '~/components/ui/separator';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '~/components/ui/table';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

type LimitUnit = 'MB' | 'GB' | 'TB';

function bytesToUnit(bytes: number | null, unit: LimitUnit): string {
  if (bytes === null) return '';
  const divisors: Record<LimitUnit, number> = {
    MB: 1024 * 1024,
    GB: 1024 * 1024 * 1024,
    TB: 1024 * 1024 * 1024 * 1024,
  };
  return (bytes / divisors[unit]).toString();
}

function unitToBytes(value: string, unit: LimitUnit): number | null {
  if (!value) return null;
  const num = parseFloat(value);
  if (isNaN(num)) return null;
  const multipliers: Record<LimitUnit, number> = {
    MB: 1024 * 1024,
    GB: 1024 * 1024 * 1024,
    TB: 1024 * 1024 * 1024 * 1024,
  };
  return Math.round(num * multipliers[unit]);
}

export function AdminOrgDetail() {
  const { orgId } = useParams<{ orgId: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [confirmPurge, setConfirmPurge] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [selectedOwnerId, setSelectedOwnerId] = useState('');

  // Limits state
  const [limitUnit, setLimitUnit] = useState<LimitUnit>('GB');
  const [limitValue, setLimitValue] = useState('');
  const [maxTokens, setMaxTokens] = useState('');
  const [limitsInitialized, setLimitsInitialized] = useState(false);

  const { data: org, isLoading } = useQuery({
    queryKey: ['admin-org', orgId],
    queryFn: () => getOrg(orgId!),
    enabled: !!orgId,
  });

  // Initialize limits from org data
  if (org && !limitsInitialized) {
    setLimitValue(bytesToUnit(org.cache_size_limit_bytes, limitUnit));
    setMaxTokens(org.max_tokens !== null ? org.max_tokens.toString() : '');
    setLimitsInitialized(true);
  }

  const { data: allUsers } = useQuery({
    queryKey: ['admin-users-for-transfer'],
    queryFn: () => listAllUsers(200, 0),
    enabled: transferOpen,
  });

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['admin-org', orgId] });
    queryClient.invalidateQueries({ queryKey: ['admin-orgs'] });
  };

  const deleteMutation = useMutation({
    mutationFn: () => deleteOrg(orgId!),
    onSuccess: () => {
      toast.success('Organization deleted');
      navigate('/admin/orgs');
    },
  });

  const purgeMutation = useMutation({
    mutationFn: () => purgeOrgCache(orgId!),
    onSuccess: (data) => {
      setConfirmPurge(false);
      toast.success(`Purged ${data.count} artifacts`);
    },
  });

  const limitsMutation = useMutation({
    mutationFn: () =>
      updateOrgLimits(orgId!, {
        cache_size_limit_bytes: unitToBytes(limitValue, limitUnit),
        max_tokens: maxTokens ? parseInt(maxTokens, 10) : null,
      }),
    onSuccess: () => { invalidate(); toast.success('Limits updated'); },
  });

  const transferMutation = useMutation({
    mutationFn: () => transferOrgOwnership(orgId!, selectedOwnerId),
    onSuccess: () => {
      invalidate();
      setTransferOpen(false);
      toast.success('Ownership transferred');
    },
  });

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-64" />
      </div>
    );
  }

  if (!org) {
    return <div className="text-center py-12 text-muted-foreground">Organization not found</div>;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate('/admin/orgs')}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <div>
          <h1 className="text-2xl font-bold">{org.name}</h1>
          <p className="text-muted-foreground font-mono">{org.slug}</p>
        </div>
      </div>

      {/* Actions bar */}
      <div className="flex flex-wrap gap-2">
        <Button variant="outline" size="sm" onClick={() => setTransferOpen(true)}>
          <ArrowRightLeft className="h-4 w-4 mr-1" />
          Transfer Ownership
        </Button>
        <Button variant="outline" size="sm" onClick={() => setConfirmPurge(true)}>
          <Database className="h-4 w-4 mr-1" />
          Purge Cache
        </Button>
        <Button
          variant="destructive"
          size="sm"
          onClick={() => setConfirmDelete(true)}
        >
          <Trash2 className="h-4 w-4 mr-1" />
          Delete Org
        </Button>
      </div>

      <Separator />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Owner */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Owner</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Name</span>
              <span>{org.owner.name}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Email</span>
              <span>{org.owner.email}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Created</span>
              <span>{new Date(org.created_at).toLocaleDateString()}</span>
            </div>
          </CardContent>
        </Card>

        {/* Cache Stats */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Cache Stats</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total cache size</span>
              <span className="font-mono">{formatBytes(org.total_cache_bytes)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total events</span>
              <span className="font-mono">{org.total_cache_events.toLocaleString()}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Active tokens</span>
              <span className="font-mono">{org.token_count}</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Limits */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Limits</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>Cache size limit</Label>
              <div className="flex gap-2">
                <Input
                  type="number"
                  placeholder="Unlimited"
                  value={limitValue}
                  onChange={(e) => setLimitValue(e.target.value)}
                />
                <Select value={limitUnit} onValueChange={(v) => setLimitUnit(v as LimitUnit)}>
                  <SelectTrigger className="w-[80px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="MB">MB</SelectItem>
                    <SelectItem value="GB">GB</SelectItem>
                    <SelectItem value="TB">TB</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <p className="text-xs text-muted-foreground">Leave empty for unlimited</p>
            </div>
            <div className="space-y-2">
              <Label>Max API tokens</Label>
              <Input
                type="number"
                placeholder="Unlimited"
                value={maxTokens}
                onChange={(e) => setMaxTokens(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">Leave empty for unlimited</p>
            </div>
          </div>
          <Button
            className="mt-4"
            size="sm"
            onClick={() => limitsMutation.mutate()}
            disabled={limitsMutation.isPending}
          >
            <Save className="h-4 w-4 mr-1" />
            {limitsMutation.isPending ? 'Saving...' : 'Save Limits'}
          </Button>
        </CardContent>
      </Card>

      {/* Members */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Members ({org.members.length})</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Email</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Role</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {org.members.map((m) => (
                <TableRow key={m.id}>
                  <TableCell
                    className="cursor-pointer hover:underline"
                    onClick={() => navigate(`/admin/users/${m.id}`)}
                  >
                    {m.email}
                  </TableCell>
                  <TableCell>{m.name || '-'}</TableCell>
                  <TableCell>
                    <Badge variant="secondary">{m.role}</Badge>
                  </TableCell>
                </TableRow>
              ))}
              {org.members.length === 0 && (
                <TableRow>
                  <TableCell colSpan={3} className="text-center text-muted-foreground py-6">
                    No members
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Teams */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Teams ({org.teams.length})</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead className="text-right">Members</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {org.teams.map((t) => (
                <TableRow key={t.id}>
                  <TableCell className="font-medium">{t.name}</TableCell>
                  <TableCell className="text-right">{t.member_count}</TableCell>
                </TableRow>
              ))}
              {org.teams.length === 0 && (
                <TableRow>
                  <TableCell colSpan={2} className="text-center text-muted-foreground py-6">
                    No teams
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Delete Confirmation */}
      <Dialog open={confirmDelete} onOpenChange={setConfirmDelete}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete organization "{org.name}"?</DialogTitle>
            <DialogDescription>
              This will permanently delete the organization and all its data. This cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmDelete(false)}>Cancel</Button>
            <Button
              variant="destructive"
              onClick={() => deleteMutation.mutate()}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Purge Confirmation */}
      <Dialog open={confirmPurge} onOpenChange={setConfirmPurge}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Purge all cached artifacts?</DialogTitle>
            <DialogDescription>
              This will delete all cached artifacts for "{org.name}". Teams will need to rebuild their caches.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmPurge(false)}>Cancel</Button>
            <Button
              variant="destructive"
              onClick={() => purgeMutation.mutate()}
              disabled={purgeMutation.isPending}
            >
              {purgeMutation.isPending ? 'Purging...' : 'Purge'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Transfer Ownership Dialog */}
      <Dialog open={transferOpen} onOpenChange={setTransferOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Transfer Ownership</DialogTitle>
            <DialogDescription>
              Select the new owner for "{org.name}".
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Select value={selectedOwnerId} onValueChange={setSelectedOwnerId}>
              <SelectTrigger>
                <SelectValue placeholder="Select new owner..." />
              </SelectTrigger>
              <SelectContent>
                {allUsers
                  ?.filter((u) => u.id !== org.owner.id)
                  .map((u) => (
                    <SelectItem key={u.id} value={u.id}>
                      {u.email} ({u.name || 'No name'})
                    </SelectItem>
                  ))}
              </SelectContent>
            </Select>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTransferOpen(false)}>Cancel</Button>
            <Button
              onClick={() => transferMutation.mutate()}
              disabled={!selectedOwnerId || transferMutation.isPending}
            >
              {transferMutation.isPending ? 'Transferring...' : 'Transfer'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
