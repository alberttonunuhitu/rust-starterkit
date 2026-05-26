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
    keypair: AsymmetricKeyPair<V4>,
    issuer: String,
    audience: String,
    expiration: String,
    refresh_expiration: String,
    environment: String,
}

static STATE: OnceLock<PasetoState> = OnceLock::new();

pub fn init(config: &AppConfig) -> Result<()> {
    let secret =
        parse_secret_key(&config.token.secret_key).context("APP_TOKEN__SECRET_KEY is invalid")?;

    let public = AsymmetricPublicKey::<V4>::try_from(&secret)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Failed to derive PASETO v4 public key")?;

    STATE
        .set(PasetoState {
            keypair: AsymmetricKeyPair { public, secret },
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

    public::sign(&state.keypair.secret, &claims, Some(&footer), None)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Failed to create access token")
}

pub fn create_refresh_token(sub: Uuid, email: &str) -> Result<String> {
    let state = state()?;
    let claims = build_claims(state, sub, email, &state.refresh_expiration)?;
    let footer = build_footer(state, "refresh")?;

    public::sign(&state.keypair.secret, &claims, Some(&footer), None)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Failed to create refresh token")
}

fn verify_token_with_type(token: &str, expected_type: Option<&str>) -> Result<Claims> {
    let state = state()?;

    let untrusted = UntrustedToken::<Public, V4>::try_from(token)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Invalid PASETO token format")?;

    let rules = ClaimsValidationRules::new();
    let trusted = public::verify(&state.keypair.public, &untrusted, &rules, None, None)
        .map_err(|e| anyhow::anyhow!("{e:?}"))
        .context("Token verification failed (invalid signature, expired, or tampered)")?;

    let footer_str = String::from_utf8_lossy(trusted.footer());
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&footer_str) {
        if let Some(env) = v.get("env").and_then(|e| e.as_str())
            && env != state.environment
        {
            anyhow::bail!(
                "Token environment mismatch: got '{env}', expected '{}'",
                state.environment
            );
        }

        if let Some(expected) = expected_type {
            match v.get("type").and_then(|t| t.as_str()) {
                Some(t) if t == expected => {}
                Some(t) => anyhow::bail!("Token type mismatch: got '{t}', expected '{expected}'"),
                None => anyhow::bail!("Token footer missing 'type' field"),
            }
        }
    }

    trusted
        .payload_claims()
        .cloned()
        .context("Token has no payload claims")
}

pub fn verify_token(token: &str) -> Result<Claims> {
    verify_token_with_type(token, None)
}

pub fn verify_access_token(token: &str) -> Result<Claims> {
    verify_token_with_type(token, Some("access"))
}

pub fn verify_refresh_token(token: &str) -> Result<Claims> {
    verify_token_with_type(token, Some("refresh"))
}
