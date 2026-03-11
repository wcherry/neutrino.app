import Foundation

@MainActor
final class DriveService: ObservableObject {
    static let shared = DriveService()

    private let api = APIClient.shared
    private let auth = AuthManager.shared
    private let cache = FileCache.shared
    private let settings = AppSettings.shared

    private init() {}

    func login(email: String, password: String) async throws {
        let tokens: AuthTokens = try await api.request("/api/v1/auth/login", method: "POST", body: LoginRequest(email: email, password: password), requiresAuth: false)
        auth.storeTokens(tokens)
        try await loadProfile()
    }

    func register(name: String, email: String, password: String) async throws {
        let _: UserProfile = try await api.request(
            "/api/v1/auth/register",
            method: "POST",
            body: RegisterRequest(email: email, name: name, password: password),
            requiresAuth: false
        )
        try await login(email: email, password: password)
    }

    func logout() async {
        do {
            try await api.requestNoContent("/api/v1/auth/logout", method: "POST")
        } catch {
            // Ignore
        }
        auth.clearTokens()
        cache.pruneExpired()
    }

    func loadProfile() async throws {
        let profile: UserProfile = try await api.request("/api/v1/auth/me")
        auth.setProfile(profile)
    }

    func fetchRootContents() async throws -> FolderContentsResponse {
        try await api.request("/api/v1/drive?limit=100")
    }

    func fetchFolderContents(folderId: String) async throws -> FolderContentsResponse {
        try await api.request("/api/v1/drive/folders/\(folderId)")
    }

    func createFolder(name: String, parentId: String?) async throws {
        let _: FolderItem = try await api.request("/api/v1/drive/folders", method: "POST", body: CreateFolderRequest(name: name, parentId: parentId))
    }

    func toggleStar(file: FileItem) async throws -> FileItem {
        let updated: FileItem = try await api.request(
            "/api/v1/drive/files/\(file.id)",
            method: "PATCH",
            body: UpdateFileRequest(name: nil, folderId: nil, isStarred: !file.isStarred)
        )
        if cache.isPinned(fileId: file.id) != updated.isStarred {
            cache.setPinned(fileId: file.id, pinned: updated.isStarred)
        }
        return updated
    }

    func moveFile(fileId: String, folderId: String?) async throws -> FileItem {
        let payload = UpdateFileRequest(name: nil, folderId: .some(folderId), isStarred: nil)
        let updated: FileItem = try await api.request(
            "/api/v1/drive/files/\(fileId)",
            method: "PATCH",
            body: payload
        )
        return updated
    }

    func uploadFile(localURL: URL) async throws -> FileItem {
        let name = localURL.lastPathComponent
        let mimeType = MimeType.infer(from: localURL)
        let file = try await api.uploadMultipart("/api/v1/drive/files", fileURL: localURL, fileName: name, mimeType: mimeType)
        return file
    }

    func downloadFile(_ file: FileItem) async throws -> URL {
        if let cached = cache.localURL(for: file.id) {
            cache.recordAccess(fileId: file.id)
            return cached
        }

        let data = try await api.requestData("api/v1/drive/files/\(file.id)")
        let directory = cache.cacheDirectory()
        let safeName = file.name.replacingOccurrences(of: "/", with: "_")
        let localURL = directory.appendingPathComponent("\(file.id)-\(safeName)")
        try data.write(to: localURL, options: [.atomic])
        cache.upsert(fileId: file.id, name: file.name, mimeType: file.mimeType, localURL: localURL, pinned: file.isStarred)
        return localURL
    }

    func getShareURL(for file: FileItem) -> URL? {
        guard let base = URL(string: settings.baseUrl), !settings.baseUrl.isEmpty else { return nil }
        let token = auth.accessToken ?? ""
        let path = "/api/v1/drive/files/\(file.id)"
        guard let url = URL(string: path, relativeTo: base)?.absoluteURL else { return nil }
        var components = URLComponents(url: url, resolvingAgainstBaseURL: false)
        components?.queryItems = [URLQueryItem(name: "token", value: token)]
        return components?.url
    }
}

enum MimeType {
    static func infer(from url: URL) -> String {
        let ext = url.pathExtension.lowercased()
        switch ext {
        case "png": return "image/png"
        case "jpg", "jpeg": return "image/jpeg"
        case "gif": return "image/gif"
        case "heic": return "image/heic"
        case "pdf": return "application/pdf"
        case "mp4": return "video/mp4"
        case "mov": return "video/quicktime"
        case "mp3": return "audio/mpeg"
        case "wav": return "audio/wav"
        case "zip": return "application/zip"
        default: return "application/octet-stream"
        }
    }
}
