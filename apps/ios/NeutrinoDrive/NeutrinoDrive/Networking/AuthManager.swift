import Foundation

final class AuthManager: ObservableObject {
    static let shared = AuthManager()

    @Published private(set) var isAuthenticated: Bool = false
    @Published private(set) var profile: UserProfile?

    private let keychain = KeychainStore.shared

    private let accessTokenKey = "neutrino.access_token"
    private let refreshTokenKey = "neutrino.refresh_token"
    private let expiresAtKey = "neutrino.expires_at"

    private init() {
        isAuthenticated = accessToken != nil
    }

    var accessToken: String? {
        keychain.get(accessTokenKey)
    }

    var refreshToken: String? {
        keychain.get(refreshTokenKey)
    }

    var expiresAt: Date? {
        guard let raw = keychain.get(expiresAtKey), let value = TimeInterval(raw) else { return nil }
        return Date(timeIntervalSince1970: value)
    }

    func storeTokens(_ tokens: AuthTokens) {
        keychain.set(tokens.accessToken, forKey: accessTokenKey)
        keychain.set(tokens.refreshToken, forKey: refreshTokenKey)
        let expiresAt = Date().addingTimeInterval(TimeInterval(tokens.expiresIn))
        keychain.set(String(expiresAt.timeIntervalSince1970), forKey: expiresAtKey)
        isAuthenticated = true
    }

    func clearTokens() {
        keychain.remove(accessTokenKey)
        keychain.remove(refreshTokenKey)
        keychain.remove(expiresAtKey)
        isAuthenticated = false
        profile = nil
    }

    func shouldRefreshSoon() -> Bool {
        guard let expiresAt else { return true }
        return expiresAt.timeIntervalSinceNow < 120
    }

    func setProfile(_ profile: UserProfile) {
        self.profile = profile
    }
}
