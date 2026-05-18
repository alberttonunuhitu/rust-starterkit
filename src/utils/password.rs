use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

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

pub fn hash(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = get_argon2();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| ApiError::InternalServerError)?
        .to_string();

    Ok(password_hash)
}

pub fn verify(password: &str, hashed_password: &str) -> Result<bool, ApiError> {
    let parsed_hash =
        PasswordHash::new(hashed_password).map_err(|_| ApiError::InternalServerError)?;

    let argon2 = get_argon2();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
