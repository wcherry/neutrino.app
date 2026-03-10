import Foundation

struct CachedFile: Codable, Hashable {
    let id: String
    let name: String
    let mimeType: String
    let localPath: String
    var lastAccessed: Date
    var isPinned: Bool
}

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
        guard let entry = index[fileId] else { return nil }
        return URL(fileURLWithPath: entry.localPath)
    }

    func isPinned(fileId: String) -> Bool {
        index[fileId]?.isPinned == true
    }

    func setPinned(fileId: String, pinned: Bool) {
        guard var entry = index[fileId] else { return }
        entry.isPinned = pinned
        entry.lastAccessed = Date()
        index[fileId] = entry
        saveIndex()
    }

    func recordAccess(fileId: String) {
        guard var entry = index[fileId] else { return }
        entry.lastAccessed = Date()
        index[fileId] = entry
        saveIndex()
    }

    func upsert(fileId: String, name: String, mimeType: String, localURL: URL, pinned: Bool) {
        let entry = CachedFile(
            id: fileId,
            name: name,
            mimeType: mimeType,
            localPath: localURL.path,
            lastAccessed: Date(),
            isPinned: pinned
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

    func cachedFiles(onlyPinned: Bool) -> [CachedFile] {
        index.values.filter { onlyPinned ? $0.isPinned : true }
            .sorted { $0.lastAccessed > $1.lastAccessed }
    }

    func pruneExpired() {
        let cutoff = Calendar.current.date(byAdding: .day, value: -recentRetentionDays, to: Date()) ?? Date()
        let expired = index.values.filter { !$0.isPinned && $0.lastAccessed < cutoff }
        for entry in expired {
            remove(fileId: entry.id)
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
