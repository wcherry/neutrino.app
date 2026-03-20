import SwiftUI
import UniformTypeIdentifiers

struct SettingsView: View {
    @EnvironmentObject private var settings: AppSettings
    @EnvironmentObject private var auth: AuthManager
    @EnvironmentObject private var drive: DriveService
    @EnvironmentObject private var sync: SyncManager
    @EnvironmentObject private var biometrics: BiometricManager

    @State private var baseUrl = ""
    @State private var showConfirm = false
    @State private var errorMessage: String?
    @State private var showFolderPicker = false

    var body: some View {
        Form {
            Section("Server") {
                TextField("https://api.example.com", text: $baseUrl)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .keyboardType(.URL)
                Button("Save Server URL") {
                    showConfirm = true
                }
            }

            Section("Account") {
                if let profile = auth.profile {
                    Text(profile.email)
                    Text(profile.name)
                }
                Toggle(biometrics.biometryDisplayName, isOn: biometricBinding)
                    .disabled(!biometricsAvailable)
                if biometricsAvailable {
                    Text("Use \(biometrics.biometryDisplayName) to unlock Neutrino Drive when you return to the app.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else {
                    Text(unavailableBiometricsMessage)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                Button("Sign Out", role: .destructive) {
                    Task { await drive.logout() }
                }
            }

            Section("Sync Sources") {
                Toggle("Sync Recent Photos (30 days)", isOn: $settings.syncPhotosEnabled)

                if settings.syncFolders.isEmpty {
                    Text("No folders selected")
                        .foregroundColor(.secondary)
                } else {
                    ForEach(settings.syncFolders) { folder in
                        HStack {
                            Text(folder.name)
                            Spacer()
                            Button(role: .destructive) {
                                settings.removeSyncFolder(id: folder.id)
                            } label: {
                                Image(systemName: "trash")
                            }
                        }
                    }
                }

                Button("Add Folder") { showFolderPicker = true }
                Button(sync.isSyncing ? "Syncing..." : "Sync Now") {
                    Task { await sync.syncAll() }
                }
                .disabled(sync.isSyncing)

                if let message = sync.lastSyncMessage {
                    Text(message)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Section("Support") {
                NavigationLink("Help", destination: HelpView())
            }
        }
        .navigationTitle("Settings")
        .onAppear {
            baseUrl = settings.baseUrl
            _ = biometrics.refreshAvailability()
        }
        .fileImporter(isPresented: $showFolderPicker, allowedContentTypes: [.folder], allowsMultipleSelection: false) { result in
            switch result {
            case .success(let urls):
                guard let url = urls.first else { return }
                do {
                    let didStart = url.startAccessingSecurityScopedResource()
                    defer { if didStart { url.stopAccessingSecurityScopedResource() } }
                    let bookmark = try SyncBookmark.createBookmark(for: url)
                    let folder = SyncFolder(name: url.lastPathComponent, bookmarkData: bookmark)
                    settings.addSyncFolder(folder)
                } catch {
                    errorMessage = error.localizedDescription
                }
            case .failure(let error):
                errorMessage = error.localizedDescription
            }
        }
        .alert("Change Server URL?", isPresented: $showConfirm) {
            Button("Cancel", role: .cancel) {}
            Button("Change", role: .destructive) { updateBaseUrl() }
        } message: {
            Text("Changing the server URL will sign you out.")
        }
        .alert("Error", isPresented: Binding(get: { errorMessage != nil }, set: { _ in errorMessage = nil })) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage ?? "")
        }
    }

    private func updateBaseUrl() {
        let normalized = normalizeBaseURL(baseUrl)
        settings.baseUrl = normalized
        Task { await drive.logout() }
    }

    private var biometricsAvailable: Bool {
        biometrics.isAvailable
    }

    private var unavailableBiometricsMessage: String {
        biometrics.lastError ?? "Set up Face ID or Touch ID on this device to enable app unlock."
    }

    private var biometricBinding: Binding<Bool> {
        Binding(
            get: { settings.biometricEnabled },
            set: { isEnabled in
                if isEnabled {
                    Task { await enableBiometrics() }
                } else {
                    settings.biometricEnabled = false
                    biometrics.clearError()
                    biometrics.setUnlocked(true)
                }
            }
        )
    }

    private func enableBiometrics() async {
        guard biometrics.refreshAvailability() else {
            errorMessage = unavailableBiometricsMessage
            settings.biometricEnabled = false
            return
        }

        let didAuthenticate = await biometrics.authenticate(reason: "Enable \(biometrics.biometryDisplayName) for Neutrino Drive")
        if didAuthenticate {
            settings.biometricEnabled = true
            biometrics.setUnlocked(true)
            biometrics.clearError()
        } else {
            settings.biometricEnabled = false
            errorMessage = biometrics.lastError ?? "Unable to enable \(biometrics.biometryDisplayName)."
        }
    }

    private func normalizeBaseURL(_ input: String) -> String {
        var value = input.trimmingCharacters(in: .whitespacesAndNewlines)
        if value.isEmpty { return value }
        if !value.hasPrefix("http://") && !value.hasPrefix("https://") {
            value = "https://" + value
        }
        if value.hasSuffix("/") {
            value.removeLast()
        }
        return value
    }
}
