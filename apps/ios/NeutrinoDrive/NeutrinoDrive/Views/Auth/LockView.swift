import SwiftUI

struct LockView: View {
    @EnvironmentObject private var biometrics: BiometricManager

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: biometrics.biometryIconName)
                .font(.system(size: 48))
                .foregroundColor(.accentColor)
            Text("\(biometrics.biometryDisplayName) Required")
                .font(.title2)
            Text("Unlock Neutrino Drive with \(biometrics.biometryDisplayName).")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
            if let error = biometrics.lastError {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
            }
            Button(biometrics.isAuthenticating ? "Checking..." : "Use \(biometrics.biometryDisplayName)") {
                Task {
                    await biometrics.authenticate()
                }
            }
            .buttonStyle(.borderedProminent)
            .disabled(biometrics.isAuthenticating)
        }
        .padding()
        .task {
            biometrics.refreshAvailability()
            await biometrics.authenticate()
        }
    }
}
