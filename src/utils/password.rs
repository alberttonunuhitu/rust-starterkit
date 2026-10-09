use std::sync::OnceLock;

use actix_web::rt::task::spawn_blocking;
use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use sha2::{Digest, Sha256};

use crate::common::{
    constants::{ARGON2_MEMORY_COST, ARGON2_PARALLELISM, ARGON2_TIME_COST},
    error::ApiError,
};

fn get_argon2() -> Argon2<'static> {
    let params = Params::new(
        ARGON2_MEMORY_COST,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        None,
    )
    .expect("Invalid Argon2 parameters");

    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
}

fn hash_blocking(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);

    get_argon2()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| {
            tracing::error!(event = "utils.password.hash_failed", error = %e);
            ApiError::InternalServerError
        })
}

fn verify_blocking(password: &str, hashed_password: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(hashed_password).map_err(|e| {
        tracing::error!(event = "utils.password.parse_hash_failed", error = %e);
        ApiError::InternalServerError
    })?;

    Ok(get_argon2()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Hash a password with Argon2id on the blocking thread pool.
pub async fn hash(password: &str) -> Result<String, ApiError> {
    let password = password.to_owned();

    spawn_blocking(move || hash_blocking(&password))
        .await
        .map_err(|e| {
            tracing::error!(event = "utils.password.task_failed", error = %e);
            ApiError::InternalServerError
        })?
}

/// Verify a password against an Argon2 hash on the blocking thread pool.
pub async fn verify(password: &str, hashed_password: &str) -> Result<bool, ApiError> {
    let password = password.to_owned();
    let hashed_password = hashed_password.to_owned();

    spawn_blocking(move || verify_blocking(&password, &hashed_password))
        .await
        .map_err(|e| {
            tracing::error!(event = "utils.password.task_failed", error = %e);
            ApiError::InternalServerError
        })?
}

/// Run a verification with the same cost as a real one, so a login for an
/// unknown email takes as long as a login with a wrong password.
pub async fn verify_dummy(password: &str) {
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();

    let password = password.to_owned();

    let _ = spawn_blocking(move || {
        let dummy = DUMMY_HASH.get_or_init(|| {
            hash_blocking("dummy-password-for-timing").expect("Failed to hash dummy password")
        });
        verify_blocking(&password, dummy)
    })
    .await;
}

/// Hash a high-entropy token (e.g. a refresh token) with SHA-256.
///
/// Refresh tokens are long random strings, so a slow KDF like Argon2 adds cost
/// without adding security.
pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Compare a token against a hash produced by [`hash_token`] in constant time.
pub fn verify_token(token: &str, hashed_token: &str) -> bool {
    let computed = hash_token(token);

    computed.len() == hashed_token.len()
        && computed
            .bytes()
            .zip(hashed_token.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_hash_roundtrip() {
        let hashed = hash_token("some-refresh-token");

        assert!(verify_token("some-refresh-token", &hashed));
        assert!(!verify_token("other-refresh-token", &hashed));
        assert!(!verify_token("some-refresh-token", "not-a-hash"));
    }

    #[test]
    fn password_hash_roundtrip() {
        let hashed = hash_blocking("secret").unwrap();

        assert!(verify_blocking("secret", &hashed).unwrap());
        assert!(!verify_blocking("wrong", &hashed).unwrap());
    }
}
