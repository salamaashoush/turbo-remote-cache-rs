import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getUser,
  activateUser,
  deactivateUser,
  deleteUser,
  forceLogout,
  adminPasswordReset,
  addUserToOrg,
  removeUserFromOrg,
  listAllOrgs,
} from '~/api/admin';
import { toast } from 'sonner';
import { ArrowLeft, LogOut, Trash2, Mail, UserPlus, X, Shield, ShieldOff } from 'lucide-react';
import { Button } from '~/components/ui/button';
import { Badge } from '~/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '~/components/ui/card';
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

export function AdminUserDetail() {
  const { userId } = useParams<{ userId: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [addOrgOpen, setAddOrgOpen] = useState(false);
  const [selectedOrgId, setSelectedOrgId] = useState('');
  const [selectedRole, setSelectedRole] = useState('member');

  const { data: user, isLoading } = useQuery({
    queryKey: ['admin-user', userId],
    queryFn: () => getUser(userId!),
    enabled: !!userId,
  });

  const { data: allOrgs } = useQuery({
    queryKey: ['admin-orgs'],
    queryFn: () => listAllOrgs(200, 0),
    enabled: addOrgOpen,
  });

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['admin-user', userId] });
    queryClient.invalidateQueries({ queryKey: ['admin-users'] });
  };

  const activateMutation = useMutation({
    mutationFn: () => activateUser(userId!),
    onSuccess: () => { invalidate(); toast.success('User activated'); },
  });

  const deactivateMutation = useMutation({
    mutationFn: () => deactivateUser(userId!),
    onSuccess: () => { invalidate(); toast.success('User deactivated'); },
  });

  const deleteMutation = useMutation({
    mutationFn: () => deleteUser(userId!),
    onSuccess: () => {
      toast.success('User deleted');
      navigate('/admin/users');
    },
  });

  const logoutMutation = useMutation({
    mutationFn: () => forceLogout(userId!),
    onSuccess: () => { invalidate(); toast.success('All sessions terminated'); },
  });

  const resetMutation = useMutation({
    mutationFn: () => adminPasswordReset(userId!),
    onSuccess: () => toast.success('Password reset email sent'),
    onError: () => toast.error('Failed to send reset email (SMTP may not be configured)'),
  });

  const addOrgMutation = useMutation({
    mutationFn: () => addUserToOrg(userId!, selectedOrgId, selectedRole),
    onSuccess: () => {
      invalidate();
      setAddOrgOpen(false);
      setSelectedOrgId('');
      toast.success('User added to organization');
    },
    onError: () => toast.error('Failed to add user to organization'),
  });

  const removeOrgMutation = useMutation({
    mutationFn: (orgId: string) => removeUserFromOrg(userId!, orgId),
    onSuccess: () => { invalidate(); toast.success('Removed from organization'); },
  });

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-64" />
      </div>
    );
  }

  if (!user) {
    return <div className="text-center py-12 text-muted-foreground">User not found</div>;
  }

  // Filter orgs user is NOT already in for the add dialog
  const availableOrgs = allOrgs?.filter(
    (o) => !user.orgs.some((uo) => uo.id === o.id),
  ) ?? [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate('/admin/users')}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <div className="flex-1">
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-bold">{user.name || user.email}</h1>
            <Badge variant={user.role === 'super_admin' ? 'default' : 'secondary'}>
              {user.role}
            </Badge>
            <Badge variant={user.is_active ? 'default' : 'destructive'}>
              {user.is_active ? 'Active' : 'Inactive'}
            </Badge>
          </div>
          <p className="text-muted-foreground">{user.email}</p>
        </div>
      </div>

      {/* Actions bar */}
      <div className="flex flex-wrap gap-2">
        {user.is_active ? (
          <Button
            variant="outline"
            size="sm"
            onClick={() => deactivateMutation.mutate()}
            disabled={deactivateMutation.isPending}
          >
            <ShieldOff className="h-4 w-4 mr-1" />
            Deactivate
          </Button>
        ) : (
          <Button
            variant="outline"
            size="sm"
            onClick={() => activateMutation.mutate()}
            disabled={activateMutation.isPending}
          >
            <Shield className="h-4 w-4 mr-1" />
            Activate
          </Button>
        )}
        <Button
          variant="outline"
          size="sm"
          onClick={() => logoutMutation.mutate()}
          disabled={logoutMutation.isPending}
        >
          <LogOut className="h-4 w-4 mr-1" />
          Force Logout
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => resetMutation.mutate()}
          disabled={resetMutation.isPending}
        >
          <Mail className="h-4 w-4 mr-1" />
          Send Password Reset
        </Button>
        <Button
          variant="destructive"
          size="sm"
          onClick={() => setConfirmDelete(true)}
        >
          <Trash2 className="h-4 w-4 mr-1" />
          Delete User
        </Button>
      </div>

      <Separator />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Account Info */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Account Info</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Email verified</span>
              <Badge variant={user.email_verified ? 'default' : 'secondary'}>
                {user.email_verified ? 'Yes' : 'No'}
              </Badge>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">2FA</span>
              <span>
                {user.twofa_enabled ? (
                  <Badge>{user.twofa_method.toUpperCase()}</Badge>
                ) : (
                  <span className="text-muted-foreground">Disabled</span>
                )}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Created</span>
              <span>{new Date(user.created_at).toLocaleDateString()}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Last active</span>
              <span>{user.last_active ? new Date(user.last_active).toLocaleString() : 'Never'}</span>
            </div>
          </CardContent>
        </Card>

        {/* Sessions & Tokens */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Sessions & Tokens</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Active sessions</span>
              <span className="font-mono">{user.session_count}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Active API tokens</span>
              <span className="font-mono">{user.token_count}</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Organizations */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="text-lg">Organizations</CardTitle>
          <Button variant="outline" size="sm" onClick={() => setAddOrgOpen(true)}>
            <UserPlus className="h-4 w-4 mr-1" />
            Add to Organization
          </Button>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Slug</TableHead>
                <TableHead>Role</TableHead>
                <TableHead className="w-[60px]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {user.orgs.map((org) => (
                <TableRow key={org.id}>
                  <TableCell
                    className="font-medium cursor-pointer hover:underline"
                    onClick={() => navigate(`/admin/orgs/${org.id}`)}
                  >
                    {org.name}
                  </TableCell>
                  <TableCell className="font-mono text-muted-foreground">{org.slug}</TableCell>
                  <TableCell>
                    <Badge variant="secondary">{org.role}</Badge>
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => removeOrgMutation.mutate(org.id)}
                      disabled={removeOrgMutation.isPending}
                    >
                      <X className="h-4 w-4 text-destructive" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
              {user.orgs.length === 0 && (
                <TableRow>
                  <TableCell colSpan={4} className="text-center text-muted-foreground py-6">
                    Not a member of any organization
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
            <DialogTitle>Delete user "{user.name || user.email}"?</DialogTitle>
            <DialogDescription>
              This will permanently delete the user and all associated data (sessions, tokens, memberships). This cannot be undone.
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

      {/* Add to Org Dialog */}
      <Dialog open={addOrgOpen} onOpenChange={setAddOrgOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add to Organization</DialogTitle>
            <DialogDescription>Select an organization and role.</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Organization</label>
              <Select value={selectedOrgId} onValueChange={setSelectedOrgId}>
                <SelectTrigger>
                  <SelectValue placeholder="Select organization..." />
                </SelectTrigger>
                <SelectContent>
                  {availableOrgs.map((org) => (
                    <SelectItem key={org.id} value={org.id}>{org.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Role</label>
              <Select value={selectedRole} onValueChange={setSelectedRole}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="member">member</SelectItem>
                  <SelectItem value="admin">admin</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddOrgOpen(false)}>Cancel</Button>
            <Button
              onClick={() => addOrgMutation.mutate()}
              disabled={!selectedOrgId || addOrgMutation.isPending}
            >
              {addOrgMutation.isPending ? 'Adding...' : 'Add'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
