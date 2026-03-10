import Foundation
import Photos

@MainActor
final class SyncManager: ObservableObject {
    static let shared = SyncManager()

    @Published private(set) var isSyncing = false
    @Published private(set) var lastSyncMessage: String?

    private let settings = AppSettings.shared
    private let drive = DriveService.shared

    private init() {}

    func syncAll() async {
        guard !isSyncing else { return }
        isSyncing = true
        defer { isSyncing = false }

        do {
            if settings.syncPhotosEnabled {
                try await syncRecentPhotos()
            }
            for folder in settings.syncFolders {
                try await syncFolder(folder)
            }
            lastSyncMessage = "Sync complete"
        } catch {
            lastSyncMessage = error.localizedDescription
        }
    }

    private func syncFolder(_ folder: SyncFolder) async throws {
        let url = try SyncBookmark.resolveURL(from: folder.bookmarkData)
        let didStart = url.startAccessingSecurityScopedResource()
        defer { if didStart { url.stopAccessingSecurityScopedResource() } }

        let targetFolder = try await ensureRemoteFolder(named: folder.name)
        let enumerator = FileManager.default.enumerator(at: url, includingPropertiesForKeys: [.isDirectoryKey], options: [.skipsHiddenFiles])
        while let fileURL = enumerator?.nextObject() as? URL {
            let values = try fileURL.resourceValues(forKeys: [.isDirectoryKey])
            if values.isDirectory == true { continue }
            try await uploadFile(fileURL: fileURL, to: targetFolder)
        }
    }

    private func syncRecentPhotos() async throws {
        let status = await requestPhotoAccess()
        guard status == .authorized || status == .limited else {
            throw APIClientError(statusCode: 0, code: "PHOTO_DENIED", message: "Photo access not granted")
        }

        let cutoff = Calendar.current.date(byAdding: .day, value: -30, to: Date()) ?? Date()
        let fetch = PHAsset.fetchAssets(with: nil)
        var assets: [PHAsset] = []
        fetch.enumerateObjects { asset, _, _ in
            if let date = asset.creationDate, date >= cutoff {
                assets.append(asset)
            }
        }

        let targetFolder = try await ensureRemoteFolder(named: "Photos")
        for asset in assets {
            if let tempURL = try await exportAsset(asset) {
                try await uploadFile(fileURL: tempURL, to: targetFolder)
                try? FileManager.default.removeItem(at: tempURL)
            }
        }
    }

    private func ensureRemoteFolder(named name: String) async throws -> FolderItem? {
        let root = try await drive.fetchRootContents()
        if let existing = root.folders.first(where: { $0.name == name }) {
            return existing
        }
        try await drive.createFolder(name: name, parentId: nil)
        let refreshed = try await drive.fetchRootContents()
        return refreshed.folders.first(where: { $0.name == name })
    }

    private func uploadFile(fileURL: URL, to folder: FolderItem?) async throws {
        let file = try await drive.uploadFile(localURL: fileURL)
        if let folderId = folder?.id {
            _ = try await drive.moveFile(fileId: file.id, folderId: folderId)
        }
    }

    private func exportAsset(_ asset: PHAsset) async throws -> URL? {
        let resources = PHAssetResource.assetResources(for: asset)
        guard let resource = resources.first else { return nil }
        let tempURL = FileManager.default.temporaryDirectory.appendingPathComponent(resource.originalFilename)

        return try await withCheckedThrowingContinuation { continuation in
            let options = PHAssetResourceRequestOptions()
            options.isNetworkAccessAllowed = true
            PHAssetResourceManager.default().writeData(for: resource, toFile: tempURL, options: options) { error in
                if let error { continuation.resume(throwing: error) }
                else { continuation.resume(returning: tempURL) }
            }
        }
    }

    private func requestPhotoAccess() async -> PHAuthorizationStatus {
        await withCheckedContinuation { continuation in
            PHPhotoLibrary.requestAuthorization(for: .readWrite) { status in
                continuation.resume(returning: status)
            }
        }
    }
}
