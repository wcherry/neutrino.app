'use client';

import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Modal,
  ModalHeader,
  ModalBody,
  Button,
  Text,
  Badge,
  Avatar,
  Spinner,
  Divider,
  useToast,
} from '@neutrino/ui';
import { Link, Copy, Trash2, Check, UserPlus, Globe, Lock } from 'lucide-react';
import {
  permissionsApi,
  sharingApi,
  usersApi,
  type FileItem,
  type Folder as FolderItem,
  type Permission,
  type ShareLink,
  type PermissionRole,
  type ResourceType,
} from '@/lib/api';
import styles from './ShareDialog.module.css';

interface Props {
  resource: FileItem | FolderItem;
  resourceType: ResourceType;
  onClose: () => void;
}

const ROLE_LABELS: Record<string, string> = {
  owner: 'Owner',
  editor: 'Editor',
  commenter: 'Commenter',
  viewer: 'Viewer',
};

const LINK_ROLE_OPTIONS = [
  { value: 'viewer', label: 'Viewer' },
  { value: 'commenter', label: 'Commenter' },
  { value: 'editor', label: 'Editor' },
];

const PERM_ROLE_OPTIONS: { value: PermissionRole; label: string }[] = [
  { value: 'viewer', label: 'Viewer' },
  { value: 'commenter', label: 'Commenter' },
  { value: 'editor', label: 'Editor' },
];

export function ShareDialog({ resource, resourceType, onClose }: Props) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const [emailInput, setEmailInput] = useState('');
  const [selectedRole, setSelectedRole] = useState<PermissionRole>('viewer');
  const [lookupLoading, setLookupLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  const permsKey = ['permissions', resourceType, resource.id];
  const linkKey = ['share-link', resourceType, resource.id];

  const { data: permsData, isLoading: permsLoading } = useQuery({
    queryKey: permsKey,
    queryFn: () => permissionsApi.listPermissions(resourceType, resource.id),
    retry: false,
  });

  const { data: shareLink, isLoading: linkLoading } = useQuery({
    queryKey: linkKey,
    queryFn: () => sharingApi.getShareLink(resourceType, resource.id),
    retry: false,
  });

  const grantMutation = useMutation({
    mutationFn: (req: Parameters<typeof permissionsApi.grantPermission>[2]) =>
      permissionsApi.grantPermission(resourceType, resource.id, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: permsKey });
      setEmailInput('');
      toast.success('Access granted');
    },
    onError: (e: Error) => toast.error(e.message),
  });

  const updateMutation = useMutation({
    mutationFn: ({ userId, role }: { userId: string; role: PermissionRole }) =>
      permissionsApi.updatePermission(resourceType, resource.id, userId, { role }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: permsKey }),
    onError: (e: Error) => toast.error(e.message),
  });

  const revokeMutation = useMutation({
    mutationFn: (userId: string) =>
      permissionsApi.revokePermission(resourceType, resource.id, userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: permsKey });
      toast.success('Access removed');
    },
    onError: (e: Error) => toast.error(e.message),
  });

  const createLinkMutation = useMutation({
    mutationFn: () =>
      sharingApi.upsertShareLink(resourceType, resource.id, {
        visibility: 'anyoneWithLink',
        role: 'viewer',
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: linkKey }),
    onError: (e: Error) => toast.error(e.message),
  });

  const toggleLinkMutation = useMutation({
    mutationFn: (isActive: boolean) =>
      sharingApi.updateShareLink(resourceType, resource.id, { isActive }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: linkKey }),
    onError: (e: Error) => toast.error(e.message),
  });

  const updateLinkRoleMutation = useMutation({
    mutationFn: (role: string) =>
      sharingApi.updateShareLink(resourceType, resource.id, { role: role as ShareLink['role'] }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: linkKey }),
    onError: (e: Error) => toast.error(e.message),
  });

  const deleteLinkMutation = useMutation({
    mutationFn: () => sharingApi.deleteShareLink(resourceType, resource.id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: linkKey }),
    onError: (e: Error) => toast.error(e.message),
  });

  async function handleAddPerson() {
    if (!emailInput.trim()) return;
    setLookupLoading(true);
    try {
      const user = await usersApi.lookupByEmail(emailInput.trim());
      if (!user) {
        toast.error('No user found with that email address');
        return;
      }
      grantMutation.mutate({
        userId: user.id,
        userEmail: user.email,
        userName: user.name,
        role: selectedRole,
      });
    } catch {
      toast.error('Failed to look up user');
    } finally {
      setLookupLoading(false);
    }
  }

  function getLinkUrl(link: ShareLink): string {
    if (typeof window === 'undefined') return '';
    return `${window.location.origin}/share/${link.token}`;
  }

  function handleCopyLink(link: ShareLink) {
    const url = getLinkUrl(link);
    navigator.clipboard.writeText(url).then(
      () => {
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      },
      () => toast.error('Failed to copy link')
    );
  }

  const permissions = permsData?.permissions ?? [];
  const nonOwners = permissions.filter((p) => p.role !== 'owner');
  const owners = permissions.filter((p) => p.role === 'owner');

  return (
    <Modal isOpen onClose={onClose} size="md">
      <ModalHeader title={`Share "${resource.name}"`} onClose={onClose} />
      <ModalBody>
        {/* Add people */}
        <div className={styles.section}>
          <Text size="sm" weight="semibold">Add people</Text>
          <div className={styles.addRow}>
            <input
              className={styles.emailInput}
              type="email"
              placeholder="Enter email address"
              value={emailInput}
              onChange={(e) => setEmailInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddPerson()}
              aria-label="Email address to share with"
            />
            <select
              className={styles.roleSelect}
              value={selectedRole}
              onChange={(e) => setSelectedRole(e.target.value as PermissionRole)}
              aria-label="Role"
            >
              {PERM_ROLE_OPTIONS.map((o) => (
                <option key={o.value} value={o.value}>{o.label}</option>
              ))}
            </select>
            <Button
              variant="primary"
              size="sm"
              icon={<UserPlus size={14} />}
              onClick={handleAddPerson}
              disabled={!emailInput.trim() || lookupLoading || grantMutation.isPending}
            >
              {lookupLoading || grantMutation.isPending ? 'Adding…' : 'Add'}
            </Button>
          </div>
        </div>

        <Divider spacing="sm" />

        {/* Collaborators */}
        <div className={styles.section}>
          <Text size="sm" weight="semibold">People with access</Text>
          {permsLoading ? (
            <div className={styles.loadingRow}><Spinner size="sm" /></div>
          ) : permissions.length === 0 ? (
            <Text size="sm" color="muted">No collaborators yet.</Text>
          ) : (
            <div className={styles.collaboratorList}>
              {[...owners, ...nonOwners].map((perm) => (
                <CollaboratorRow
                  key={perm.id}
                  perm={perm}
                  onRoleChange={(role) => updateMutation.mutate({ userId: perm.userId, role })}
                  onRevoke={() => revokeMutation.mutate(perm.userId)}
                  isPending={updateMutation.isPending || revokeMutation.isPending}
                />
              ))}
            </div>
          )}
        </div>

        <Divider spacing="sm" />

        {/* Share link */}
        <div className={styles.section}>
          <Text size="sm" weight="semibold">Share link</Text>
          {linkLoading ? (
            <div className={styles.loadingRow}><Spinner size="sm" /></div>
          ) : shareLink ? (
            <ShareLinkSection
              link={shareLink}
              onCopy={() => handleCopyLink(shareLink)}
              copied={copied}
              onToggle={(active) => toggleLinkMutation.mutate(active)}
              onRoleChange={(role) => updateLinkRoleMutation.mutate(role)}
              onDelete={() => deleteLinkMutation.mutate()}
              isPending={toggleLinkMutation.isPending || updateLinkRoleMutation.isPending || deleteLinkMutation.isPending}
            />
          ) : (
            <div className={styles.noLink}>
              <Text size="sm" color="muted">No shareable link yet.</Text>
              <Button
                variant="secondary"
                size="sm"
                icon={<Link size={14} />}
                onClick={() => createLinkMutation.mutate()}
                disabled={createLinkMutation.isPending}
              >
                {createLinkMutation.isPending ? 'Creating…' : 'Create link'}
              </Button>
            </div>
          )}
        </div>
      </ModalBody>
    </Modal>
  );
}

function CollaboratorRow({
  perm,
  onRoleChange,
  onRevoke,
  isPending,
}: {
  perm: Permission;
  onRoleChange: (role: PermissionRole) => void;
  onRevoke: () => void;
  isPending: boolean;
}) {
  const isOwner = perm.role === 'owner';
  const displayName = perm.userName || perm.userEmail || perm.userId;
  const displayEmail = perm.userEmail || perm.userId;

  return (
    <div className={styles.collaborator}>
      <Avatar
        name={displayName}
        size="sm"
      />
      <div className={styles.collaboratorInfo}>
        <Text size="sm" weight="medium">{displayName}</Text>
        <Text size="xs" color="muted">{displayEmail}</Text>
      </div>
      {isOwner ? (
        <Badge variant="default" size="sm">Owner</Badge>
      ) : (
        <>
          <select
            className={styles.roleSelectSm}
            value={perm.role}
            onChange={(e) => onRoleChange(e.target.value as PermissionRole)}
            disabled={isPending}
            aria-label={`Role for ${displayName}`}
          >
            {PERM_ROLE_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>{o.label}</option>
            ))}
          </select>
          <button
            type="button"
            className={styles.revokeBtn}
            onClick={onRevoke}
            disabled={isPending}
            aria-label={`Remove access for ${displayName}`}
            title="Remove access"
          >
            <Trash2 size={13} />
          </button>
        </>
      )}
    </div>
  );
}

function ShareLinkSection({
  link,
  onCopy,
  copied,
  onToggle,
  onRoleChange,
  onDelete,
  isPending,
}: {
  link: ShareLink;
  onCopy: () => void;
  copied: boolean;
  onToggle: (active: boolean) => void;
  onRoleChange: (role: string) => void;
  onDelete: () => void;
  isPending: boolean;
}) {
  return (
    <div className={styles.linkSection}>
      <div className={styles.linkStatusRow}>
        <span className={styles.linkIcon}>
          {link.isActive ? <Globe size={15} /> : <Lock size={15} />}
        </span>
        <div className={styles.linkStatus}>
          <Text size="sm" weight="medium">
            {link.isActive ? 'Link sharing is on' : 'Link sharing is off'}
          </Text>
          <Text size="xs" color="muted">
            {link.isActive
              ? link.visibility === 'public'
                ? 'Anyone on the internet can find and access'
                : 'Anyone with the link can access'
              : 'Only people with permission can access'}
          </Text>
        </div>
        <Button
          variant={link.isActive ? 'ghost' : 'secondary'}
          size="sm"
          onClick={() => onToggle(!link.isActive)}
          disabled={isPending}
        >
          {link.isActive ? 'Turn off' : 'Turn on'}
        </Button>
      </div>

      {link.isActive && (
        <>
          <div className={styles.linkRoleRow}>
            <Text size="xs" color="muted">Anyone with link can:</Text>
            <select
              className={styles.roleSelectSm}
              value={link.role}
              onChange={(e) => onRoleChange(e.target.value)}
              disabled={isPending}
              aria-label="Link sharing role"
            >
              {LINK_ROLE_OPTIONS.map((o) => (
                <option key={o.value} value={o.value}>{o.label}</option>
              ))}
            </select>
          </div>
          <div className={styles.linkActions}>
            <Button
              variant="secondary"
              size="sm"
              icon={copied ? <Check size={14} /> : <Copy size={14} />}
              onClick={onCopy}
            >
              {copied ? 'Copied!' : 'Copy link'}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              icon={<Trash2 size={14} />}
              onClick={onDelete}
              disabled={isPending}
            >
              Remove link
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
