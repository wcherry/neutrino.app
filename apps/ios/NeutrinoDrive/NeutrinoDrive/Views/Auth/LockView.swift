import SwiftUI

struct LockView: View {
    @EnvironmentObject private var biometrics: BiometricManager

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "faceid")
                .font(.system(size: 48))
                .foregroundColor(.accentColor)
            Text("Face ID Required")
                .font(.title2)
            if let error = biometrics.lastError {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Button("Unlock") {
                biometrics.authenticate()
            }
            .buttonStyle(.borderedProminent)
        }
        .padding()
    }
}
