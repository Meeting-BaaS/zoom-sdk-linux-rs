use chrono::Utc;
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

const TOKEN_VALIDATION_HOURS: i64 = 6;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Payload {
    appKey: String,
    iat: i64,
    exp: i64,
    tokenExp: i64,
}

/// Generate a zoom JWT Token.
#[tracing::instrument(ret, level = "debug", skip(sdk_secret))]
pub fn generate_jwt(sdk_key: &str, sdk_secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    let header = Header {
        algorithm: AlgorithmType::Hs256,
        ..Default::default()
    };

    let now = Utc::now().timestamp();
    let payload = Payload {
        appKey: sdk_key.to_string(),
        iat: now,
        exp: now + (3600 * TOKEN_VALIDATION_HOURS),
        tokenExp: now + 86400,
    };

    let key: Hmac<Sha256> = Hmac::new_from_slice(sdk_secret.as_bytes())?;

    //tracing::info!("returning token");
    let token = Token::new(header, payload).sign_with_key(&key)?;
    Ok(token.into())
}
