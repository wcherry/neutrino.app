import Foundation

struct APIClientError: Error, LocalizedError {
    let statusCode: Int
    let code: String
    let message: String

    var errorDescription: String? { message }
}

@MainActor
final class APIClient {
    static let shared = APIClient()

    private let settings = AppSettings.shared
    private let auth = AuthManager.shared

    private let decoder: JSONDecoder = {
        let decoder = JSONDecoder()
        return decoder
    }()

    private let encoder: JSONEncoder = {
        let encoder = JSONEncoder()
        return encoder
    }()

    private init() {}

    func request<T: Decodable>(
        _ path: String,
        method: String = "GET",
        body: Encodable? = nil,
        requiresAuth: Bool = true
    ) async throws -> T {
        if requiresAuth && auth.shouldRefreshSoon() {
            try await refreshTokensIfPossible()
        }
        let request = try buildRequest(path: path, method: method, body: body, requiresAuth: requiresAuth)
        return try await execute(request: request, decodeTo: T.self, retryOnAuth: requiresAuth)
    }

    func requestNoContent(
        _ path: String,
        method: String = "POST",
        body: Encodable? = nil,
        requiresAuth: Bool = true
    ) async throws {
        if requiresAuth && auth.shouldRefreshSoon() {
            try await refreshTokensIfPossible()
        }
        let request = try buildRequest(path: path, method: method, body: body, requiresAuth: requiresAuth)
        _ = try await execute(request: request, decodeTo: EmptyResponse.self, retryOnAuth: requiresAuth)
    }

    func requestData(
        _ path: String,
        method: String = "GET",
        requiresAuth: Bool = true
    ) async throws -> Data {
        if requiresAuth && auth.shouldRefreshSoon() {
            try await refreshTokensIfPossible()
        }
        let request = try buildRequest(path: path, method: method, body: nil, requiresAuth: requiresAuth)
        return try await executeData(request: request, retryOnAuth: requiresAuth)
    }

    func requestResponse(
        _ path: String,
        method: String = "GET",
        requiresAuth: Bool = true
    ) async throws -> (Data, HTTPURLResponse) {
        if requiresAuth && auth.shouldRefreshSoon() {
            try await refreshTokensIfPossible()
        }
        let request = try buildRequest(path: path, method: method, body: nil, requiresAuth: requiresAuth)
        return try await executeResponse(request: request, retryOnAuth: requiresAuth)
    }

    func uploadMultipart(
        _ path: String,
        fileURL: URL,
        fileName: String,
        mimeType: String,
        fieldName: String = "file"
    ) async throws -> FileItem {
        guard let base = URL(string: settings.baseUrl), !settings.baseUrl.isEmpty else {
            throw APIClientError(statusCode: 0, code: "NO_BASE_URL", message: "Server URL is not set")
        }

        let boundary = "Boundary-\(UUID().uuidString)"
        let tempFile = try MultipartWriter.writeMultipartFile(
            boundary: boundary,
            fileURL: fileURL,
            fieldName: fieldName,
            fileName: fileName,
            mimeType: mimeType
        )

        let cleanedPath = path.hasPrefix("/") ? path : "/" + path
        guard let url = URL(string: cleanedPath, relativeTo: base)?.absoluteURL else {
            throw APIClientError(statusCode: 0, code: "BAD_URL", message: "Invalid URL")
        }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        if let token = auth.accessToken {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        let (data, response) = try await URLSession.shared.upload(for: request, fromFile: tempFile)
        try? FileManager.default.removeItem(at: tempFile)
        return try decodeResponse(data: data, response: response, decodeTo: FileItem.self)
    }

    private func buildRequest(
        path: String,
        method: String,
        body: Encodable?,
        requiresAuth: Bool
    ) throws -> URLRequest {
        guard let base = URL(string: settings.baseUrl), !settings.baseUrl.isEmpty else {
            throw APIClientError(statusCode: 0, code: "NO_BASE_URL", message: "Server URL is not set")
        }
        let cleanedPath = path.hasPrefix("/") ? path : "/" + path
        guard let url = URL(string: cleanedPath, relativeTo: base)?.absoluteURL else {
            throw APIClientError(statusCode: 0, code: "BAD_URL", message: "Invalid URL")
        }
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        if requiresAuth, let token = auth.accessToken {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        if let body {
            request.httpBody = try encoder.encode(AnyEncodable(body))
        }

        return request
    }

    private func execute<T: Decodable>(
        request: URLRequest,
        decodeTo: T.Type,
        retryOnAuth: Bool
    ) async throws -> T {
        let (data, response) = try await URLSession.shared.data(for: request)
        if let http = response as? HTTPURLResponse, http.statusCode == 401, retryOnAuth {
            try await refreshTokensIfPossible()
            var retry = request
            if let token = auth.accessToken {
                retry.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
            }
            let (retryData, retryResponse) = try await URLSession.shared.data(for: retry)
            return try decodeResponse(data: retryData, response: retryResponse, decodeTo: T.self)
        }
        return try decodeResponse(data: data, response: response, decodeTo: T.self)
    }

    private func executeData(
        request: URLRequest,
        retryOnAuth: Bool
    ) async throws -> Data {
        let (data, _) = try await executeResponse(request: request, retryOnAuth: retryOnAuth)
        return data
    }

    private func executeResponse(
        request: URLRequest,
        retryOnAuth: Bool
    ) async throws -> (Data, HTTPURLResponse) {
        let (data, response) = try await URLSession.shared.data(for: request)
        if let http = response as? HTTPURLResponse, http.statusCode == 401, retryOnAuth {
            try await refreshTokensIfPossible()
            var retry = request
            if let token = auth.accessToken {
                retry.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
            }
            let (retryData, retryResponse) = try await URLSession.shared.data(for: retry)
            let http = try validatedHTTPResponse(data: retryData, response: retryResponse)
            return (retryData, http)
        }

        let http = try validatedHTTPResponse(data: data, response: response)
        return (data, http)
    }

    private func decodeResponse<T: Decodable>(
        data: Data,
        response: URLResponse,
        decodeTo: T.Type
    ) throws -> T {
        _ = try validatedHTTPResponse(data: data, response: response)
        if T.self == EmptyResponse.self {
            return EmptyResponse() as! T
        }
        return try decoder.decode(T.self, from: data)
    }

    private func validatedHTTPResponse(
        data: Data,
        response: URLResponse
    ) throws -> HTTPURLResponse {
        guard let http = response as? HTTPURLResponse else {
            throw APIClientError(statusCode: 0, code: "NO_RESPONSE", message: "No response")
        }
        guard (200...299).contains(http.statusCode) else {
            if let apiError = try? decoder.decode(ApiErrorEnvelope.self, from: data) {
                throw APIClientError(statusCode: http.statusCode, code: apiError.error.code, message: apiError.error.message)
            }
            throw APIClientError(statusCode: http.statusCode, code: "HTTP_\(http.statusCode)", message: "HTTP \(http.statusCode)")
        }
        return http
    }

    private func refreshTokensIfPossible() async throws {
        guard let refresh = auth.refreshToken else { throw APIClientError(statusCode: 401, code: "NO_REFRESH", message: "No refresh token") }
        do {
            let tokens: AuthTokens = try await request(
                "/api/v1/auth/refresh",
                method: "POST",
                body: RefreshRequest(refreshToken: refresh),
                requiresAuth: false
            )
            auth.storeTokens(tokens)
        } catch {
            auth.clearTokens()
            throw error
        }
    }
}

private struct EmptyResponse: Decodable {}

private struct AnyEncodable: Encodable {
    private let encodeBlock: (Encoder) throws -> Void

    init(_ value: Encodable) {
        self.encodeBlock = value.encode
    }

    func encode(to encoder: Encoder) throws {
        try encodeBlock(encoder)
    }
}

enum MultipartWriter {
    static func writeMultipartFile(
        boundary: String,
        fileURL: URL,
        fieldName: String,
        fileName: String,
        mimeType: String
    ) throws -> URL {
        let tempDir = FileManager.default.temporaryDirectory
        let tempFile = tempDir.appendingPathComponent("upload-\(UUID().uuidString)")
        FileManager.default.createFile(atPath: tempFile.path, contents: nil)
        let handle = try FileHandle(forWritingTo: tempFile)

        func write(_ string: String) throws {
            if let data = string.data(using: .utf8) {
                try handle.write(contentsOf: data)
            }
        }

        try write("--\(boundary)\r\n")
        try write("Content-Disposition: form-data; name=\"\(fieldName)\"; filename=\"\(fileName)\"\r\n")
        try write("Content-Type: \(mimeType)\r\n\r\n")

        let readHandle = try FileHandle(forReadingFrom: fileURL)
        while true {
            let chunk = try readHandle.read(upToCount: 1024 * 1024) ?? Data()
            if chunk.isEmpty { break }
            try handle.write(contentsOf: chunk)
        }
        try readHandle.close()

        try write("\r\n--\(boundary)--\r\n")
        try handle.close()

        return tempFile
    }
}
