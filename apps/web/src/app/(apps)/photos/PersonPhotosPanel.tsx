'use client';

import React, { useState, useRef, useEffect } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { X, Users, Image as ImageIcon, Pencil, Check, GitMerge, Trash2 } from 'lucide-react';
import { personsApi, type PersonResponse } from '@/lib/api';
import styles from './PersonPhotosPanel.module.css';

interface Props {
  person: PersonResponse;
  allPersons: PersonResponse[];
  onClose: () => void;
  onPersonUpdated: (updated: PersonResponse) => void;
  onPersonDeleted: () => void;
}

export function PersonPhotosPanel({ person, allPersons, onClose, onPersonUpdated, onPersonDeleted }: Props) {
  const queryClient = useQueryClient();

  // ── Rename state ──────────────────────────────────────────────────────────
  const [isRenaming, setIsRenaming] = useState(false);
  const [nameInput, setNameInput] = useState(person.name ?? '');
  const nameInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isRenaming) nameInputRef.current?.focus();
  }, [isRenaming]);

  useEffect(() => {
    setNameInput(person.name ?? '');
  }, [person.name]);

  const renameMutation = useMutation({
    mutationFn: (name: string) => personsApi.renamePerson(person.id, name),
    onSuccess: (updated) => {
      setIsRenaming(false);
      onPersonUpdated(updated);
    },
  });

  function commitRename() {
    const trimmed = nameInput.trim();
    if (trimmed && trimmed !== (person.name ?? '')) {
      renameMutation.mutate(trimmed);
    } else {
      setIsRenaming(false);
    }
  }

  // ── Merge state ───────────────────────────────────────────────────────────
  const [showMerge, setShowMerge] = useState(false);
  const mergeTargets = allPersons.filter((p) => p.id !== person.id);

  const mergeMutation = useMutation({
    mutationFn: (sourceId: string) => personsApi.mergePersons(person.id, sourceId),
    onSuccess: (updated) => {
      setShowMerge(false);
      onPersonUpdated(updated);
      queryClient.invalidateQueries({ queryKey: ['personPhotos', person.id] });
    },
  });

  // ── Remove face ───────────────────────────────────────────────────────────
  const removeFaceMutation = useMutation({
    mutationFn: (faceId: string) => personsApi.removeFace(person.id, faceId),
    onSuccess: () => {
      // If last face removed, the person is deleted on the backend.
      if (person.faceCount <= 1) {
        onPersonDeleted();
      } else {
        const updated: PersonResponse = {
          ...person,
          faceCount: person.faceCount - 1,
          faces: person.faces.filter((f) => f.id !== removeFaceMutation.variables),
        };
        onPersonUpdated(updated);
      }
      queryClient.invalidateQueries({ queryKey: ['personPhotos', person.id] });
    },
  });

  // ── Photos query ──────────────────────────────────────────────────────────
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
      {/* Header */}
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
            {isRenaming ? (
              <div className={styles.renameRow}>
                <input
                  ref={nameInputRef}
                  className={styles.renameInput}
                  value={nameInput}
                  onChange={(e) => setNameInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') commitRename();
                    if (e.key === 'Escape') setIsRenaming(false);
                  }}
                  placeholder="Enter a name"
                  maxLength={80}
                />
                <button
                  className={styles.iconAction}
                  onClick={commitRename}
                  disabled={renameMutation.isPending}
                  title="Save name"
                >
                  <Check size={13} />
                </button>
              </div>
            ) : (
              <div className={styles.renameRow}>
                <span className={styles.headerTitle}>
                  {person.name ?? 'Unknown person'}
                </span>
                <button
                  className={styles.iconAction}
                  onClick={() => setIsRenaming(true)}
                  title="Rename"
                >
                  <Pencil size={12} />
                </button>
              </div>
            )}
            <span className={styles.headerCount}>
              {person.faceCount} photo{person.faceCount !== 1 ? 's' : ''}
            </span>
          </div>
        </div>
        <div className={styles.headerActions}>
          {mergeTargets.length > 0 && (
            <div className={styles.mergeWrapper}>
              <button
                className={styles.iconAction}
                onClick={() => setShowMerge((v) => !v)}
                title="Merge with another person"
              >
                <GitMerge size={14} />
              </button>
              {showMerge && (
                <div className={styles.mergeDropdown}>
                  <p className={styles.mergeLabel}>Merge with:</p>
                  {mergeTargets.map((target) => (
                    <button
                      key={target.id}
                      className={styles.mergeOption}
                      disabled={mergeMutation.isPending}
                      onClick={() => mergeMutation.mutate(target.id)}
                    >
                      {target.name ?? 'Unknown person'}
                      <span className={styles.mergeOptionCount}>({target.faceCount})</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close">
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Faces strip */}
      {person.faces.length > 0 && (
        <div className={styles.facesSection}>
          <p className={styles.sectionLabel}>Faces in this cluster</p>
          <div className={styles.facesStrip}>
            {person.faces.map((face) => {
              const src =
                face.thumbnail && face.thumbnailMimeType
                  ? `data:${face.thumbnailMimeType};base64,${face.thumbnail}`
                  : null;
              return (
                <div key={face.id} className={styles.faceItem}>
                  {src ? (
                    <img src={src} alt="" className={styles.faceThumb} />
                  ) : (
                    <div className={styles.faceThumbPlaceholder}>
                      <Users size={14} />
                    </div>
                  )}
                  <button
                    className={styles.faceRemoveBtn}
                    onClick={() => removeFaceMutation.mutate(face.id)}
                    disabled={removeFaceMutation.isPending}
                    title="Remove this face from cluster"
                  >
                    <Trash2 size={10} />
                  </button>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Photos grid */}
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
