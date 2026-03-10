import SwiftUI

@main
struct NeutrinoDriveApp: App {
    @StateObject private var auth = AuthManager.shared
    @StateObject private var settings = AppSettings.shared
    @StateObject private var cache = FileCache.shared
    @StateObject private var drive = DriveService.shared
    @StateObject private var sync = SyncManager.shared
    @StateObject private var biometrics = BiometricManager.shared

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(auth)
                .environmentObject(settings)
                .environmentObject(cache)
                .environmentObject(drive)
                .environmentObject(sync)
                .environmentObject(biometrics)
        }
    }
}
