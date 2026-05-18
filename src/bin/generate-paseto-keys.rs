//! Generate PASETO v4 Ed25519 key pairs for `.env`.
//!
//! Usage: cargo run --bin generate-paseto-keys

use pasetors::{
    keys::{AsymmetricKeyPair, Generate},
    paserk::FormatAsPaserk,
    version4::V4,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hex_mode = std::env::args().any(|a| a == "--hex");

    let (access_key, refresh_key) = (
        to_env_value(AsymmetricKeyPair::<V4>::generate()?, hex_mode),
        to_env_value(AsymmetricKeyPair::<V4>::generate()?, hex_mode),
    );

    println!("APP_TOKEN__SECRET_KEY={access_key}");
    println!("APP_TOKEN__REFRESH_SECRET_KEY={refresh_key}");

    Ok(())
}

fn to_env_value(pair: AsymmetricKeyPair<V4>, hex_mode: bool) -> String {
    if hex_mode {
        hex::encode(pair.secret.as_bytes())
    } else {
        let mut out = String::new();
        pair.secret.fmt(&mut out).expect("PASERK format failed");
        out
    }
}
