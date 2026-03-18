'use client';

import React from 'react';
import {
  X,
  Image as ImageIcon,
  Video,
  Calendar,
  HardDrive,
  Tag,
  Aperture,
  Camera,
  MapPin,
  Sun,
} from 'lucide-react';
import { Text, Heading } from '@neutrino/ui';
import { type PhotoResponse } from '@/lib/api';
import { LocationMap } from './LocationMap';
import styles from './PhotoInfoPanel.module.css';

function formatFileSize(bytes: number): string {
  if (bytes <= 0) return '—';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

function formatFNumber(f: number): string {
  return `f/${f % 1 === 0 ? f.toFixed(0) : f.toFixed(1)}`;
}

function formatFocalLength(mm: number): string {
  return `${mm % 1 === 0 ? mm.toFixed(0) : mm.toFixed(1)} mm`;
}

interface Props {
  photo: PhotoResponse;
  onClose: () => void;
}

export function PhotoInfoPanel({ photo, onClose }: Props) {
  const meta = photo.metadata;
  const exif = meta?.exif;

  const isVideo = photo.mimeType.startsWith('video/');
  const ext = photo.fileName.includes('.')
    ? photo.fileName.split('.').pop()!.toUpperCase()
    : 'Unknown';

  return (
    <aside className={styles.panel} aria-label="Photo information">
      <div className={styles.header}>
        <Heading level={3} size="sm">Photo info</Heading>
        <button
          type="button"
          className={styles.closeBtn}
          onClick={onClose}
          aria-label="Close photo info"
        >
          <X size={16} />
        </button>
      </div>

      <div className={styles.preview}>
        {photo.thumbnail && photo.thumbnailMimeType ? (
          <img
            src={`data:${photo.thumbnailMimeType};base64,${photo.thumbnail}`}
            alt={photo.fileName}
            className={styles.previewImg}
          />
        ) : (
          <div className={styles.previewPlaceholder}>
            {isVideo ? <Video size={40} strokeWidth={1} /> : <ImageIcon size={40} strokeWidth={1} />}
          </div>
        )}
      </div>

      <Text weight="medium" size="sm" truncate>
        {photo.fileName}
      </Text>

      {/* Details */}
      <div className={styles.section}>
        <Text size="xs" color="muted" weight="semibold">Details</Text>
        <dl className={styles.list}>
          {meta?.width && meta?.height && (
            <div className={styles.row}>
              <dt><ImageIcon size={13} />Dimensions</dt>
              <dd>{meta.width} × {meta.height}</dd>
            </div>
          )}
          <div className={styles.row}>
            <dt><HardDrive size={13} />Size</dt>
            <dd>{formatFileSize(photo.sizeBytes)}</dd>
          </div>
          <div className={styles.row}>
            <dt><Tag size={13} />Type</dt>
            <dd>{ext}</dd>
          </div>
          {photo.captureDate && (
            <div className={styles.row}>
              <dt><Calendar size={13} />Captured</dt>
              <dd>{formatDate(photo.captureDate)}</dd>
            </div>
          )}
          <div className={styles.row}>
            <dt><Calendar size={13} />Added</dt>
            <dd>{formatDate(photo.createdAt)}</dd>
          </div>
        </dl>
      </div>

      {/* Camera / EXIF */}
      {exif && (exif.make || exif.model || exif.exposureTime || exif.fNumber || exif.iso || exif.focalLength) && (
        <div className={styles.section}>
          <Text size="xs" color="muted" weight="semibold">Camera</Text>
          <dl className={styles.list}>
            {(exif.make || exif.model) && (
              <div className={styles.row}>
                <dt><Camera size={13} />Camera</dt>
                <dd>{[exif.make, exif.model].filter(Boolean).join(' ')}</dd>
              </div>
            )}
            {exif.exposureTime && (
              <div className={styles.row}>
                <dt><Sun size={13} />Exposure</dt>
                <dd>{exif.exposureTime} s</dd>
              </div>
            )}
            {exif.fNumber != null && (
              <div className={styles.row}>
                <dt><Aperture size={13} />Aperture</dt>
                <dd>{formatFNumber(exif.fNumber)}</dd>
              </div>
            )}
            {exif.iso != null && (
              <div className={styles.row}>
                <dt><Sun size={13} />ISO</dt>
                <dd>{exif.iso}</dd>
              </div>
            )}
            {exif.focalLength != null && (
              <div className={styles.row}>
                <dt><Camera size={13} />Focal length</dt>
                <dd>{formatFocalLength(exif.focalLength)}</dd>
              </div>
            )}
          </dl>
        </div>
      )}

      {/* Location */}
      {exif?.gpsLatitude != null && exif?.gpsLongitude != null && (
        <div className={styles.section}>
          <Text size="xs" color="muted" weight="semibold">Location</Text>
          <LocationMap lat={exif.gpsLatitude} lng={exif.gpsLongitude} />
          <dl className={styles.list}>
            <div className={styles.row}>
              <dt><MapPin size={13} />Coordinates</dt>
              <dd>
                {exif.gpsLatitude.toFixed(5)}, {exif.gpsLongitude.toFixed(5)}
              </dd>
            </div>
          </dl>
        </div>
      )}

      {/* MIME type */}
      <div className={styles.section}>
        <Text size="xs" color="muted" weight="semibold">MIME type</Text>
        <Text size="xs" color="muted">{photo.mimeType}</Text>
      </div>
    </aside>
  );
}
