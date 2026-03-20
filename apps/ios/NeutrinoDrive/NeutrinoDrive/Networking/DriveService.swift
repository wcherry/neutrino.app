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
        cache.updateStarred(fileId: file.id, isStarred: updated.isStarred)
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

    func downloadFile(_ file: FileItem, makeAvailableOffline: Bool = false) async throws -> URL {
        if let cached = cache.localURL(for: file.id) {
            cache.upsert(
                fileId: file.id,
                name: file.name,
                mimeType: file.mimeType,
                localURL: cached,
                isAvailableOffline: makeAvailableOffline ? true : nil,
                isStarred: file.isStarred,
                remoteVersionIdentifier: versionIdentifier(updatedAt: file.updatedAt),
                remoteUpdatedAt: file.updatedAt
            )
            cache.recordAccess(fileId: file.id)
            return cached
        }

        let data = try await api.requestData("api/v1/drive/files/\(file.id)")
        let directory = cache.cacheDirectory()
        let safeName = file.name.replacingOccurrences(of: "/", with: "_")
        let localURL = directory.appendingPathComponent("\(file.id)-\(safeName)")
        try data.write(to: localURL, options: [.atomic])
        cache.upsert(
            fileId: file.id,
            name: file.name,
            mimeType: file.mimeType,
            localURL: localURL,
            isAvailableOffline: makeAvailableOffline,
            isStarred: file.isStarred,
            remoteVersionIdentifier: versionIdentifier(updatedAt: file.updatedAt),
            remoteUpdatedAt: file.updatedAt
        )
        return localURL
    }

    func prepareOfflineFileForPreview(_ item: CachedFile) async -> CachedFile {
        guard let localURL = cache.localURL(for: item.id) else { return item }
        let cachedItem = cache.cachedFile(fileId: item.id) ?? item

        do {
            let remoteFreshness = try await fetchRemoteFreshness(fileId: item.id)
            if shouldRefreshOfflineFile(cachedItem, localURL: localURL, remote: remoteFreshness) {
                return try await refreshOfflineFile(cachedItem, with: remoteFreshness)
            }

            bootstrapFreshnessMetadataIfNeeded(for: cachedItem, localURL: localURL, remote: remoteFreshness)
        } catch {
            // Fall back to the existing offline file if freshness checks fail.
        }

        cache.recordAccess(fileId: item.id)
        return cache.cachedFile(fileId: item.id) ?? cachedItem
    }

    func loadNeutrinoPreview(file: FileItem) async throws -> NeutrinoPreviewPayload? {
        guard let kind = file.neutrinoPreviewKind else { return nil }
        let data = try await api.requestData("api/v1/drive/files/\(file.id)")
        let content = decodeTextContent(data)
        return NeutrinoPreviewPayload(title: file.name, kind: kind, content: content)
    }

    private func decodeTextContent(_ data: Data) -> String {
        if let text = String(data: data, encoding: .utf8) {
            return text
        }
        if let text = String(data: data, encoding: .utf16) {
            return text
        }
        if let text = String(data: data, encoding: .ascii) {
            return text
        }
        return ""
    }

    private func refreshOfflineFile(
        _ item: CachedFile,
        with freshness: RemoteFileFreshness
    ) async throws -> CachedFile {
        let data = try await api.requestData("/api/v1/drive/files/\(item.id)")
        let safeName = item.name.replacingOccurrences(of: "/", with: "_")
        let localURL = cache.cacheDirectory().appendingPathComponent("\(item.id)-\(safeName)")
        try data.write(to: localURL, options: [.atomic])
        cache.upsert(
            fileId: item.id,
            name: item.name,
            mimeType: item.mimeType,
            localURL: localURL,
            isAvailableOffline: item.isAvailableOffline,
            isStarred: item.isStarred,
            remoteVersionIdentifier: freshness.versionIdentifier ?? item.remoteVersionIdentifier,
            remoteUpdatedAt: freshness.updatedAt ?? item.remoteUpdatedAt
        )
        return cache.cachedFile(fileId: item.id) ?? item
    }

    private func fetchRemoteFreshness(fileId: String) async throws -> RemoteFileFreshness {
        let candidatePaths = [
            "/api/v1/drive/file/\(fileId)",
            "/api/v1/drive/files/\(fileId)"
        ]
        var lastError: Error?

        for path in candidatePaths {
            do {
                let (data, response) = try await api.requestResponse(path, method: "OPTIONS")
                return parseRemoteFreshness(from: data, response: response)
            } catch let error as APIClientError where error.statusCode == 404 || error.statusCode == 405 {
                lastError = error
                continue
            } catch {
                lastError = error
                break
            }
        }

        throw lastError ?? APIClientError(
            statusCode: 0,
            code: "OFFLINE_REFRESH_UNAVAILABLE",
            message: "Unable to check file freshness"
        )
    }

    private func parseRemoteFreshness(
        from data: Data,
        response: HTTPURLResponse
    ) -> RemoteFileFreshness {
        let body = parseJSONDictionary(data)

        let versionNumber = intValue(
            in: body,
            keys: ["versionNumber", "latestVersionNumber", "fileVersion", "version"]
        ) ?? intHeader(
            in: response,
            names: ["X-File-Version", "X-Version-Number", "X-Latest-Version"]
        )

        let updatedAt = stringValue(
            in: body,
            keys: ["updatedAt", "lastModified"]
        ) ?? headerValue(
            in: response,
            names: ["X-Updated-At", "Last-Modified"]
        )

        let eTag = stringValue(in: body, keys: ["etag", "eTag"]) ?? headerValue(in: response, names: ["ETag"])

        let versionIdentifier: String?
        if let versionNumber {
            versionIdentifier = "version:\(versionNumber)"
        } else if let updatedAt, !updatedAt.isEmpty {
            versionIdentifier = "updatedAt:\(updatedAt)"
        } else if let eTag, !eTag.isEmpty {
            versionIdentifier = "etag:\(eTag)"
        } else {
            versionIdentifier = nil
        }

        return RemoteFileFreshness(
            versionIdentifier: versionIdentifier,
            updatedAt: updatedAt,
            updatedAtDate: updatedAt.flatMap(parseRemoteDate)
        )
    }

    private func shouldRefreshOfflineFile(
        _ cached: CachedFile,
        localURL: URL,
        remote: RemoteFileFreshness
    ) -> Bool {
        if let remoteIdentifier = remote.versionIdentifier,
           let cachedIdentifier = cached.remoteVersionIdentifier {
            return remoteIdentifier != cachedIdentifier
        }

        guard let remoteDate = remote.updatedAtDate else { return false }

        if let cachedDate = cached.remoteUpdatedAt.flatMap(parseRemoteDate) {
            return remoteDate.timeIntervalSince(cachedDate) > 1
        }

        if let localModifiedAt = localFileModifiedAt(for: localURL) {
            return remoteDate.timeIntervalSince(localModifiedAt) > 1
        }

        return false
    }

    private func bootstrapFreshnessMetadataIfNeeded(
        for cached: CachedFile,
        localURL: URL,
        remote: RemoteFileFreshness
    ) {
        guard cached.remoteVersionIdentifier == nil || cached.remoteUpdatedAt == nil else { return }
        guard let remoteIdentifier = remote.versionIdentifier else { return }

        if let remoteDate = remote.updatedAtDate,
           let localModifiedAt = localFileModifiedAt(for: localURL),
           abs(remoteDate.timeIntervalSince(localModifiedAt)) <= 1 {
            cache.updateFreshnessMetadata(
                fileId: cached.id,
                remoteVersionIdentifier: remoteIdentifier,
                remoteUpdatedAt: remote.updatedAt
            )
        }
    }

    private func versionIdentifier(updatedAt: String) -> String? {
        guard !updatedAt.isEmpty else { return nil }
        return "updatedAt:\(updatedAt)"
    }

    private func parseJSONDictionary(_ data: Data) -> [String: Any] {
        guard !data.isEmpty,
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return [:]
        }
        return json.reduce(into: [String: Any]()) { partialResult, item in
            partialResult[normalizedKey(item.key)] = item.value
        }
    }

    private func stringValue(in dictionary: [String: Any], keys: [String]) -> String? {
        for key in keys {
            if let value = dictionary[normalizedKey(key)] as? String {
                return value
            }
        }
        return nil
    }

    private func intValue(in dictionary: [String: Any], keys: [String]) -> Int? {
        for key in keys {
            let normalized = normalizedKey(key)
            if let value = dictionary[normalized] as? Int {
                return value
            }
            if let value = dictionary[normalized] as? NSNumber {
                return value.intValue
            }
            if let value = dictionary[normalized] as? String, let parsed = Int(value) {
                return parsed
            }
        }
        return nil
    }

    private func headerValue(in response: HTTPURLResponse, names: [String]) -> String? {
        let headers = response.allHeaderFields.reduce(into: [String: String]()) { partialResult, item in
            partialResult[String(describing: item.key).lowercased()] = String(describing: item.value)
        }

        for name in names {
            if let value = headers[name.lowercased()], !value.isEmpty {
                return value
            }
        }

        return nil
    }

    private func intHeader(in response: HTTPURLResponse, names: [String]) -> Int? {
        guard let value = headerValue(in: response, names: names) else { return nil }
        return Int(value)
    }

    private func normalizedKey(_ key: String) -> String {
        key.lowercased().filter { $0.isLetter || $0.isNumber }
    }

    private func parseRemoteDate(_ value: String) -> Date? {
        let iso8601WithFractional = ISO8601DateFormatter()
        iso8601WithFractional.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let date = iso8601WithFractional.date(from: value) {
            return date
        }

        let iso8601 = ISO8601DateFormatter()
        iso8601.formatOptions = [.withInternetDateTime]
        if let date = iso8601.date(from: value) {
            return date
        }

        let formatters: [DateFormatter] = {
            let httpDate = DateFormatter()
            httpDate.locale = Locale(identifier: "en_US_POSIX")
            httpDate.timeZone = TimeZone(secondsFromGMT: 0)
            httpDate.dateFormat = "EEE, dd MMM yyyy HH:mm:ss zzz"

            let backendDate = DateFormatter()
            backendDate.locale = Locale(identifier: "en_US_POSIX")
            backendDate.timeZone = TimeZone(secondsFromGMT: 0)
            backendDate.dateFormat = "yyyy-MM-dd'T'HH:mm:ss"

            let backendDateWithSpace = DateFormatter()
            backendDateWithSpace.locale = Locale(identifier: "en_US_POSIX")
            backendDateWithSpace.timeZone = TimeZone(secondsFromGMT: 0)
            backendDateWithSpace.dateFormat = "yyyy-MM-dd HH:mm:ss"

            let backendDateWithFractional = DateFormatter()
            backendDateWithFractional.locale = Locale(identifier: "en_US_POSIX")
            backendDateWithFractional.timeZone = TimeZone(secondsFromGMT: 0)
            backendDateWithFractional.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS"

            return [httpDate, backendDate, backendDateWithSpace, backendDateWithFractional]
        }()

        for formatter in formatters {
            if let date = formatter.date(from: value) {
                return date
            }
        }

        return nil
    }

    private func localFileModifiedAt(for url: URL) -> Date? {
        let values = try? url.resourceValues(forKeys: [.contentModificationDateKey])
        return values?.contentModificationDate
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

private struct RemoteFileFreshness {
    let versionIdentifier: String?
    let updatedAt: String?
    let updatedAtDate: Date?
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
