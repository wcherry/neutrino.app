'use client';

import React from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import { Badge, Button, Card, EmptyState, Heading, Spinner, Text } from '@neutrino/ui';
import { ApiClientError, getShareDownloadUrl, getSharePreviewUrl, sharingApi } from '@/lib/api';
import styles from './page.module.css';

function formatExpiresAt(expiresAt: string | null): string | null {
  if (!expiresAt) return null;
  const date = new Date(expiresAt);
  if (Number.isNaN(date.getTime())) return expiresAt;
  return date.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

export default function ShareTokenClient() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const token = searchParams.get('token') ?? '';

  const { data, isLoading, error } = useQuery({
    queryKey: ['share-token', token],
    enabled: Boolean(token),
    retry: false,
    queryFn: () => sharingApi.resolveToken(token),
  });

  if (!token) {
    return (
      <div className={styles.page}>
        <EmptyState
          title="Invalid share link"
          description="This share link is missing a token."
        />
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className={styles.page}>
        <Spinner size="lg" />
      </div>
    );
  }

  if (error) {
    const status = error instanceof ApiClientError ? error.statusCode : null;
    const title = status === 410 ? 'This link has expired' : 'Share link not found';
    const description = status === 410
      ? 'Ask the owner to generate a new share link.'
      : 'The link may be invalid or has been disabled.';
    return (
      <div className={styles.page}>
        <EmptyState title={title} description={description} />
      </div>
    );
  }

  if (!data) {
    return (
      <div className={styles.page}>
        <EmptyState
          title="Share link not found"
          description="The link may be invalid or has been disabled."
        />
      </div>
    );
  }

  const expiresAt = formatExpiresAt(data.expiresAt);
  const isFile = data.resourceType === 'file';
  const downloadUrl = getShareDownloadUrl(token);
  const previewUrl = getSharePreviewUrl(token);

  return (
    <div className={styles.page}>
      <Card className={styles.card}>
        <div className={styles.header}>
          <Text size="xs" color="muted" weight="semibold">Shared item</Text>
          <Heading level={1} size="lg">{data.resourceName}</Heading>
          <div className={styles.badges}>
            <Badge size="sm" >{data.resourceType}</Badge>
            <Badge size="sm" >{data.role}</Badge>
            <Badge size="sm" >{data.visibility}</Badge>
          </div>
        </div>

        <div className={styles.meta}>
          {expiresAt && (
            <Text size="sm" color="muted">Expires {expiresAt}</Text>
          )}
        </div>

        <div className={styles.actions}>
          {isFile ? (
            <>
              <Button onClick={() => window.location.assign(previewUrl)}>
                View
              </Button>
              <Button variant="secondary" onClick={() => window.location.assign(downloadUrl)}>
                Download
              </Button>
            </>
          ) : (
            <Button onClick={() => router.push('/drive')}>
              Open in Drive
            </Button>
          )}
          <Button variant="secondary" onClick={() => router.push('/')}>
            Go to home
          </Button>
        </div>
      </Card>
    </div>
  );
}
