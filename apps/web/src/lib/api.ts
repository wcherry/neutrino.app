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
  coverThumbnail: string | null;
  coverThumbnailMimeType: string | null;
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
// Version types
// ---------------------------------------------------------------------------

export interface FileVersionItem {
  id: string;
  fileId: string;
  versionNumber: number;
  sizeBytes: number;
  label: string | null;
  createdAt: string;
}

export interface ListVersionsResponse {
  versions: FileVersionItem[];
  total: number;
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
  name: string;
  parentId: string | null;
  color: string | null;
  isStarred: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface FolderContentsResponse {
  /** Present when listing a non-root folder */
  folder: Folder | null;
  folders: Folder[];
  files: FileItem[];
  shortcuts: unknown[];
}

export interface StarredContentsResponse {
  files: FileItem[];
  folders: Folder[];
}

export interface FolderCreateRequest {
  name: string;
  parentId?: string;
}

export interface FolderUpdateRequest {
  name?: string;
  parentId?: string;
  isStarred?: boolean;
}

export interface FileUpdateRequest {
  name?: string;
  /** Move to folder (null = move to root) */
  folderId?: string | null;
  isStarred?: boolean;
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

const AUTH_LOGIN_PATH = '/api/v1/auth/login';
const AUTH_REGISTER_PATH = '/api/v1/auth/register';
const AUTH_REFRESH_PATH = '/api/v1/auth/refresh';
const LOGIN_REDIRECT_PATH = '/sign-in';
let refreshInFlight: Promise<AuthTokens | null> | null = null;

function getAuthHeader(): Record<string, string> {
  if (typeof window === 'undefined') return {};
  const token = localStorage.getItem('access_token');
  if (!token || token === 'undefined' || token === 'null') return {};
  return { Authorization: `Bearer ${token}` };
}

function shouldSkipRefresh(path: string): boolean {
  return (
    path.startsWith(AUTH_LOGIN_PATH) ||
    path.startsWith(AUTH_REGISTER_PATH) ||
    path.startsWith(AUTH_REFRESH_PATH)
  );
}

function clearAuthAndRedirect(): void {
  if (typeof window === 'undefined') return;
  localStorage.removeItem('access_token');
  localStorage.removeItem('refresh_token');
  if (window.location.pathname !== LOGIN_REDIRECT_PATH) {
    window.location.assign(LOGIN_REDIRECT_PATH);
  }
}

async function refreshTokens(refreshToken?: string): Promise<AuthTokens | null> {
  if (typeof window === 'undefined') return null;
  const token = refreshToken ?? localStorage.getItem('refresh_token');
  if (!token || token === 'undefined' || token === 'null') return null;

  try {
    const res = await fetch(`${BASE_URL}${AUTH_REFRESH_PATH}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refreshToken: token } satisfies RefreshRequest),
    });

    if (!res.ok) return null;
    const tokens = (await res.json()) as AuthTokens;
    localStorage.setItem('access_token', tokens.accessToken);
    localStorage.setItem('refresh_token', tokens.refreshToken);
    return tokens;
  } catch {
    return null;
  }
}

async function refreshTokensOnce(): Promise<AuthTokens | null> {
  if (refreshInFlight) return refreshInFlight;
  refreshInFlight = (async () => {
    try {
      return await refreshTokens();
    } finally {
      refreshInFlight = null;
    }
  })();
  return refreshInFlight;
}

type RequestConfig = {
  retry?: boolean;
  auth?: 'auto' | 'none';
  responseType?: 'json' | 'blob' | 'text' | 'none';
  onUploadProgress?: (percent: number) => void;
};

function requestWithXhr<T>(path: string, options: RequestInit, config: RequestConfig): Promise<T> {
  return new Promise((resolve, reject) => {
    const url = `${BASE_URL}${path}`;
    const includeAuth = config.auth !== 'none';
    const xhr = new XMLHttpRequest();

    if (config.responseType === 'blob') xhr.responseType = 'blob';

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable && config.onUploadProgress) {
        config.onUploadProgress(Math.round((e.loaded / e.total) * 100));
      }
    });

    xhr.addEventListener('load', () => {
      if (xhr.status === 401 && includeAuth && !config.retry && !shouldSkipRefresh(path)) {
        refreshTokensOnce().then((refreshed) => {
          if (refreshed) {
            requestWithXhr<T>(path, options, { ...config, retry: true }).then(resolve, reject);
          } else {
            clearAuthAndRedirect();
            reject(new ApiClientError(401, 'UNAUTHENTICATED', 'Session expired'));
          }
        }).catch(reject);
        return;
      }

      if (xhr.status >= 200 && xhr.status < 300) {
        if (xhr.status === 204 || config.responseType === 'none') {
          resolve(undefined as unknown as T);
        } else if (config.responseType === 'blob') {
          resolve(xhr.response as T);
        } else if (config.responseType === 'text') {
          resolve(xhr.responseText as unknown as T);
        } else {
          try {
            resolve(JSON.parse(xhr.responseText) as T);
          } catch {
            reject(new Error('Invalid response from server'));
          }
        }
      } else {
        try {
          const err = JSON.parse(xhr.responseText) as ApiError;
          reject(new ApiClientError(xhr.status, err.error.code, err.error.message));
        } catch {
          reject(new ApiClientError(xhr.status, 'UNKNOWN_ERROR', `HTTP ${xhr.status}`));
        }
      }
    });

    xhr.addEventListener('error', () => reject(new Error('Network error')));
    xhr.addEventListener('abort', () => reject(new Error('Request aborted')));

    xhr.open(options.method ?? 'GET', url);

    if (includeAuth) {
      const authHeaders = getAuthHeader();
      Object.entries(authHeaders).forEach(([k, v]) => xhr.setRequestHeader(k, v));
    }
    if (!(options.body instanceof FormData)) {
      xhr.setRequestHeader('Content-Type', 'application/json');
    }

    xhr.send(options.body as XMLHttpRequestBodyInit | null);
  });
}

async function request<T>(
  path: string,
  options: RequestInit = {},
  config: RequestConfig = {}
): Promise<T> {
  if (config.onUploadProgress) {
    return requestWithXhr<T>(path, options, config);
  }

  const url = `${BASE_URL}${path}`;
  const includeAuth = config.auth !== 'none';
  const isFormData = options.body instanceof FormData;

  const headers: Record<string, string> = {
    ...(isFormData ? {} : { 'Content-Type': 'application/json' }),
    ...(options.headers as Record<string, string> | undefined),
    ...(includeAuth ? getAuthHeader() : {}),
  };

  const res = await fetch(url, { ...options, headers });

  if (res.status === 401 && includeAuth && !config.retry && !shouldSkipRefresh(path)) {
    const refreshed = await refreshTokensOnce();
    if (refreshed) {
      return request<T>(path, options, { ...config, retry: true });
    }
    clearAuthAndRedirect();
    throw new ApiClientError(401, 'UNAUTHENTICATED', 'Session expired');
  }

  if (!res.ok) {
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

  if (res.status === 204 || config.responseType === 'none') {
    return undefined as unknown as T;
  }

  if (config.responseType === 'blob') {
    return res.blob() as unknown as Promise<T>;
  }

  if (config.responseType === 'text') {
    return res.text() as unknown as Promise<T>;
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

function normalizeShareLinkVisibility(
  visibility: string | null | undefined
): ShareLink['visibility'] {
  if (visibility === 'public') return 'public';
  if (visibility === 'anyoneWithLink') return 'anyoneWithLink';
  if (visibility === 'anyone_with_link') return 'anyoneWithLink';
  return 'anyoneWithLink';
}

function normalizeShareLink(link: ShareLink): ShareLink {
  return { ...link, visibility: normalizeShareLinkVisibility((link as ShareLink).visibility) };
}

function normalizeResolvedShareLink(link: ResolvedShareLink): ResolvedShareLink {
  return { ...link, visibility: normalizeShareLinkVisibility(link.visibility) };
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
    const token =
      refreshToken ??
      (typeof window !== 'undefined' ? localStorage.getItem('refresh_token') : null);
    if (!token || token === 'undefined' || token === 'null') {
      throw new ApiClientError(401, 'NO_REFRESH_TOKEN', 'No refresh token available');
    }
    const tokens = await refreshTokens(token);
    if (!tokens) {
      throw new ApiClientError(401, 'REFRESH_FAILED', 'Unable to refresh session');
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
    onProgress?: (percent: number) => void,
    folderId?: string | null,
  ): Promise<FileItem> {
    const formData = new FormData();
    if (folderId) formData.append('folder_id', folderId);
    formData.append('file', file);
    return request<FileItem>('/api/v1/drive/files/upload', {
      method: 'POST',
      body: formData,
    }, { onUploadProgress: onProgress });
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
    return request<Blob>(`/api/v1/drive/files/${fileId}`, {}, { responseType: 'blob' });
  },

  async getQuota(): Promise<QuotaInfo> {
    return request<QuotaInfo>('/api/v1/drive/quota');
  },

  async deleteFile(fileId: string): Promise<void> {
    return request<void>(`/api/v1/drive/files/${fileId}`, { method: 'DELETE' });
  },

  /** Fetch file content as a Blob URL for in-browser preview. Caller must call URL.revokeObjectURL when done. */
  async fetchPreviewBlobUrl(fileId: string): Promise<string> {
    const blob = await request<Blob>(`/api/v1/drive/files/${fileId}/preview`, {}, { responseType: 'blob' });
    return URL.createObjectURL(blob);
  },

  /** Fetch text content of a file for preview (text/code files). */
  async fetchPreviewText(fileId: string): Promise<string> {
    return request<string>(`/api/v1/drive/files/${fileId}/preview`, {}, { responseType: 'text' });
  },

  async getZipContents(fileId: string): Promise<ZipContentsResponse> {
    return request<ZipContentsResponse>(`/api/v1/drive/files/${fileId}/zip-contents`);
  },

  async listVersions(fileId: string): Promise<ListVersionsResponse> {
    return request<ListVersionsResponse>(`/api/v1/drive/files/${fileId}/versions`);
  },

  async labelVersion(fileId: string, versionId: string, label: string): Promise<FileVersionItem> {
    return request<FileVersionItem>(`/api/v1/drive/files/${fileId}/versions/${versionId}`, {
      method: 'PATCH',
      body: JSON.stringify({ label }),
    });
  },

  async restoreVersion(fileId: string, versionId: string): Promise<void> {
    return request<void>(`/api/v1/drive/files/${fileId}/versions/${versionId}/restore`, {
      method: 'POST',
    });
  },
};

// ---------------------------------------------------------------------------
// Filesystem API (Phase 1.2+)
// ---------------------------------------------------------------------------

export const filesystemApi = {
  // Folder contents (primary navigation)
  async getStarred(limit = 5): Promise<StarredContentsResponse> {
    return request<StarredContentsResponse>(`/api/v1/drive/starred?limit=${limit}`);
  },

  async getRootContents(query: FileListQuery = {}): Promise<FolderContentsResponse> {
    const { limit = 200, offset = 0, orderBy, direction } = query;
    const qs = buildQuery({ limit, offset, orderBy, direction });
    return request<FolderContentsResponse>(`/api/v1/drive${qs}`);
  },

  async getFolderContents(folderId: string, query: FileListQuery = {}): Promise<FolderContentsResponse> {
    const { limit = 200, offset = 0, orderBy, direction } = query;
    const qs = buildQuery({ limit, offset, orderBy, direction });
    return request<FolderContentsResponse>(`/api/v1/drive/folders/${folderId}${qs}`);
  },

  // Folders
  async createFolder(body: FolderCreateRequest): Promise<Folder> {
    return request<Folder>('/api/v1/drive/folders', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async updateFolder(folderId: string, body: FolderUpdateRequest): Promise<Folder> {
    return request<Folder>(`/api/v1/drive/folders/${folderId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },

  async deleteFolder(folderId: string): Promise<void> {
    return request<void>(`/api/v1/drive/folders/${folderId}`, { method: 'DELETE' });
  },

  // File management
  async updateFile(fileId: string, body: FileUpdateRequest): Promise<FileItem> {
    return request<FileItem>(`/api/v1/drive/files/${fileId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },

  // Bulk operations
  async bulkMove(body: BulkMoveRequest): Promise<void> {
    return request<void>('/api/v1/drive/bulk/move', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async bulkDelete(body: BulkDeleteRequest): Promise<void> {
    return request<void>('/api/v1/drive/bulk/trash', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  // Trash
  async listTrash(): Promise<{ files: unknown[]; folders: unknown[] }> {
    return request<{ files: unknown[]; folders: unknown[] }>('/api/v1/drive/trash');
  },

  async emptyTrash(): Promise<void> {
    return request<void>('/api/v1/drive/trash', { method: 'DELETE' });
  },

  // Shortcuts
  async createShortcut(body: ShortcutCreateRequest): Promise<Shortcut> {
    return request<Shortcut>('/api/v1/drive/shortcuts', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async deleteShortcut(shortcutId: string): Promise<void> {
    return request<void>(`/api/v1/drive/shortcuts/${shortcutId}`, { method: 'DELETE' });
  },
};

// ---------------------------------------------------------------------------
// Permissions types
// ---------------------------------------------------------------------------

export type PermissionRole = 'owner' | 'editor' | 'commenter' | 'viewer';
export type ResourceType = 'file' | 'folder';

export interface Permission {
  id: string;
  resourceType: string;
  resourceId: string;
  userId: string;
  userEmail: string;
  userName: string;
  role: string;
  grantedBy: string;
  createdAt: string;
}

export interface ListPermissionsResponse {
  permissions: Permission[];
}

export interface GrantPermissionRequest {
  userId: string;
  userEmail: string;
  userName: string;
  role: PermissionRole;
}

export interface UpdatePermissionRequest {
  role: PermissionRole;
}

// ---------------------------------------------------------------------------
// Sharing (share link) types
// ---------------------------------------------------------------------------

export interface ShareLink {
  id: string;
  resourceType: string;
  resourceId: string;
  token: string;
  visibility: 'public' | 'anyoneWithLink';
  role: 'viewer' | 'commenter' | 'editor';
  expiresAt: string | null;
  isActive: boolean;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface UpsertShareLinkRequest {
  visibility?: 'public' | 'anyoneWithLink';
  role?: 'viewer' | 'commenter' | 'editor';
  expiresAt?: string | null;
}

export interface UpdateShareLinkRequest {
  visibility?: 'public' | 'anyoneWithLink';
  role?: 'viewer' | 'commenter' | 'editor';
  expiresAt?: string | null;
  isActive?: boolean;
}

export interface ResolvedShareLink {
  resourceType: string;
  resourceId: string;
  role: string;
  visibility: 'public' | 'anyoneWithLink';
  expiresAt: string | null;
  resourceName: string;
}

// ---------------------------------------------------------------------------
// User lookup types
// ---------------------------------------------------------------------------

export interface UserLookup {
  id: string;
  email: string;
  name: string;
}

// ---------------------------------------------------------------------------
// Access request types
// ---------------------------------------------------------------------------

export interface AccessRequest {
  id: string;
  resourceType: string;
  resourceId: string;
  requesterId: string;
  requesterEmail: string;
  requesterName: string;
  message: string | null;
  requestedRole: string;
  status: 'pending' | 'approved' | 'denied';
  createdAt: string;
  updatedAt: string;
}

export interface ListAccessRequestsResponse {
  requests: AccessRequest[];
}

export interface CreateAccessRequestRequest {
  message?: string;
  requestedRole?: string;
  requesterName: string;
}

export interface ApproveAccessRequestRequest {
  role?: string;
  requesterEmail: string;
  requesterName: string;
}

// ---------------------------------------------------------------------------
// Shared with me types
// ---------------------------------------------------------------------------

export interface SharedWithMeResponse {
  files: FileItem[];
  folders: Folder[];
}

// ---------------------------------------------------------------------------
// Permissions API
// ---------------------------------------------------------------------------

export const permissionsApi = {
  async listPermissions(resourceType: ResourceType, resourceId: string): Promise<ListPermissionsResponse> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/permissions`
      : `/api/v1/drive/folders/${resourceId}/permissions`;
    return request<ListPermissionsResponse>(path);
  },

  async grantPermission(resourceType: ResourceType, resourceId: string, body: GrantPermissionRequest): Promise<Permission> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/permissions`
      : `/api/v1/drive/folders/${resourceId}/permissions`;
    return request<Permission>(path, { method: 'POST', body: JSON.stringify(body) });
  },

  async updatePermission(resourceType: ResourceType, resourceId: string, userId: string, body: UpdatePermissionRequest): Promise<Permission> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/permissions/${userId}`
      : `/api/v1/drive/folders/${resourceId}/permissions/${userId}`;
    return request<Permission>(path, { method: 'PATCH', body: JSON.stringify(body) });
  },

  async revokePermission(resourceType: ResourceType, resourceId: string, userId: string): Promise<void> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/permissions/${userId}`
      : `/api/v1/drive/folders/${resourceId}/permissions/${userId}`;
    return request<void>(path, { method: 'DELETE' });
  },

  async transferOwnership(resourceType: ResourceType, resourceId: string, newOwnerId: string): Promise<void> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/transfer-ownership`
      : `/api/v1/drive/folders/${resourceId}/transfer-ownership`;
    return request<void>(path, { method: 'POST', body: JSON.stringify({ newOwnerId }) });
  },
};

// ---------------------------------------------------------------------------
// Sharing (share link) API
// ---------------------------------------------------------------------------

export const sharingApi = {
  async getShareLink(resourceType: ResourceType, resourceId: string): Promise<ShareLink | null> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/share-link`
      : `/api/v1/drive/folders/${resourceId}/share-link`;
    try {
      const link = await request<ShareLink>(path);
      return normalizeShareLink(link);
    } catch (e) {
      if (e instanceof ApiClientError && e.statusCode === 404) return null;
      throw e;
    }
  },

  async upsertShareLink(resourceType: ResourceType, resourceId: string, body: UpsertShareLinkRequest): Promise<ShareLink> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/share-link`
      : `/api/v1/drive/folders/${resourceId}/share-link`;
    const link = await request<ShareLink>(path, { method: 'PUT', body: JSON.stringify(body) });
    return normalizeShareLink(link);
  },

  async updateShareLink(resourceType: ResourceType, resourceId: string, body: UpdateShareLinkRequest): Promise<ShareLink> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/share-link`
      : `/api/v1/drive/folders/${resourceId}/share-link`;
    const link = await request<ShareLink>(path, { method: 'PATCH', body: JSON.stringify(body) });
    return normalizeShareLink(link);
  },

  async deleteShareLink(resourceType: ResourceType, resourceId: string): Promise<void> {
    const path = resourceType === 'file'
      ? `/api/v1/drive/files/${resourceId}/share-link`
      : `/api/v1/drive/folders/${resourceId}/share-link`;
    return request<void>(path, { method: 'DELETE' });
  },

  async resolveToken(token: string): Promise<ResolvedShareLink> {
    const link = await request<ResolvedShareLink>(`/api/v1/share/${token}`, {}, { auth: 'none' });
    return normalizeResolvedShareLink(link);
  },
};

export function getShareDownloadUrl(token: string): string {
  return `${BASE_URL}/api/v1/share/${token}/download`;
}

export function getSharePreviewUrl(token: string): string {
  return `${BASE_URL}/api/v1/share/${token}/preview`;
}

// ---------------------------------------------------------------------------
// Users API
// ---------------------------------------------------------------------------

export const usersApi = {
  async lookupByEmail(email: string): Promise<UserLookup | null> {
    try {
      return await request<UserLookup>(`/api/v1/auth/users/lookup?email=${encodeURIComponent(email)}`);
    } catch (e) {
      if (e instanceof ApiClientError && e.statusCode === 404) return null;
      throw e;
    }
  },

  async getById(userId: string): Promise<UserLookup | null> {
    try {
      return await request<UserLookup>(`/api/v1/auth/users/${userId}`);
    } catch (e) {
      if (e instanceof ApiClientError && e.statusCode === 404) return null;
      throw e;
    }
  },
};

// ---------------------------------------------------------------------------
// Access Requests API
// ---------------------------------------------------------------------------

export const accessRequestsApi = {
  async requestFileAccess(fileId: string, body: CreateAccessRequestRequest): Promise<AccessRequest> {
    return request<AccessRequest>(`/api/v1/drive/files/${fileId}/request-access`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async requestFolderAccess(folderId: string, body: CreateAccessRequestRequest): Promise<AccessRequest> {
    return request<AccessRequest>(`/api/v1/drive/folders/${folderId}/request-access`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async listPending(): Promise<ListAccessRequestsResponse> {
    return request<ListAccessRequestsResponse>('/api/v1/drive/access-requests');
  },

  async approve(requestId: string, body: ApproveAccessRequestRequest): Promise<AccessRequest> {
    return request<AccessRequest>(`/api/v1/drive/access-requests/${requestId}/approve`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async deny(requestId: string): Promise<AccessRequest> {
    return request<AccessRequest>(`/api/v1/drive/access-requests/${requestId}/deny`, {
      method: 'POST',
    });
  },
};

// ---------------------------------------------------------------------------
// Shared With Me API
// ---------------------------------------------------------------------------

export const sharedWithMeApi = {
  async list(): Promise<SharedWithMeResponse> {
    return request<SharedWithMeResponse>('/api/v1/drive/shared-with-me');
  },
};

// ---------------------------------------------------------------------------
// Drive content helpers (read/write content directly, bypassing app services)
// ---------------------------------------------------------------------------

/**
 * Fetch file content as text directly from the drive API.
 * @param path - The `contentUrl` returned by the app service (e.g. /api/v1/drive/files/{id})
 */
export async function driveReadContent(path: string): Promise<string> {
  return request<string>(path, {}, { responseType: 'text' });
}

/**
 * Upload file content directly to the drive API as a new version.
 * @param path - The `contentWriteUrl` returned by the app service (e.g. /api/v1/drive/files/{id}/versions)
 * @param content - The content string to upload
 * @param filename - Filename hint for the multipart upload (e.g. "doc.json")
 */
export async function driveWriteContent(path: string, content: string, filename: string): Promise<void> {
  const formData = new FormData();
  formData.append('file', new Blob([content], { type: 'application/json' }), filename);
  return request<void>(path, { method: 'POST', body: formData });
}

// ---------------------------------------------------------------------------
// Docs API
// ---------------------------------------------------------------------------

export interface PageSetup {
  marginTop: number;
  marginBottom: number;
  marginLeft: number;
  marginRight: number;
  orientation: 'portrait' | 'landscape';
  pageSize: 'letter' | 'a4' | 'legal' | 'a3' | 'a5' | 'tabloid' | 'executive';
}

export interface DocResponse {
  id: string;
  title: string;
  /** Path to read document content directly from the drive API (GET). */
  contentUrl: string;
  /** Path to write document content directly to the drive API (multipart POST). */
  contentWriteUrl: string;
  pageSetup: PageSetup;
  folderId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface DocMetaResponse {
  id: string;
  title: string;
  folderId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateDocRequest {
  title: string;
  folderId?: string | null;
}

export interface SaveDocRequest {
  pageSetup?: PageSetup;
  title?: string;
}

export interface ExportTextResponse {
  text: string;
  wordCount: number;
  charCount: number;
}

export interface ListDocsResponse {
  docs: DocMetaResponse[];
}

export const docsApi = {
  async listDocs(): Promise<ListDocsResponse> {
    return request<ListDocsResponse>('/api/v1/docs');
  },

  async createDoc(body: CreateDocRequest): Promise<DocResponse> {
    return request<DocResponse>('/api/v1/docs', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async getDoc(docId: string): Promise<DocResponse> {
    return request<DocResponse>(`/api/v1/docs/${docId}`);
  },

  async saveDoc(docId: string, body: SaveDocRequest): Promise<DocMetaResponse> {
    return request<DocMetaResponse>(`/api/v1/docs/${docId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },

  async exportText(docId: string): Promise<ExportTextResponse> {
    return request<ExportTextResponse>(`/api/v1/docs/${docId}/export/text`);
  },
};

// ---------------------------------------------------------------------------
// Sheets API
// ---------------------------------------------------------------------------

export interface SheetResponse {
  id: string;
  title: string;
  /** Path to read spreadsheet content directly from the drive API (GET). */
  contentUrl: string;
  /** Path to write spreadsheet content directly to the drive API (multipart POST). */
  contentWriteUrl: string;
  folderId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface SheetMetaResponse {
  id: string;
  title: string;
  folderId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateSheetRequest {
  title: string;
  folderId?: string | null;
}

export interface SaveSheetRequest {
  title?: string;
}

export interface ListSheetsResponse {
  sheets: SheetMetaResponse[];
}

export const sheetsApi = {
  async listSheets(): Promise<ListSheetsResponse> {
    return request<ListSheetsResponse>('/api/v1/sheets');
  },

  async createSheet(body: CreateSheetRequest): Promise<SheetResponse> {
    return request<SheetResponse>('/api/v1/sheets', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async getSheet(sheetId: string): Promise<SheetResponse> {
    return request<SheetResponse>(`/api/v1/sheets/${sheetId}`);
  },

  async saveSheet(sheetId: string, body: SaveSheetRequest): Promise<SheetMetaResponse> {
    return request<SheetMetaResponse>(`/api/v1/sheets/${sheetId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },
};

// ---------------------------------------------------------------------------
// Slides API
// ---------------------------------------------------------------------------

export interface SlideResponse {
  id: string;
  title: string;
  /** Path to read presentation content directly from the drive API (GET). */
  contentUrl: string;
  /** Path to write presentation content directly to the drive API (multipart POST). */
  contentWriteUrl: string;
  folderId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface SlideMetaResponse {
  id: string;
  title: string;
  folderId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateSlideRequest {
  title: string;
  folderId?: string | null;
}

export interface SaveSlideRequest {
  title?: string;
}

export interface ListSlidesResponse {
  slides: SlideMetaResponse[];
}

export const slidesApi = {
  async listSlides(): Promise<ListSlidesResponse> {
    return request<ListSlidesResponse>('/api/v1/slides');
  },

  async createSlide(body: CreateSlideRequest): Promise<SlideResponse> {
    return request<SlideResponse>('/api/v1/slides', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async getSlide(slideId: string): Promise<SlideResponse> {
    return request<SlideResponse>(`/api/v1/slides/${slideId}`);
  },

  async saveSlide(slideId: string, body: SaveSlideRequest): Promise<SlideMetaResponse> {
    return request<SlideMetaResponse>(`/api/v1/slides/${slideId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },
};

// ---------------------------------------------------------------------------
// Photos API (Phase 3.5)
// ---------------------------------------------------------------------------

export interface PhotoExifData {
  make?: string;
  model?: string;
  exposureTime?: string;
  fNumber?: number;
  iso?: number;
  focalLength?: number;
  gpsLatitude?: number;
  gpsLongitude?: number;
  datetimeOriginal?: string;
}

export interface PhotoMetadata {
  width?: number;
  height?: number;
  format?: string;
  exif?: PhotoExifData;
}

export interface PhotoResponse {
  id: string;
  fileId: string;
  fileName: string;
  mimeType: string;
  sizeBytes: number;
  /** URL to read/stream the media via Drive API */
  contentUrl: string;
  /** Base64-encoded thumbnail bytes, null if not yet generated */
  thumbnail: string | null;
  thumbnailMimeType: string | null;
  isStarred: boolean;
  isArchived: boolean;
  captureDate: string | null;
  createdAt: string;
  updatedAt: string;
  /** Extracted image metadata; null until the worker has processed the photo */
  metadata: PhotoMetadata | null;
}

export interface ListPhotosResponse {
  photos: PhotoResponse[];
  total: number;
}

export interface RegisterPhotoRequest {
  fileId: string;
  captureDate?: string | null;
}

export interface UpdatePhotoRequest {
  isStarred?: boolean;
  isArchived?: boolean;
}

export interface FaceBoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
  confidence: number;
  imageWidth: number;
  imageHeight: number;
}

export interface FaceResponse {
  id: string;
  photoId: string;
  boundingBox: FaceBoundingBox;
  thumbnail: string | null;
  thumbnailMimeType: string | null;
  personId: string | null;
  createdAt: string;
}

export interface ListFacesResponse {
  faces: FaceResponse[];
  total: number;
}

export interface PersonFaceThumbnail {
  id: string;
  thumbnail: string | null;
  thumbnailMimeType: string | null;
}

export interface PersonResponse {
  id: string;
  name: string | null;
  coverFaceId: string | null;
  coverThumbnail: string | null;
  coverThumbnailMimeType: string | null;
  faceCount: number;
  faces: PersonFaceThumbnail[];
  createdAt: string;
  updatedAt: string;
}

export interface ListPersonsResponse {
  persons: PersonResponse[];
  total: number;
}

export interface AlbumResponse {
  id: string;
  title: string;
  description: string | null;
  isAuto: boolean;
  personId: string | null;
  photoCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface ListAlbumsResponse {
  albums: AlbumResponse[];
}

export interface CreateAlbumRequest {
  title: string;
  description?: string | null;
}

export interface UpdateAlbumRequest {
  title?: string;
  description?: string | null;
}

// ---------------------------------------------------------------------------
// Phase 7 types: Timeline & Relationships
// ---------------------------------------------------------------------------

export interface TimelineGroup {
  label: string;
  month: string;
  photos: PhotoResponse[];
}

export interface PersonTimelineResponse {
  groups: TimelineGroup[];
}

export interface PersonRelationship {
  personAId: string;
  personAName: string | null;
  personAThumbnail: string | null;
  personAThumbnailMimeType: string | null;
  personBId: string;
  personBName: string | null;
  personBThumbnail: string | null;
  personBThumbnailMimeType: string | null;
  photoCount: number;
}

export interface PersonRelationshipsResponse {
  relationships: PersonRelationship[];
}

export const photosApi = {
  async listPhotos(opts?: {
    archivedOnly?: boolean;
    starredOnly?: boolean;
    personIds?: string[];
    excludePersonIds?: string[];
  }): Promise<ListPhotosResponse> {
    const qs = buildQuery({
      archivedOnly: opts?.archivedOnly,
      starredOnly: opts?.starredOnly,
      personIds: opts?.personIds?.length ? opts.personIds.join(',') : undefined,
      excludePersonIds: opts?.excludePersonIds?.length ? opts.excludePersonIds.join(',') : undefined,
    });
    return request<ListPhotosResponse>(`/api/v1/photos${qs}`);
  },

  async registerPhoto(body: RegisterPhotoRequest): Promise<PhotoResponse> {
    return request<PhotoResponse>('/api/v1/photos', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async getPhoto(photoId: string): Promise<PhotoResponse> {
    return request<PhotoResponse>(`/api/v1/photos/${photoId}`);
  },

  async updatePhoto(photoId: string, body: UpdatePhotoRequest): Promise<PhotoResponse> {
    return request<PhotoResponse>(`/api/v1/photos/${photoId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },

  async trashPhoto(photoId: string): Promise<void> {
    return request<void>(`/api/v1/photos/${photoId}`, { method: 'DELETE' });
  },

  async restorePhoto(photoId: string): Promise<PhotoResponse> {
    return request<PhotoResponse>(`/api/v1/photos/${photoId}/restore`, { method: 'POST' });
  },

  async listTrash(): Promise<ListPhotosResponse> {
    return request<ListPhotosResponse>('/api/v1/photos/trash');
  },

  async emptyTrash(): Promise<void> {
    return request<void>('/api/v1/photos/trash', { method: 'DELETE' });
  },

  /** Upload a media file to Drive then register it in Photos. Returns the photo record. */
  async uploadPhoto(
    file: File,
    onProgress?: (percent: number) => void,
  ): Promise<PhotoResponse> {
    const formData = new FormData();
    formData.append('file', file);
    const fileItem = await request<FileItem>('/api/v1/drive/files/upload', {
      method: 'POST',
      body: formData,
    }, { onUploadProgress: onProgress });
    return request<PhotoResponse>('/api/v1/photos', {
      method: 'POST',
      body: JSON.stringify({ fileId: fileItem.id } satisfies RegisterPhotoRequest),
    });
  },
};

export const facesApi = {
  async listFaces(photoId: string): Promise<ListFacesResponse> {
    return request<ListFacesResponse>(`/api/v1/photos/${photoId}/faces`);
  },
};

// ---------------------------------------------------------------------------
// Suggestions types & API
// ---------------------------------------------------------------------------

export interface SuggestionResponse {
  id: string;
  faceId: string;
  faceThumbnail: string | null;
  faceThumbnailMimeType: string | null;
  personId: string;
  personName: string | null;
  personThumbnail: string | null;
  personThumbnailMimeType: string | null;
  /** Similarity score 0–1; higher = better match */
  confidence: number;
  createdAt: string;
}

export interface ListSuggestionsResponse {
  suggestions: SuggestionResponse[];
  total: number;
}

export const suggestionsApi = {
  async listSuggestions(): Promise<ListSuggestionsResponse> {
    return request<ListSuggestionsResponse>('/api/v1/photos/suggestions');
  },
  async acceptSuggestion(id: string): Promise<void> {
    return request<void>(`/api/v1/photos/suggestions/${id}/accept`, { method: 'POST' });
  },
  async rejectSuggestion(id: string): Promise<void> {
    return request<void>(`/api/v1/photos/suggestions/${id}/reject`, { method: 'POST' });
  },
};

export const personsApi = {
  async listPersons(): Promise<ListPersonsResponse> {
    return request<ListPersonsResponse>('/api/v1/photos/persons/list');
  },
  async listPersonPhotos(personId: string): Promise<ListPhotosResponse> {
    return request<ListPhotosResponse>(`/api/v1/photos/persons/${personId}/photos`);
  },
  async getPersonTimeline(personId: string): Promise<PersonTimelineResponse> {
    return request<PersonTimelineResponse>(`/api/v1/photos/persons/${personId}/timeline`);
  },
  async getRelationships(): Promise<PersonRelationshipsResponse> {
    return request<PersonRelationshipsResponse>('/api/v1/photos/persons/relationships');
  },
  async createSmartAlbum(personId: string): Promise<AlbumResponse> {
    return request<AlbumResponse>(`/api/v1/photos/persons/${personId}/smart-album`, {
      method: 'POST',
    });
  },
  async renamePerson(personId: string, name: string): Promise<PersonResponse> {
    return request<PersonResponse>(`/api/v1/photos/persons/${personId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    });
  },
  async mergePersons(targetId: string, sourceId: string): Promise<PersonResponse> {
    return request<PersonResponse>(`/api/v1/photos/persons/${targetId}/merge`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ sourceId }),
    });
  },
  async reassignFace(personId: string, faceId: string, targetPersonId: string): Promise<void> {
    return request<void>(`/api/v1/photos/persons/${personId}/faces/${faceId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ targetPersonId }),
    });
  },
  async removeFace(personId: string, faceId: string): Promise<void> {
    return request<void>(`/api/v1/photos/persons/${personId}/faces/${faceId}`, {
      method: 'DELETE',
    });
  },
};

export const albumsApi = {
  async listAlbums(): Promise<ListAlbumsResponse> {
    return request<ListAlbumsResponse>('/api/v1/albums');
  },

  async createAlbum(body: CreateAlbumRequest): Promise<AlbumResponse> {
    return request<AlbumResponse>('/api/v1/albums', {
      method: 'POST',
      body: JSON.stringify(body),
    });
  },

  async getAlbum(albumId: string): Promise<AlbumResponse> {
    return request<AlbumResponse>(`/api/v1/albums/${albumId}`);
  },

  async updateAlbum(albumId: string, body: UpdateAlbumRequest): Promise<AlbumResponse> {
    return request<AlbumResponse>(`/api/v1/albums/${albumId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    });
  },

  async deleteAlbum(albumId: string): Promise<void> {
    return request<void>(`/api/v1/albums/${albumId}`, { method: 'DELETE' });
  },

  async addPhoto(albumId: string, photoId: string): Promise<void> {
    return request<void>(`/api/v1/albums/${albumId}/items`, {
      method: 'POST',
      body: JSON.stringify({ photoId }),
    });
  },

  async removePhoto(albumId: string, photoId: string): Promise<void> {
    return request<void>(`/api/v1/albums/${albumId}/items/${photoId}`, { method: 'DELETE' });
  },
};

// ---------------------------------------------------------------------------
// Comments types
// ---------------------------------------------------------------------------

export interface CommentReply {
  id: string;
  commentId: string;
  userId: string;
  userName: string;
  body: string;
  createdAt: string;
  updatedAt: string;
}

export interface Comment {
  id: string;
  fileId: string;
  userId: string;
  userName: string;
  anchorJson: string | null;
  body: string;
  status: 'open' | 'resolved';
  assigneeId: string | null;
  createdAt: string;
  updatedAt: string;
  resolvedAt: string | null;
  resolvedBy: string | null;
  replies: CommentReply[];
}

export interface CommentListResponse {
  comments: Comment[];
  total: number;
}

// ---------------------------------------------------------------------------
// Comments API
// ---------------------------------------------------------------------------

export const commentsApi = {
  async listComments(fileId: string, status?: 'open' | 'resolved'): Promise<CommentListResponse> {
    const qs = status ? `?status=${status}` : '';
    return request<CommentListResponse>(`/api/v1/drive/files/${fileId}/comments${qs}`);
  },

  async createComment(
    fileId: string,
    body: string,
    anchorJson?: string,
    assigneeId?: string,
  ): Promise<Comment> {
    return request<Comment>(`/api/v1/drive/files/${fileId}/comments`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ body, anchorJson, assigneeId }),
    });
  },

  async updateComment(
    fileId: string,
    commentId: string,
    patch: { body?: string; status?: string },
  ): Promise<Comment> {
    return request<Comment>(`/api/v1/drive/files/${fileId}/comments/${commentId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(patch),
    });
  },

  async deleteComment(fileId: string, commentId: string): Promise<void> {
    return request<void>(`/api/v1/drive/files/${fileId}/comments/${commentId}`, {
      method: 'DELETE',
    });
  },

  async addReply(fileId: string, commentId: string, body: string): Promise<CommentReply> {
    return request<CommentReply>(`/api/v1/drive/files/${fileId}/comments/${commentId}/replies`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ body }),
    });
  },

  async deleteReply(fileId: string, commentId: string, replyId: string): Promise<void> {
    return request<void>(
      `/api/v1/drive/files/${fileId}/comments/${commentId}/replies/${replyId}`,
      { method: 'DELETE' },
    );
  },
};

// ---------------------------------------------------------------------------
// Convenience re-exports
// ---------------------------------------------------------------------------

export { ApiClientError };
