use totp_rs::{Algorithm, Secret, TOTP};

pub fn generate_secret() -> String {
    Secret::generate_secret().to_encoded().to_string()
}

pub fn generate_otpauth_uri(secret: &str, email: &str, issuer: &str) -> Result<String, String> {
    let secret_bytes = Secret::Encoded(secret.to_string())
        .to_bytes()
        .map_err(|e| format!("Invalid TOTP secret: {e}"))?;
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some(issuer.to_string()),
        email.to_string(),
    )
    .map_err(|e| format!("Failed to create TOTP: {e}"))?;
    Ok(totp.get_url())
}

pub fn verify_totp(secret: &str, code: &str) -> bool {
    let Ok(secret_bytes) = Secret::Encoded(secret.to_string()).to_bytes() else {
        return false;
    };
    let Ok(totp) = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        None,
        String::from("user"),
    ) else {
        return false;
    };
    totp.check_current(code).unwrap_or(false)
}
