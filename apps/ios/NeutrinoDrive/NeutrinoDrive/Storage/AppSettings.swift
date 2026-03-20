import Foundation

@MainActor
final class AppSettings: ObservableObject {
    static let shared = AppSettings()

    private let baseUrlKey = "neutrino.base_url"
    private let syncFoldersKey = "neutrino.sync_folders"
    private let syncPhotosKey = "neutrino.sync_photos"
    private let biometricKey = "neutrino.biometric_enabled"

    @Published var baseUrl: String {
        didSet { UserDefaults.standard.set(baseUrl, forKey: baseUrlKey) }
    }

    @Published var syncFolders: [SyncFolder] {
        didSet { saveSyncFolders() }
    }

    @Published var syncPhotosEnabled: Bool {
        didSet { UserDefaults.standard.set(syncPhotosEnabled, forKey: syncPhotosKey) }
    }

    @Published var biometricEnabled: Bool {
        didSet { UserDefaults.standard.set(biometricEnabled, forKey: biometricKey) }
    }

    private init() {
        self.baseUrl = UserDefaults.standard.string(forKey: baseUrlKey) ?? ""
        self.syncPhotosEnabled = UserDefaults.standard.bool(forKey: syncPhotosKey)
        self.syncFolders = AppSettings.loadSyncFolders(key: syncFoldersKey)
        self.biometricEnabled = UserDefaults.standard.bool(forKey: biometricKey)
    }

    func reset() {
        UserDefaults.standard.removeObject(forKey: baseUrlKey)
        UserDefaults.standard.removeObject(forKey: syncFoldersKey)
        UserDefaults.standard.removeObject(forKey: syncPhotosKey)
        UserDefaults.standard.removeObject(forKey: biometricKey)
        baseUrl = ""
        syncFolders = []
        syncPhotosEnabled = false
        biometricEnabled = false
    }

    func addSyncFolder(_ folder: SyncFolder) {
        syncFolders.append(folder)
    }

    func removeSyncFolder(id: UUID) {
        syncFolders.removeAll { $0.id == id }
    }

    private func saveSyncFolders() {
        if let data = try? JSONEncoder().encode(syncFolders) {
            UserDefaults.standard.set(data, forKey: syncFoldersKey)
        }
    }

    private static func loadSyncFolders(key: String) -> [SyncFolder] {
        guard let data = UserDefaults.standard.data(forKey: key),
              let decoded = try? JSONDecoder().decode([SyncFolder].self, from: data) else {
            return []
        }
        return decoded
    }
}
