import Foundation
import LocalAuthentication

final class BiometricManager: ObservableObject {
    static let shared = BiometricManager()

    @Published private(set) var isUnlocked = false
    @Published private(set) var lastError: String?

    private init() {}

    func reset() {
        isUnlocked = false
    }

    func setUnlocked(_ value: Bool) {
        isUnlocked = value
    }

    func authenticate(reason: String = "Unlock Neutrino Drive") {
        let context = LAContext()
        var error: NSError?
        guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {
            lastError = error?.localizedDescription ?? "Biometrics unavailable"
            isUnlocked = true
            return
        }

        context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: reason) { success, evalError in
            DispatchQueue.main.async {
                if success {
                    self.isUnlocked = true
                    self.lastError = nil
                } else {
                    self.isUnlocked = false
                    self.lastError = evalError?.localizedDescription ?? "Authentication failed"
                }
            }
        }
    }
}
