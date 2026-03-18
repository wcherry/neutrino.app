'use client';

import React, { useRef, useState } from 'react';
import { Cloud, Upload } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import styles from './Sidebar.module.css';

export interface NavItem {
  id: string;
  label: string;
  icon: LucideIcon;
  href?: string;
  onClick?: () => void;
  active?: boolean;
  badge?: number | string;
}

export interface NavSection {
  id: string;
  label?: string;
  items: NavItem[];
}

export interface StorageQuota {
  usedBytes: number;
  totalBytes: number;
}

export interface SidebarProps {
  logoText?: string;
  logoHref?: string;
  sections?: NavSection[];
  quota?: StorageQuota;
  onUpload?: (files: FileList) => void;
  className?: string;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function Sidebar({
  logoText = 'Neutrino',
  logoHref = '/',
  sections = [],
  quota,
  onUpload,
  className = '',
}: SidebarProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);
  const quotaPercent =
    quota && quota.totalBytes > 0
      ? Math.min(100, (quota.usedBytes / quota.totalBytes) * 100)
      : 0;

  const quotaBarClass =
    quotaPercent >= 90
      ? styles.danger
      : quotaPercent >= 75
      ? styles.warning
      : '';

  return (
    <aside
      className={[styles.sidebar, className].filter(Boolean).join(' ')}
      aria-label="Application navigation"
    >
      {/* Logo */}
      <a className={styles.logo} href={logoHref} aria-label={`${logoText} home`}>
        <span className={styles['logo-icon']} aria-hidden="true">
          <Cloud size={20} />
        </span>
        <span className={styles['logo-text']}>{logoText}</span>
      </a>

      {/* Upload */}
      {onUpload && (
        <div className={styles['upload-area']}>
          <button
            type="button"
            className={[styles['upload-btn'], dragOver ? styles['drag-over'] : ''].filter(Boolean).join(' ')}
            onClick={() => fileInputRef.current?.click()}
            onDragEnter={(e) => { e.preventDefault(); setDragOver(true); }}
            onDragOver={(e) => e.preventDefault()}
            onDragLeave={() => setDragOver(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragOver(false);
              if (e.dataTransfer.files.length > 0) onUpload(e.dataTransfer.files);
            }}
          >
            <Upload size={16} aria-hidden="true" />
            Upload files
          </button>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            style={{ display: 'none' }}
            onChange={(e) => {
              if (e.target.files && e.target.files.length > 0) {
                onUpload(e.target.files);
                e.target.value = '';
              }
            }}
            tabIndex={-1}
            aria-hidden="true"
          />
        </div>
      )}

      {/* Navigation */}
      <nav className={styles.nav} aria-label="Primary navigation">
        {sections.map((section) => (
          <div key={section.id} className={styles['nav-section']}>
            {section.label && (
              <p className={styles['nav-section-label']} aria-hidden="true">
                {section.label}
              </p>
            )}
            {section.items.map((item) => {
              const IconComponent = item.icon;
              const classes = [styles['nav-item'], item.active ? styles.active : '']
                .filter(Boolean)
                .join(' ');

              if (item.href) {
                return (
                  <a
                    key={item.id}
                    href={item.href}
                    className={classes}
                    aria-current={item.active ? 'page' : undefined}
                  >
                    <span className={styles['nav-icon']} aria-hidden="true">
                      <IconComponent size={18} strokeWidth={1.75} />
                    </span>
                    <span className={styles['nav-label']}>{item.label}</span>
                    {item.badge !== undefined && (
                      <span className={styles['nav-badge']} aria-label={`${item.badge} items`}>
                        {item.badge}
                      </span>
                    )}
                  </a>
                );
              }

              return (
                <button
                  key={item.id}
                  type="button"
                  className={classes}
                  onClick={item.onClick}
                  aria-current={item.active ? 'page' : undefined}
                >
                  <span className={styles['nav-icon']} aria-hidden="true">
                    <IconComponent size={18} strokeWidth={1.75} />
                  </span>
                  <span className={styles['nav-label']}>{item.label}</span>
                  {item.badge !== undefined && (
                    <span className={styles['nav-badge']} aria-label={`${item.badge} items`}>
                      {item.badge}
                    </span>
                  )}
                </button>
              );
            })}
          </div>
        ))}
      </nav>

      {/* Storage quota */}
      {quota && (
        <div className={styles.quota}>
          <div className={styles['quota-header']}>
            <span className={styles['quota-label']}>Storage</span>
            <span className={styles['quota-value']}>{Math.round(quotaPercent)}%</span>
          </div>
          <div
            className={styles['quota-track']}
            role="progressbar"
            aria-valuenow={Math.round(quotaPercent)}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label="Storage usage"
          >
            <div
              className={[styles['quota-bar'], quotaBarClass].filter(Boolean).join(' ')}
              style={{ width: `${quotaPercent}%` }}
            />
          </div>
          <p className={styles['quota-sub']}>
            {formatBytes(quota.usedBytes)} of {formatBytes(quota.totalBytes)} used
            <a href="/settings/storage" className={styles['quota-link']}>
              Manage
            </a>
          </p>
        </div>
      )}
    </aside>
  );
}
