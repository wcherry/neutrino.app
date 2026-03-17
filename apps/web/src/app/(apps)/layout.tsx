'use client';

import React, { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import {
  AppShell,
  Sidebar,
  Topbar,
  Spinner,
  type NavSection,
  type StorageQuota,
} from '@neutrino/ui';
import {
  HardDrive,
  Users,
  Star,
  Clock,
  Trash2,
  Share2,
  FileTextIcon,
  FileSpreadsheet,
  Presentation,
  Image as ImageIcon,
} from 'lucide-react';
import { authApi, storageApi, type UserProfile, type QuotaInfo } from '@/lib/api';

const NAV_SECTIONS: NavSection[] = [
  {
    id: 'main',
    items: [
      { id: 'my-drive', label: 'My Drive', icon: HardDrive, href: '/drive', active: true },
      { id: 'photos', label: 'Photos', icon: ImageIcon, href: '/photos' },
      { id: 'docs', label: 'Docs', icon: FileTextIcon, href: '/docs' },
      { id: 'sheets', label: 'Sheets', icon: FileSpreadsheet, href: '/sheets' },
      { id: 'slides', label: 'Slides', icon: Presentation, href: '/slides' },
      { id: 'shared', label: 'Shared with me', icon: Share2, href: '/drive/shared' },
      { id: 'recent', label: 'Recent', icon: Clock, href: '/drive/recent' },
      { id: 'starred', label: 'Starred', icon: Star, href: '/drive/starred' },
      { id: 'trash', label: 'Trash', icon: Trash2, href: '/drive/trash' },
    ],
  },
  {
    id: 'team',
    label: 'Team',
    items: [
      { id: 'shared-drives', label: 'Shared Drives', icon: Users, href: '/drive/team' },
    ],
  },
];

const DEFAULT_QUOTA_BYTES = 15 * 1024 * 1024 * 1024; // 15 GB fallback when no server limit set

function quotaFromInfo(info: QuotaInfo): StorageQuota {
  return {
    usedBytes: info.usedBytes,
    totalBytes: info.quotaBytes ?? DEFAULT_QUOTA_BYTES,
  };
}

type AuthState =
  | { status: 'loading' }
  | { status: 'ready'; user: UserProfile; quota: StorageQuota };

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const [auth, setAuth] = useState<AuthState>({ status: 'loading' });

  useEffect(() => {
    async function init() {
      async function fetchProfile(): Promise<UserProfile> {
        try {
          return await authApi.getProfile();
        } catch {
          // Expired access token — try a refresh once.
          await authApi.refresh();
          return authApi.getProfile();
        }
      }

      try {
        const [user, quotaInfo] = await Promise.all([
          fetchProfile(),
          storageApi.getQuota().catch(() => null),
        ]);
        setAuth({
          status: 'ready',
          user,
          quota: quotaInfo
            ? quotaFromInfo(quotaInfo)
            : { usedBytes: 0, totalBytes: DEFAULT_QUOTA_BYTES },
        });
      } catch {
        // Not authenticated or refresh failed — redirect to sign-in.
        router.replace('/sign-in');
      }
    }

    init();
  }, [router]);

  async function handleSignOut() {
    await authApi.logout().catch(() => {});
    router.replace('/sign-in');
  }

  if (auth.status === 'loading') {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh' }}>
        <Spinner size="lg" />
      </div>
    );
  }

  const sidebar = (
    <Sidebar
      logoText="Neutrino"
      logoHref="/drive"
      sections={NAV_SECTIONS}
      quota={auth.quota}
      onUpload={() => {}}
    />
  );

  const topbar = (
    <Topbar
      user={{ name: auth.user.name, email: auth.user.email }}
      onSearch={(q) => console.log('search:', q)}
      searchPlaceholder="Search in Drive..."
      onSettings={() => console.log('settings')}
      onSignOut={handleSignOut}
      onProfileClick={() => console.log('profile')}
    />
  );

  return (
    <AppShell sidebar={sidebar} topbar={topbar}>
      {children}
    </AppShell>
  );
}
