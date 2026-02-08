import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listArtifacts, purgeArtifact, purgeAll } from '~/api/analytics';
import { toast } from 'sonner';
import { Trash2 } from 'lucide-react';
import { Button } from '~/components/ui/button';
import { Card } from '~/components/ui/card';
import { Badge } from '~/components/ui/badge';
import { Skeleton } from '~/components/ui/skeleton';
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
  DialogTrigger,
} from '~/components/ui/dialog';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function ArtifactBrowser() {
  const { orgId } = useParams<{ orgId: string }>();
  const queryClient = useQueryClient();
  const { data: artifacts, isLoading } = useQuery({
    queryKey: ['artifacts', orgId],
    queryFn: () => listArtifacts(orgId!),
  });
  const [confirmPurgeAll, setConfirmPurgeAll] = useState(false);

  const purgeMutation = useMutation({
    mutationFn: (hash: string) => purgeArtifact(orgId!, hash),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['artifacts', orgId] });
      toast.success('Artifact purged');
    },
  });

  const purgeAllMutation = useMutation({
    mutationFn: () => purgeAll(orgId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['artifacts', orgId] });
      setConfirmPurgeAll(false);
      toast.success('All artifacts purged');
    },
  });

  const eventBadgeVariant = (event: string) => {
    if (event === 'hit') return 'default' as const;
    if (event === 'miss') return 'destructive' as const;
    return 'secondary' as const;
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-4">Cache</h1>

      <div className="flex justify-between items-center mb-4">
        <p className="text-sm text-muted-foreground">{artifacts?.length ?? 0} artifacts</p>
        <Dialog open={confirmPurgeAll} onOpenChange={setConfirmPurgeAll}>
          <DialogTrigger asChild>
            <Button variant="destructive" size="sm">
              Purge All
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Purge all artifacts?</DialogTitle>
              <DialogDescription>
                This will permanently delete all cached artifacts. This cannot be undone.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" onClick={() => setConfirmPurgeAll(false)}>
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => purgeAllMutation.mutate()}
                disabled={purgeAllMutation.isPending}
              >
                {purgeAllMutation.isPending ? 'Purging...' : 'Purge All'}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton key={i} className="h-12" />
          ))}
        </div>
      ) : (
        <Card>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Hash</TableHead>
                <TableHead>Last Event</TableHead>
                <TableHead>Events</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Last Seen</TableHead>
                <TableHead className="w-[50px]" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {artifacts?.map((a) => (
                <TableRow key={a.artifact_hash}>
                  <TableCell className="font-mono text-xs">
                    {a.artifact_hash.substring(0, 16)}...
                  </TableCell>
                  <TableCell>
                    <Badge variant={eventBadgeVariant(a.last_event)}>{a.last_event}</Badge>
                  </TableCell>
                  <TableCell className="text-muted-foreground">{a.total_events}</TableCell>
                  <TableCell className="text-muted-foreground">
                    {formatBytes(a.total_bytes)}
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {new Date(a.last_seen).toLocaleDateString()}
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => purgeMutation.mutate(a.artifact_hash)}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
              {artifacts?.length === 0 && (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                    No artifacts found
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </Card>
      )}
    </div>
  );
}
