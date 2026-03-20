import Foundation

enum NeutrinoPreviewKind: String, Codable, Hashable {
    case doc
    case sheet
    case slide

    static func from(mimeType: String) -> NeutrinoPreviewKind? {
        switch mimeType.lowercased() {
        case "application/x-neutrino-doc":
            return .doc
        case "application/x-neutrino-sheet":
            return .sheet
        case "application/x-neutrino-slide", "application/x-neutrino-slides":
            return .slide
        default:
            return nil
        }
    }

    var displayName: String {
        switch self {
        case .doc:
            return "Doc"
        case .sheet:
            return "Sheet"
        case .slide:
            return "Slide"
        }
    }
}

struct NeutrinoPreviewPayload: Identifiable, Hashable {
    let id = UUID()
    let title: String
    let kind: NeutrinoPreviewKind
    let content: String
}

struct ApiErrorEnvelope: Codable, Error {
    let error: ApiErrorDetail
}

struct ApiErrorDetail: Codable {
    let code: String
    let message: String
}

struct AuthTokens: Codable {
    let accessToken: String
    let refreshToken: String
    let tokenType: String
    let expiresIn: Int
}

struct UserProfile: Codable {
    let id: String
    let email: String
    let name: String
    let createdAt: String
}

struct FileItem: Codable, Identifiable, Hashable {
    let id: String
    let name: String
    let sizeBytes: Int64
    let mimeType: String
    let folderId: String?
    let isStarred: Bool
    let createdAt: String
    let updatedAt: String

    var neutrinoPreviewKind: NeutrinoPreviewKind? {
        NeutrinoPreviewKind.from(mimeType: mimeType)
    }
}

struct FolderItem: Codable, Identifiable, Hashable {
    let id: String
    let name: String
    let parentId: String?
    let color: String?
    let isStarred: Bool
    let createdAt: String
    let updatedAt: String
}

struct ShortcutItem: Codable, Identifiable, Hashable {
    let id: String
    let targetFileId: String
    let folderId: String?
    let createdAt: String
}

struct FolderContentsResponse: Codable {
    let folder: FolderItem?
    let folders: [FolderItem]
    let files: [FileItem]
    let shortcuts: [ShortcutItem]
}

struct CreateFolderRequest: Codable {
    let name: String
    let parentId: String?
}

struct UpdateFileRequest: Encodable {
    let name: String?
    /// nil = omit, .some(nil) = set null (move to root), .some(.some(id)) = set folder
    let folderId: String??
    let isStarred: Bool?

    enum CodingKeys: String, CodingKey { case name, folderId, isStarred }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        if let name { try container.encode(name, forKey: .name) }
        if let isStarred { try container.encode(isStarred, forKey: .isStarred) }
        if let folderId = folderId {
            if let value = folderId {
                try container.encode(value, forKey: .folderId)
            } else {
                try container.encodeNil(forKey: .folderId)
            }
        }
    }
}

struct UpdateFolderRequest: Encodable {
    let name: String?
    /// nil = omit, .some(nil) = clear color, .some(.some(value)) = set color
    let color: String??
    let isStarred: Bool?

    enum CodingKeys: String, CodingKey { case name, color, isStarred }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        if let name { try container.encode(name, forKey: .name) }
        if let isStarred { try container.encode(isStarred, forKey: .isStarred) }
        if let color = color {
            if let value = color {
                try container.encode(value, forKey: .color)
            } else {
                try container.encodeNil(forKey: .color)
            }
        }
    }
}

struct QuotaInfo: Codable {
    let usedBytes: Int64
    let dailyUploadBytes: Int64
    let dailyResetAt: String
    let quotaBytes: Int64?
    let dailyCapBytes: Int64?
}

struct LoginRequest: Codable {
    let email: String
    let password: String
}

struct RegisterRequest: Codable {
    let email: String
    let name: String
    let password: String
}

struct RefreshRequest: Codable {
    let refreshToken: String
}

struct FileListResponse: Codable {
    let files: [FileItem]
    let total: Int
    let limit: Int
    let offset: Int
}
