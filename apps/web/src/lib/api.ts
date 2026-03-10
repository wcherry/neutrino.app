/**
 * Typed API client for the Neutrino Rust backend.
 *
 * All responses follow camelCase JSON (Serde rename_all = "camelCase").
 * Error envelope: { error: { code: string; message: string } }
 */

const BASE_URL = process.env.NEXT_PUBLIC_API_URL ?? '';

// ---------------------------------------------------------------------------
// Common types
// ---------------------------------------------------------------------------

export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface ListQuery {
  page?: number;
  pageSize?: number;
  sortBy?: string;
  sortDir?: 'asc' | 'desc';
}

// ---------------------------------------------------------------------------
// Auth types
// ---------------------------------------------------------------------------

export interface RegisterRequest {
  email: string;
  name: string;
  password: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  tokenType: string;
  expiresIn: number;
}

export interface RefreshRequest {
  refreshToken: string;
}

export interface UserProfile {
  id: string;
  email: string;
  name: string;
  createdAt: string;
}

// ---------------------------------------------------------------------------
// Storage types
// ---------------------------------------------------------------------------

export interface FileItem {
  id: string;
  name: string;
  sizeBytes: number;
  mimeType: string;
  folderId: string | null;
  isStarred: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface FileListQuery {
  limit?: number;
  offset?: number;
  orderBy?: 'name' | 'size' | 'createdAt' | 'updatedAt';
  direction?: 'asc' | 'desc';
}

interface BackendFileListResponse {
  files: FileItem[];
  total: number;
  limit: number;
  offset: number;
}

export interface QuotaInfo {
  usedBytes: number;
  dailyUploadBytes: number;
  dailyResetAt: string;
  quotaBytes: number | null;
  dailyCapBytes: number | null;
}

// ---------------------------------------------------------------------------
// Preview types
// ---------------------------------------------------------------------------

export interface ZipEntry {
  name: string;
  size: number;
  compressedSize: number;
  isDir: boolean;
}

export interface ZipContentsResponse {
  entries: ZipEntry[];
}

// ---------------------------------------------------------------------------
// Filesystem types (future phases)
// ---------------------------------------------------------------------------

export interface Folder {
  id: string;
  userId: string;
  name: string;
  parentId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface FolderCreateRequest {
  name: string;
  parentId?: string;
}

export interface FolderUpdateRequest {
  name?: string;
  parentId?: string;
}

export interface FileUpdateRequest {
  name?: string;
  parentFolderId?: string;
}

export interface BulkMoveRequest {
  fileIds: string[];
  folderIds: string[];
  targetFolderId: string | null;
}

export interface BulkDeleteRequest {
  fileIds: string[];
  folderIds: string[];
}

export interface Shortcut {
  id: string;
  userId: string;
  name: string;
  targetId: string;
  targetType: 'file' | 'folder';
  createdAt: string;
}

export interface ShortcutCreateRequest {
  targetId: string;
  targetType: 'file' | 'folder';
  name?: string;
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

class ApiClientError extends Error {
  constructor(
    public readonly statusCode: number,
    public readonly code: string,
    message: string
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

function getAuthHeader(): Record<string, string> {
  if (typeof window === 'undefined') return {};
  const token = localStorage.getItem('access_token');
  if (!token || token === 'undefined' || token === 'null') return {};
  return { Authorization: `Bearer ${token}` };
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${BASE_URL}${path}`;

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...getAuthHeader(),
    ...(options.headers as Record<string, string> | undefined),
  };

  const res = await fetch(url, { ...options, headers });

  if (!res.ok) {
    // Clear stale tokens on 401 so the user is redirected to sign-in
    if (res.status === 401 && typeof window !== 'undefined') {
      localStorage.removeItem('access_token');
    }
    let errorBody: ApiError | null = null;
    try {
      errorBody = (await res.json()) as ApiError;
    } catch {
      // response body is not JSON
    }
    throw new ApiClientError(
      res.status,
      errorBody?.error?.code ?? 'UNKNOWN_ERROR',
      errorBody?.error?.message ?? `HTTP ${res.status}`
    );
  }

  if (res.status === 204) {
    return undefined as unknown as T;
  }

  return res.json() as Promise<T>;
}

function buildQuery(params: Record<string, string | number | boolean | undefined>): string {
  const entries = Object.entries(params).filter(
    ([, v]) => v !== undefined && v !== null && v !== ''
  );
  if (entries.length === 0) return '';
  return '?' + new URLSearchParams(entries.map(([k, v]) => [k, String(v)])).toString();
}

// ---------------------------------------------------------------------------
// Auth API
// ---------------------------------------------------------------------------

export const authApi = {
  async register(body: RegisterRequest): Promise<UserProfile> {
    return request<UserProfile>('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async login(body: LoginRequest): Promise<AuthTokens> {
    const tokens = await request<AuthTokens>('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify(body),
    });
    if (typeof window !== 'undefined') {
      localStorage.setItem('access_token', tokens.accessToken);
      localStorage.setItem('refresh_token', tokens.refreshToken);
    }
    return tokens;
  },

  async refresh(refreshToken?: string): Promise<AuthTokens> {
    const token = refreshToken ?? (typeof window !== 'undefined' ? localStorage.getItem('refresh_token') : null);
    if (!token) throw new ApiClientError(401, 'NO_REFRESH_TOKEN', 'No refresh token available');

    const tokens = await request<AuthTokens>('/api/v1/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({ refreshToken: token } satisfies RefreshRequest),
    });
    if (typeof window !== 'undefined') {
      localStorage.setItem('access_token', tokens.accessToken);
      localStorage.setItem('refresh_token', tokens.refreshToken);
    }
    return tokens;
  },

  async logout(): Promise<void> {
    try {
      await request<void>('/api/v1/auth/logout', { method: 'POST' });
    } finally {
      if (typeof window !== 'undefined') {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
      }
    }
  },

  async getProfile(): Promise<UserProfile> {
    return request<UserProfile>('/api/v1/auth/me');
  },

  isAuthenticated(): boolean {
    if (typeof window === 'undefined') return false;
    return !!localStorage.getItem('access_token');
  },
};

// ---------------------------------------------------------------------------
// Storage API
// ---------------------------------------------------------------------------

export const storageApi = {
  async uploadFile(
    file: File,
    onProgress?: (percent: number) => void
  ): Promise<FileItem> {
    return new Promise((resolve, reject) => {
      const formData = new FormData();
      formData.append('file', file);

      const xhr = new XMLHttpRequest();

      xhr.upload.addEventListener('progress', (e) => {
        if (e.lengthComputable && onProgress) {
          onProgress(Math.round((e.loaded / e.total) * 100));
        }
      });

      xhr.addEventListener('load', () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          try {
            resolve(JSON.parse(xhr.responseText) as FileItem);
          } catch {
            reject(new Error('Invalid response from server'));
          }
        } else {
          try {
            const err = JSON.parse(xhr.responseText) as ApiError;
            reject(
              new ApiClientError(xhr.status, err.error.code, err.error.message)
            );
          } catch {
            reject(new ApiClientError(xhr.status, 'UPLOAD_ERROR', `Upload failed with status ${xhr.status}`));
          }
        }
      });

      xhr.addEventListener('error', () => {
        reject(new Error('Network error during upload'));
      });

      xhr.addEventListener('abort', () => {
        reject(new Error('Upload aborted'));
      });

      const token = typeof window !== 'undefined' ? localStorage.getItem('access_token') : null;
      xhr.open('POST', `${BASE_URL}/api/v1/drive/files`);
      if (token) xhr.setRequestHeader('Authorization', `Bearer ${token}`);
      xhr.send(formData);
    });
  },

  async listFiles(
    query: FileListQuery = {}
  ): Promise<PaginatedResponse<FileItem>> {
    const { limit = 50, offset = 0, orderBy, direction } = query;
    const qs = buildQuery({ limit, offset, orderBy, direction });
    const raw = await request<BackendFileListResponse>(`/api/v1/drive/files${qs}`);
    return {
      items: raw.files,
      total: raw.total,
      page: Math.floor(raw.offset / raw.limit) + 1,
      pageSize: raw.limit,
      totalPages: Math.ceil(raw.total / raw.limit),
    };
  },

  async getFileMetadata(fileId: string): Promise<FileItem> {
    return request<FileItem>(`/api/v1/drive/files/${fileId}/metadata`);
  },

  getFileDownloadUrl(fileId: string): string {
    const token = typeof window !== 'undefined' ? localStorage.getItem('access_token') : '';
    return `${BASE_URL}/api/v1/drive/files/${fileId}?token=${token ?? ''}`;
  },

  async downloadFile(fileId: string): Promise<Blob> {
    const res = await fetch(`${BASE_URL}/api/v1/drive/files/${fileId}`, {
      headers: getAuthHeader(),
    });
    if (!res.ok) throw new ApiClientError(res.status, 'DOWNLOAD_ERROR', `Download failed`);
    return res.blob();
  },

  async getQuota(): Promise<QuotaInfo> {
    return request<QuotaInfo>('/api/v1/drive/quota');
  },

  async deleteFile(fileId: string): Promise<void> {
    return request<void>(`/api/v1/drive/files/${fileId}`, { method: 'DELETE' });
  },

  /** Fetch file content as a Blob URL for in-browser preview. Caller must call URL.revokeObjectURL when done. */
  async fetchPreviewBlobUrl(fileId: string): Promise<string> {
    const res = await fetch(`${BASE_URL}/api/v1/drive/files/${fileId}/preview`, {
      headers: getAuthHeader(),
    });
    if (!res.ok) throw new ApiClientError(res.status, 'PREVIEW_ERROR', `Preview failed`);
    const blob = await res.blob();
    return URL.createObjectURL(blob);
  },

  /** Fetch text content of a file for preview (text/code files). */
  async fetchPreviewText(fileId: string): Promise<string> {
    const res = await fetch(`${BASE_URL}/api/v1/drive/files/${fileId}/preview`, {
      headers: getAuthHeader(),
    });
    if (!res.ok) throw new ApiClientError(res.status, 'PREVIEW_ERROR', `Preview failed`);
    return res.text();
  },

  async getZipContents(fileId: string): Promise<ZipContentsResponse> {
    return request<ZipContentsResponse>(`/api/v1/drive/files/${fileId}/zip-contents`);
  },
};

// ---------------------------------------------------------------------------
// Filesystem API (Phase 1.2+)
// ---------------------------------------------------------------------------

export const filesystemApi = {
  // Folders
  async createFolder(body: FolderCreateRequest): Promise<Folder> {
    return request<Folder>('/api/v1/fs/folders', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async listFolders(parentId?: string): Promise<Folder[]> {
    const qs = buildQuery({ parent_id: parentId });
    return request<Folder[]>(`/api/v1/fs/folders${qs}`);
  },

  async getFolder(folderId: string): Promise<Folder> {
    return request<Folder>(`/api/v1/fs/folders/${folderId}`);
  },

  async updateFolder(folderId: string, body: FolderUpdateRequest): Promise<Folder> {
    return request<Folder>(`/api/v1/fs/folders/${folderId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },

  async deleteFolder(folderId: string): Promise<void> {
    return request<void>(`/api/v1/fs/folders/${folderId}`, { method: 'DELETE' });
  },

  // File management
  async updateFile(fileId: string, body: FileUpdateRequest): Promise<FileItem> {
    return request<FileItem>(`/api/v1/fs/files/${fileId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },

  // Trash
  async trashItem(itemId: string, itemType: 'file' | 'folder'): Promise<void> {
    return request<void>(`/api/v1/fs/trash`, {
      method: 'POST',
      body: JSON.stringify({ id: itemId, type: itemType }),
    });
  },

  async restoreItem(itemId: string, itemType: 'file' | 'folder'): Promise<void> {
    return request<void>(`/api/v1/fs/trash/${itemId}/restore`, {
      method: 'POST',
      body: JSON.stringify({ type: itemType }),
    });
  },

  async listTrash(): Promise<(FileItem | Folder)[]> {
    return request<(FileItem | Folder)[]>('/api/v1/fs/trash');
  },

  async emptyTrash(): Promise<void> {
    return request<void>('/api/v1/fs/trash', { method: 'DELETE' });
  },

  // Bulk operations
  async bulkMove(body: BulkMoveRequest): Promise<void> {
    return request<void>('/api/v1/fs/bulk/move', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async bulkDelete(body: BulkDeleteRequest): Promise<void> {
    return request<void>('/api/v1/fs/bulk/delete', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  // Shortcuts
  async createShortcut(body: ShortcutCreateRequest): Promise<Shortcut> {
    return request<Shortcut>('/api/v1/fs/shortcuts', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async listShortcuts(): Promise<Shortcut[]> {
    return request<Shortcut[]>('/api/v1/fs/shortcuts');
  },

  async deleteShortcut(shortcutId: string): Promise<void> {
    return request<void>(`/api/v1/fs/shortcuts/${shortcutId}`, { method: 'DELETE' });
  },
};

// ---------------------------------------------------------------------------
// Convenience re-exports
// ---------------------------------------------------------------------------

export { ApiClientError };
