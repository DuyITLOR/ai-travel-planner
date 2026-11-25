//! Bearer token check for Supabase Auth access tokens (HS256, signed with the
//! project's JWT secret). The caller's id is the token's `sub`.

use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    problem::{Problem, ProblemType},
};

/// Audience Supabase puts in tokens of signed-in users.
pub const AUDIENCE: &str = "authenticated";

#[derive(Clone)]
pub struct JwtVerifier {
    key: Option<DecodingKey>,
    validation: Validation,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

impl JwtVerifier {
    /// `None` or an empty secret: every token is rejected (the api still starts).
    pub fn new(secret: Option<&str>) -> Self {
        let key = secret
            .filter(|s| !s.is_empty())
            .and_then(|s| DecodingKey::from_secret(s.as_bytes()).ok());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[AUDIENCE]);
        validation.set_required_spec_claims(&["exp", "sub", "aud"]);
        validation.leeway = 0;
        Self { key, validation }
    }

    pub fn from_env() -> Self {
        let secret = std::env::var("SUPABASE_JWT_SECRET").ok();
        if secret.as_deref().is_none_or(str::is_empty) {
            tracing::warn!("SUPABASE_JWT_SECRET is not set: every authenticated request gets 401");
        }
        Self::new(secret.as_deref())
    }

    pub fn verify(&self, token: &str) -> Option<Uuid> {
        let key = self.key.as_ref()?;
        let data = decode::<Claims>(token, key, &self.validation).ok()?;
        Uuid::parse_str(&data.claims.sub).ok()
    }
}

/// The signed-in caller. Extracting it answers `401 unauthenticated` on any failure.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser(pub Uuid);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Problem> {
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            // The scheme name is case-insensitive (RFC 9110, section 11.1).
            .and_then(|v| v.split_once(' '))
            .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
            .map(|(_, token)| token.trim())
            .filter(|t| !t.is_empty());
        let Some(token) = token else {
            return Err(Problem::new(
                ProblemType::Unauthenticated,
                "Send a Supabase access token as `Authorization: Bearer <token>`.",
            ));
        };
        state.auth.verify(token).map(AuthUser).ok_or_else(|| {
            Problem::new(
                ProblemType::Unauthenticated,
                "The access token is invalid or expired. Sign in again.",
            )
        })
    }
}
