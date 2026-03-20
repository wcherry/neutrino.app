import Foundation

struct CachedFile: Codable, Hashable {
    let id: String
    let name: String
    let mimeType: String
    let localPath: String
    var lastAccessed: Date
    var isAvailableOffline: Bool
    var isStarred: Bool
    var remoteVersionIdentifier: String?
    var remoteUpdatedAt: String?

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case mimeType
        case localPath
        case lastAccessed
        case isAvailableOffline
        case isStarred
        case remoteVersionIdentifier
        case remoteUpdatedAt
        case legacyPinned = "isPinned"
    }

    init(
        id: String,
        name: String,
        mimeType: String,
        localPath: String,
        lastAccessed: Date,
        isAvailableOffline: Bool,
        isStarred: Bool,
        remoteVersionIdentifier: String?,
        remoteUpdatedAt: String?
    ) {
        self.id = id
        self.name = name
        self.mimeType = mimeType
        self.localPath = localPath
        self.lastAccessed = lastAccessed
        self.isAvailableOffline = isAvailableOffline
        self.isStarred = isStarred
        self.remoteVersionIdentifier = remoteVersionIdentifier
        self.remoteUpdatedAt = remoteUpdatedAt
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        name = try container.decode(String.self, forKey: .name)
        mimeType = try container.decode(String.self, forKey: .mimeType)
        localPath = try container.decode(String.self, forKey: .localPath)
        lastAccessed = try container.decode(Date.self, forKey: .lastAccessed)

        let legacyPinned = try container.decodeIfPresent(Bool.self, forKey: .legacyPinned) ?? false
        isAvailableOffline = try container.decodeIfPresent(Bool.self, forKey: .isAvailableOffline) ?? legacyPinned
        isStarred = try container.decodeIfPresent(Bool.self, forKey: .isStarred) ?? false
        remoteVersionIdentifier = try container.decodeIfPresent(String.self, forKey: .remoteVersionIdentifier)
        remoteUpdatedAt = try container.decodeIfPresent(String.self, forKey: .remoteUpdatedAt)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(name, forKey: .name)
        try container.encode(mimeType, forKey: .mimeType)
        try container.encode(localPath, forKey: .localPath)
        try container.encode(lastAccessed, forKey: .lastAccessed)
        try container.encode(isAvailableOffline, forKey: .isAvailableOffline)
        try container.encode(isStarred, forKey: .isStarred)
        try container.encodeIfPresent(remoteVersionIdentifier, forKey: .remoteVersionIdentifier)
        try container.encodeIfPresent(remoteUpdatedAt, forKey: .remoteUpdatedAt)
    }
}

@MainActor
final class FileCache: ObservableObject {
    static let shared = FileCache()

    @Published private(set) var index: [String: CachedFile] = [:]

    private let cacheFolderName = "NeutrinoDriveCache"
    private let indexFileName = "cache-index.json"
    private let recentRetentionDays: Int = 30

    private init() {
        loadIndex()
        pruneExpired()
    }

    func localURL(for fileId: String) -> URL? {
        guard let entry = cachedFile(fileId: fileId) else { return nil }
        return URL(fileURLWithPath: entry.localPath)
    }

    func cachedFile(fileId: String) -> CachedFile? {
        guard let entry = index[fileId] else { return nil }
        let url = URL(fileURLWithPath: entry.localPath)
        guard FileManager.default.fileExists(atPath: url.path) else {
            index.removeValue(forKey: fileId)
            saveIndex()
            return nil
        }
        return entry
    }

    func isAvailableOffline(fileId: String) -> Bool {
        index[fileId]?.isAvailableOffline == true
    }

    func setAvailableOffline(fileId: String, availableOffline: Bool) {
        guard var entry = index[fileId] else { return }
        entry.isAvailableOffline = availableOffline
        entry.lastAccessed = Date()
        index[fileId] = entry
        saveIndex()
    }

    func updateStarred(fileId: String, isStarred: Bool) {
        guard var entry = index[fileId] else { return }
        entry.isStarred = isStarred
        entry.lastAccessed = Date()
        index[fileId] = entry
        saveIndex()
    }

    func updateFreshnessMetadata(
        fileId: String,
        remoteVersionIdentifier: String?,
        remoteUpdatedAt: String?
    ) {
        guard var entry = index[fileId] else { return }
        entry.remoteVersionIdentifier = remoteVersionIdentifier
        entry.remoteUpdatedAt = remoteUpdatedAt
        index[fileId] = entry
        saveIndex()
    }

    func recordAccess(fileId: String) {
        guard var entry = index[fileId] else { return }
        entry.lastAccessed = Date()
        index[fileId] = entry
        saveIndex()
    }

    func upsert(
        fileId: String,
        name: String,
        mimeType: String,
        localURL: URL,
        isAvailableOffline: Bool? = nil,
        isStarred: Bool? = nil,
        remoteVersionIdentifier: String? = nil,
        remoteUpdatedAt: String? = nil
    ) {
        let existing = index[fileId]
        let entry = CachedFile(
            id: fileId,
            name: name,
            mimeType: mimeType,
            localPath: localURL.path,
            lastAccessed: Date(),
            isAvailableOffline: isAvailableOffline ?? existing?.isAvailableOffline ?? false,
            isStarred: isStarred ?? existing?.isStarred ?? false,
            remoteVersionIdentifier: remoteVersionIdentifier ?? existing?.remoteVersionIdentifier,
            remoteUpdatedAt: remoteUpdatedAt ?? existing?.remoteUpdatedAt
        )
        index[fileId] = entry
        saveIndex()
    }

    func remove(fileId: String) {
        guard let entry = index[fileId] else { return }
        try? FileManager.default.removeItem(atPath: entry.localPath)
        index.removeValue(forKey: fileId)
        saveIndex()
    }

    func offlineFiles() -> [CachedFile] {
        index.values
            .filter { $0.isAvailableOffline && FileManager.default.fileExists(atPath: $0.localPath) }
            .sorted { $0.lastAccessed > $1.lastAccessed }
    }

    func pruneExpired() {
        let cutoff = Calendar.current.date(byAdding: .day, value: -recentRetentionDays, to: Date()) ?? Date()
        let missing = index.values.filter { !FileManager.default.fileExists(atPath: $0.localPath) }
        for entry in missing {
            index.removeValue(forKey: entry.id)
        }

        let expired = index.values.filter { !$0.isAvailableOffline && $0.lastAccessed < cutoff }
        for entry in expired {
            remove(fileId: entry.id)
        }

        if !missing.isEmpty {
            saveIndex()
        }
    }

    func cacheDirectory() -> URL {
        let base = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
        let dir = base.appendingPathComponent(cacheFolderName, isDirectory: true)
        if !FileManager.default.fileExists(atPath: dir.path) {
            try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        }
        return dir
    }

    private func loadIndex() {
        let url = cacheDirectory().appendingPathComponent(indexFileName)
        guard let data = try? Data(contentsOf: url) else { return }
        if let decoded = try? JSONDecoder().decode([String: CachedFile].self, from: data) {
            self.index = decoded
        }
    }

    private func saveIndex() {
        let url = cacheDirectory().appendingPathComponent(indexFileName)
        if let data = try? JSONEncoder().encode(index) {
            try? data.write(to: url, options: [.atomic])
        }
    }
}
