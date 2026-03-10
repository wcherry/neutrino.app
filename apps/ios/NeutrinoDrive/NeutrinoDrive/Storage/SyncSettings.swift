import Foundation

struct SyncFolder: Identifiable, Codable, Hashable {
    let id: UUID
    let name: String
    let bookmarkData: Data

    init(id: UUID = UUID(), name: String, bookmarkData: Data) {
        self.id = id
        self.name = name
        self.bookmarkData = bookmarkData
    }
}

enum SyncBookmark {
    static func createBookmark(for url: URL) throws -> Data {
        try url.bookmarkData(options: .minimalBookmark, includingResourceValuesForKeys: nil, relativeTo: nil)
    }

    static func resolveURL(from bookmark: Data) throws -> URL {
        var isStale = false
        let url = try URL(resolvingBookmarkData: bookmark, options: .withoutUI, relativeTo: nil, bookmarkDataIsStale: &isStale)
        return url
    }
}
