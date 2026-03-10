import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var auth: AuthManager
    @EnvironmentObject private var settings: AppSettings
    @EnvironmentObject private var drive: DriveService
    @EnvironmentObject private var biometrics: BiometricManager
    @Environment(\.scenePhase) private var scenePhase

    @State private var loading = false
    @State private var errorMessage: String?

    var body: some View {
        Group {
            if auth.isAuthenticated {
                if settings.biometricEnabled && !biometrics.isUnlocked {
                    LockView()
                        .onAppear { biometrics.authenticate() }
                } else {
                    DriveHomeView()
                        .task {
                            await bootstrap()
                        }
                }
            } else {
                AuthView()
            }
        }
        .alert("Error", isPresented: Binding(get: { errorMessage != nil }, set: { _ in errorMessage = nil })) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage ?? "")
        }
        .onChange(of: scenePhase) { phase in
            if phase == .background {
                biometrics.reset()
            }
        }
        .onChange(of: settings.biometricEnabled) { enabled in
            if enabled {
                biometrics.reset()
                biometrics.authenticate()
            } else {
                biometrics.setUnlocked(true)
            }
        }
    }

    private func bootstrap() async {
        guard !loading else { return }
        loading = true
        do {
            try await drive.loadProfile()
        } catch {
            errorMessage = error.localizedDescription
        }
        loading = false
    }
}
