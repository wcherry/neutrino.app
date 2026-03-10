import SwiftUI

struct AuthView: View {
    @EnvironmentObject private var settings: AppSettings
    @EnvironmentObject private var drive: DriveService

    @State private var isRegister = false
    @State private var serverUrl = ""
    @State private var name = ""
    @State private var email = ""
    @State private var password = ""
    @State private var isLoading = false
    @State private var errorMessage: String?

    var body: some View {
        NavigationStack {
            VStack(spacing: 20) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Server URL")
                        .font(.headline)
                    TextField("https://api.example.com", text: $serverUrl)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .keyboardType(.URL)
                        .textFieldStyle(.roundedBorder)
                }

                Picker("Mode", selection: $isRegister) {
                    Text("Sign In").tag(false)
                    Text("Create Account").tag(true)
                }
                .pickerStyle(.segmented)

                if isRegister {
                    TextField("Name", text: $name)
                        .textFieldStyle(.roundedBorder)
                }

                TextField("Email", text: $email)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .keyboardType(.emailAddress)
                    .textFieldStyle(.roundedBorder)

                SecureField("Password", text: $password)
                    .textFieldStyle(.roundedBorder)

                Button(action: submit) {
                    if isLoading {
                        ProgressView()
                            .progressViewStyle(.circular)
                    } else {
                        Text(isRegister ? "Create Account" : "Sign In")
                            .frame(maxWidth: .infinity)
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(isLoading)

                Spacer()
            }
            .padding()
            .navigationTitle("Neutrino Drive")
            .alert("Error", isPresented: Binding(get: { errorMessage != nil }, set: { _ in errorMessage = nil })) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage ?? "")
            }
            .onAppear {
                serverUrl = settings.baseUrl
            }
        }
    }

    private func submit() {
        Task {
            isLoading = true
            errorMessage = nil
            do {
                let normalized = normalizeBaseURL(serverUrl)
                guard !normalized.isEmpty else {
                    errorMessage = "Server URL is required"
                    isLoading = false
                    return
                }
                settings.baseUrl = normalized
                if isRegister {
                    try await drive.register(name: name.trimmingCharacters(in: .whitespaces), email: email.trimmingCharacters(in: .whitespaces), password: password)
                } else {
                    try await drive.login(email: email.trimmingCharacters(in: .whitespaces), password: password)
                }
            } catch {
                errorMessage = error.localizedDescription
            }
            isLoading = false
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
