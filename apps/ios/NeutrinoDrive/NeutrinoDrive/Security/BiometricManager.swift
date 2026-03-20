import Foundation
import LocalAuthentication

@MainActor
final class BiometricManager: ObservableObject {
    static let shared = BiometricManager()

    @Published private(set) var isUnlocked = false
    @Published private(set) var isAuthenticating = false
    @Published private(set) var isAvailable = false
    @Published private(set) var lastError: String?
    @Published private(set) var biometryType: LABiometryType = .none

    private init() {}

    func reset() {
        isUnlocked = false
    }

    func setUnlocked(_ value: Bool) {
        isUnlocked = value
    }

    func clearError() {
        lastError = nil
    }

    @discardableResult
    func refreshAvailability() -> Bool {
        let context = LAContext()
        var error: NSError?
        let canEvaluate = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
        biometryType = context.biometryType
        isAvailable = canEvaluate
        return canEvaluate
    }

    @discardableResult
    func authenticate(reason: String = "Unlock Neutrino Drive") async -> Bool {
        guard !isAuthenticating else { return false }

        clearError()
        isAuthenticating = true
        defer { isAuthenticating = false }

        let context = LAContext()
        var error: NSError?
        let canEvaluate = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
        biometryType = context.biometryType
        isAvailable = canEvaluate

        guard canEvaluate else {
            lastError = message(for: error)
            isUnlocked = shouldAllowAccessWithoutBiometrics(error)
            return false
        }

        do {
            let success = try await evaluatePolicy(using: context, reason: reason)
            if success {
                isUnlocked = true
                lastError = nil
                return true
            }
            isUnlocked = false
            lastError = "Authentication failed"
            return false
        } catch {
            isUnlocked = false
            lastError = message(for: error)
            return false
        }
    }

    var biometryDisplayName: String {
        switch biometryType {
        case .faceID:
            return "Face ID"
        case .touchID:
            return "Touch ID"
        default:
            return "Face/Touch ID"
        }
    }

    var biometryIconName: String {
        switch biometryType {
        case .faceID:
            return "faceid"
        case .touchID:
            return "touchid"
        default:
            return "lock.shield"
        }
    }

    private func evaluatePolicy(using context: LAContext, reason: String) async throws -> Bool {
        try await withCheckedThrowingContinuation { continuation in
            context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: reason) { success, error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: success)
                }
            }
        }
    }

    private func shouldAllowAccessWithoutBiometrics(_ error: NSError?) -> Bool {
        guard let code = LAError.Code(rawValue: error?.code ?? -1) else { return false }
        switch code {
        case .biometryNotAvailable, .biometryNotEnrolled, .passcodeNotSet:
            return true
        default:
            return false
        }
    }

    private func message(for error: Error?) -> String {
        guard let error else { return "Authentication failed" }
        let nsError = error as NSError
        guard let code = LAError.Code(rawValue: nsError.code) else {
            return nsError.localizedDescription
        }

        switch code {
        case .authenticationFailed:
            return "Biometric authentication failed."
        case .userCancel:
            return "\(biometryDisplayName) was canceled."
        case .userFallback:
            return "Passcode fallback is not enabled for biometric unlock."
        case .systemCancel:
            return "\(biometryDisplayName) was interrupted."
        case .passcodeNotSet:
            return "Set a device passcode before enabling \(biometryDisplayName)."
        case .biometryNotAvailable:
            return "\(biometryDisplayName) is not available on this device."
        case .biometryNotEnrolled:
            return "Set up \(biometryDisplayName) in Settings before enabling it here."
        case .biometryLockout:
            return "\(biometryDisplayName) is locked. Unlock your device and try again."
        case .invalidContext:
            return "Biometric authentication is temporarily unavailable."
        default:
            return nsError.localizedDescription
        }
    }
}
