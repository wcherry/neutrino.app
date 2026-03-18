'use client';

import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { X, Users, Image as ImageIcon } from 'lucide-react';
import { personsApi, type PersonResponse } from '@/lib/api';
import styles from './PersonPhotosPanel.module.css';

interface Props {
  person: PersonResponse;
  onClose: () => void;
}

export function PersonPhotosPanel({ person, onClose }: Props) {
  const { data, isLoading } = useQuery({
    queryKey: ['personPhotos', person.id],
    queryFn: () => personsApi.listPersonPhotos(person.id),
    staleTime: 60_000,
  });

  const photos = data?.photos ?? [];

  const avatarSrc =
    person.coverThumbnail && person.coverThumbnailMimeType
      ? `data:${person.coverThumbnailMimeType};base64,${person.coverThumbnail}`
      : null;

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <div className={styles.headerInfo}>
          <div className={styles.avatar}>
            {avatarSrc ? (
              <img src={avatarSrc} alt="Person" className={styles.avatarImg} />
            ) : (
              <div className={styles.avatarPlaceholder}>
                <Users size={18} />
              </div>
            )}
          </div>
          <div className={styles.headerText}>
            <span className={styles.headerTitle}>Person</span>
            <span className={styles.headerCount}>{person.faceCount} photo{person.faceCount !== 1 ? 's' : ''}</span>
          </div>
        </div>
        <button className={styles.closeBtn} onClick={onClose} aria-label="Close person panel">
          <X size={14} />
        </button>
      </div>

      <div className={styles.body}>
        {isLoading ? (
          <p className={styles.hint}>Loading photos…</p>
        ) : photos.length === 0 ? (
          <p className={styles.hint}>No photos found</p>
        ) : (
          <div className={styles.grid}>
            {photos.map((photo) => {
              const src =
                photo.thumbnail && photo.thumbnailMimeType
                  ? `data:${photo.thumbnailMimeType};base64,${photo.thumbnail}`
                  : null;
              return (
                <div key={photo.id} className={styles.thumb}>
                  {src ? (
                    <img src={src} alt={photo.fileName} className={styles.thumbImg} />
                  ) : (
                    <div className={styles.thumbPlaceholder}>
                      <ImageIcon size={18} />
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
