use chrono::Utc;
use pasetors::{
    Public,
    claims::{Claims, ClaimsValidationRules},
    footer::Footer,
    keys::{AsymmetricKeyPair, AsymmetricPublicKey, AsymmetricSecretKey},
    public,
    token::UntrustedToken,
    version4::V4,
};
use std::sync::OnceLock;
use uuid::Uuid;

use anyhow::{Context, Result};

use crate::{config::AppConfig, utils::duration::parse_token_expiration};

const KEY_VERSION: &str = "v1";

struct PasetoState {
    access_keypair: AsymmetricKeyPair<V4>,
    refresh_keypair: AsymmetricKeyPair<V4>,
    issuer: String,
    audience: String,
    expiration: String,
    refresh_expiration: String,
    environment: String,
}

static STATE: OnceLock<PasetoState> = OnceLock::new();

pub fn init(config: &AppConfig) -> Result<()> {
    let access_keypair =
        load_keypair(&config.token.secret_key).context("APP_TOKEN__SECRET_KEY is invalid")?;
    let refresh_keypair = load_keypair(&config.token.refresh_secret_key)
        .context("APP_TOKEN__REFRESH_SECRET_KEY is invalid")?;

    if access_keypair.secret.as_bytes() == refresh_keypair.secret.as_bytes() {
        anyhow::bail!("APP_TOKEN__SECRET_KEY and APP_TOKEN__REFRESH_SECRET_KEY must be different");
    }

    STATE
        .set(PasetoState {
            access_keypair,
            refresh_keypair,
            issuer: config.token.issuer.clone(),
            audience: config.token.audience.clone(),
            expiration: config.token.expiration.clone(),
            refresh_expiration: config.token.refresh_expiration.clone(),
            environment: config.app.environment.clone(),
        })
        .map_err(|_| anyhow::anyhow!("PASETO already initialized"))?;

    tracing::info!(event = "utils.paseto.init", "PASETO v4.public initialized");

    Ok(())
}

fn state() -> Result<&'static PasetoState> {
    STATE
        .get()
        .context("PASETO not initialized (call paseto::init first)")
}

fn load_keypair(secret_key: &str) -> Result<AsymmetricKeyPair<V4>> {
    let secret = parse_secret_key(secret_key)?;

    let public = AsymmetricPublicKey::<V4>::try_from(&secret)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Failed to derive PASETO v4 public key")?;

    Ok(AsymmetricKeyPair { public, secret })
}

/// Load a PASETO v4 secret key from PASERK (`k4.secret.…`) or hex (`seed || public`, 64 bytes).
fn parse_secret_key(secret_key: &str) -> Result<AsymmetricSecretKey<V4>> {
    let trimmed = secret_key.trim().trim_matches('"');

    if trimmed.starts_with("k4.secret.") {
        return AsymmetricSecretKey::<V4>::try_from(trimmed).map_err(|e| anyhow::anyhow!("{e:?}"));
    }

    let bytes = hex::decode(trimmed).context("expected `k4.secret.…` or 128 hex chars")?;

    AsymmetricSecretKey::<V4>::from(&bytes)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("invalid Ed25519 secret (run `cargo run --bin generate-paseto-keys`)")
}

fn build_footer(state: &PasetoState, token_type: &str) -> Result<Footer> {
    let mut footer = Footer::new();
    footer.add_additional("ver", KEY_VERSION)?;
    footer.add_additional("env", state.environment.as_str())?;
    footer.add_additional("type", token_type)?;
    Ok(footer)
}

fn build_claims(state: &PasetoState, sub: Uuid, email: &str, expiration: &str) -> Result<Claims> {
    let exp = Utc::now() + parse_token_expiration(expiration);
    let mut claims = Claims::new()?;
    claims.expiration(&exp.to_rfc3339())?;
    claims.subject(&sub.to_string())?;
    claims.issuer(&state.issuer)?;
    claims.audience(&state.audience)?;
    claims.token_identifier(&Uuid::new_v4().to_string())?;
    claims.add_additional("email", email)?;
    Ok(claims)
}

pub fn create_access_token(sub: Uuid, email: &str) -> Result<String> {
    let state = state()?;
    let claims = build_claims(state, sub, email, &state.expiration)?;
    let footer = build_footer(state, "access")?;

    public::sign(&state.access_keypair.secret, &claims, Some(&footer), None)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Failed to create access token")
}

pub fn create_refresh_token(sub: Uuid, email: &str) -> Result<String> {
    let state = state()?;
    let claims = build_claims(state, sub, email, &state.refresh_expiration)?;
    let footer = build_footer(state, "refresh")?;

    public::sign(&state.refresh_keypair.secret, &claims, Some(&footer), None)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Failed to create refresh token")
}

fn verify_token_with_type(
    token: &str,
    public_key: &AsymmetricPublicKey<V4>,
    expected_type: &str,
) -> Result<Claims> {
    let state = state()?;

    let untrusted = UntrustedToken::<Public, V4>::try_from(token)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Invalid PASETO token format")?;

    let mut rules = ClaimsValidationRules::new();
    rules.validate_issuer_with(&state.issuer);
    rules.validate_audience_with(&state.audience);

    let trusted = public::verify(public_key, &untrusted, &rules, None, None)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Token verification failed (invalid signature, expired, or tampered)")?;

    let footer: serde_json::Value =
        serde_json::from_slice(trusted.footer()).context("Token footer is not valid JSON")?;

    match footer.get("env").and_then(|e| e.as_str()) {
        Some(env) if env == state.environment => {}
        Some(env) => anyhow::bail!(
            "Token environment mismatch: got '{env}', expected '{}'",
            state.environment
        ),
        None => anyhow::bail!("Token footer missing 'env' field"),
    }

    match footer.get("type").and_then(|t| t.as_str()) {
        Some(t) if t == expected_type => {}
        Some(t) => anyhow::bail!("Token type mismatch: got '{t}', expected '{expected_type}'"),
        None => anyhow::bail!("Token footer missing 'type' field"),
    }

    trusted
        .payload_claims()
        .cloned()
        .context("Token has no payload claims")
}

pub fn verify_access_token(token: &str) -> Result<Claims> {
    let state = state()?;
    verify_token_with_type(token, &state.access_keypair.public, "access")
}

pub fn verify_refresh_token(token: &str) -> Result<Claims> {
    let state = state()?;
    verify_token_with_type(token, &state.refresh_keypair.public, "refresh")
}

#[cfg(test)]
mod tests {
    use pasetors::{keys::Generate, paserk::FormatAsPaserk};

    use super::*;

    fn generate_secret() -> String {
        let pair = AsymmetricKeyPair::<V4>::generate().unwrap();
        let mut out = String::new();
        pair.secret.fmt(&mut out).unwrap();
        out
    }

    // `STATE` is process-global, so the whole flow lives in a single test.
    #[test]
    fn access_and_refresh_tokens_use_separate_keys() {
        let mut config = AppConfig::default();
        config.token.secret_key = generate_secret();
        config.token.refresh_secret_key = generate_secret();
        init(&config).unwrap();

        let user_id = Uuid::now_v7();
        let access = create_access_token(user_id, "user@example.com").unwrap();
        let refresh = create_refresh_token(user_id, "user@example.com").unwrap();

        let claims = verify_access_token(&access).unwrap();
        assert_eq!(
            claims.get_claim("sub").and_then(|v| v.as_str()),
            Some(user_id.to_string().as_str())
        );
        assert!(verify_refresh_token(&refresh).is_ok());

        assert!(verify_access_token(&refresh).is_err());
        assert!(verify_refresh_token(&access).is_err());
    }
}
