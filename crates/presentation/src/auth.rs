use std::sync::Arc;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::error::AppError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrincipalKind {
    Admin,
    Device,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Principal {
    Admin { subject: String },
    Device { client_id: String },
}

impl Principal {
    pub fn kind(&self) -> PrincipalKind {
        match self {
            Self::Admin { .. } => PrincipalKind::Admin,
            Self::Device { .. } => PrincipalKind::Device,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthorizationPolicy {
    allowed_kinds: Vec<PrincipalKind>,
}

impl AuthorizationPolicy {
    pub fn only(kind: PrincipalKind) -> Self {
        Self::any_of([kind])
    }

    pub fn any_of<I>(kinds: I) -> Self
    where
        I: IntoIterator<Item = PrincipalKind>,
    {
        Self {
            allowed_kinds: kinds.into_iter().collect(),
        }
    }

    pub fn allows(&self, principal: &Principal) -> bool {
        self.allowed_kinds.contains(&principal.kind())
    }
}

#[derive(Clone)]
pub struct Authenticator {
    client: Client,
    issuer: String,
    audience: String,
    dashboard_client_id: String,
    jwks_uri: String,
    jwks: Arc<RwLock<Option<JwkSet>>>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid authorization token")]
    InvalidToken,
    #[error("failed to fetch Auth0 signing keys")]
    JwksRequest(#[source] reqwest::Error),
    #[error("Auth0 signing keys response was invalid")]
    JwksResponse(#[source] reqwest::Error),
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(default)]
    azp: Option<String>,
    #[serde(default)]
    permissions: Vec<String>,
}

impl Authenticator {
    /// `dashboard_client_id` identifies the only user-facing client trusted as an admin principal.
    /// Its login flow must enforce the GitHub organization membership Action in Auth0.
    pub fn new(
        client: Client,
        issuer: String,
        audience: String,
        dashboard_client_id: String,
    ) -> Self {
        let issuer = format!("{}/", issuer.trim_end_matches('/'));
        let jwks_uri = format!("{issuer}.well-known/jwks.json");
        Self {
            client,
            issuer,
            audience,
            dashboard_client_id,
            jwks_uri,
            jwks: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn authenticate(&self, token: &str) -> Result<Principal, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        if header.alg != Algorithm::RS256 {
            return Err(AuthError::InvalidToken);
        }

        let jwk = self.find_jwk(header.kid.as_deref()).await?;
        let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|_| AuthError::InvalidToken)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(std::slice::from_ref(&self.issuer));
        validation.set_audience(std::slice::from_ref(&self.audience));

        let claims = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::InvalidToken)?
            .claims;
        principal_from_claims(claims, &self.dashboard_client_id)
    }

    async fn find_jwk(&self, kid: Option<&str>) -> Result<jsonwebtoken::jwk::Jwk, AuthError> {
        if let Some(jwk) = self
            .jwks
            .read()
            .await
            .as_ref()
            .and_then(|set| find_jwk(set, kid))
        {
            return Ok(jwk.clone());
        }

        let jwks = self.fetch_jwks().await?;
        let jwk = find_jwk(&jwks, kid).ok_or(AuthError::InvalidToken)?.clone();
        *self.jwks.write().await = Some(jwks);
        Ok(jwk)
    }

    async fn fetch_jwks(&self) -> Result<JwkSet, AuthError> {
        self.client
            .get(&self.jwks_uri)
            .send()
            .await
            .map_err(AuthError::JwksRequest)?
            .error_for_status()
            .map_err(AuthError::JwksRequest)?
            .json()
            .await
            .map_err(AuthError::JwksResponse)
    }
}

pub async fn middleware(
    axum::extract::State(authenticator): axum::extract::State<Authenticator>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| {
            AppError::new(
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_owned(),
            )
        })?;

    let principal = authenticator.authenticate(token).await.map_err(|_| {
        AppError::new(
            StatusCode::UNAUTHORIZED,
            "Invalid authentication token".to_owned(),
        )
    })?;
    request.extensions_mut().insert(principal);
    Ok(next.run(request).await)
}

pub async fn authorize(
    axum::extract::State(policy): axum::extract::State<AuthorizationPolicy>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match request.extensions().get::<Principal>() {
        Some(principal) if policy.allows(principal) => Ok(next.run(request).await),
        Some(_) => Err(AppError::new(StatusCode::FORBIDDEN, "Forbidden".to_owned())),
        None => Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_owned(),
        )),
    }
}

fn find_jwk<'a>(set: &'a JwkSet, kid: Option<&str>) -> Option<&'a jsonwebtoken::jwk::Jwk> {
    set.keys
        .iter()
        .find(|jwk| jwk.common.key_id.as_deref() == kid)
}

fn principal_from_claims(
    claims: Claims,
    dashboard_client_id: &str,
) -> Result<Principal, AuthError> {
    if claims
        .permissions
        .iter()
        .any(|permission| permission == "device")
        && claims.sub.ends_with("@clients")
    {
        let client_id = claims
            .azp
            .or_else(|| claims.sub.strip_suffix("@clients").map(ToOwned::to_owned))
            .ok_or(AuthError::InvalidToken)?;
        return Ok(Principal::Device { client_id });
    }

    if claims.azp.as_deref() == Some(dashboard_client_id) {
        return Ok(Principal::Admin {
            subject: claims.sub,
        });
    }

    Err(AuthError::InvalidToken)
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use super::{Authenticator, Claims, Principal, PrincipalKind, principal_from_claims};

    #[test]
    fn normalizes_issuer_for_jwt_validation_and_jwks_lookup() {
        let authenticator = Authenticator::new(
            Client::new(),
            "https://example.auth0.com".into(),
            "https://api.example.com".into(),
            "dashboard-client-id".into(),
        );

        assert_eq!(authenticator.issuer, "https://example.auth0.com/");
        assert_eq!(
            authenticator.jwks_uri,
            "https://example.auth0.com/.well-known/jwks.json"
        );
    }

    #[test]
    fn converts_device_claims_to_device_principal() {
        let principal = principal_from_claims(
            Claims {
                sub: "client-id@clients".into(),
                azp: Some("client-id".into()),
                permissions: vec!["device".into()],
            },
            "dashboard-client-id",
        )
        .unwrap();

        assert_eq!(
            principal,
            Principal::Device {
                client_id: "client-id".into()
            }
        );
    }

    #[test]
    fn converts_admin_claims_to_admin_principal() {
        let principal = principal_from_claims(
            Claims {
                sub: "auth0|user-id".into(),
                azp: Some("dashboard-client-id".into()),
                permissions: vec![],
            },
            "dashboard-client-id",
        )
        .unwrap();

        assert_eq!(
            principal,
            Principal::Admin {
                subject: "auth0|user-id".into()
            }
        );
    }

    #[test]
    fn rejects_device_permission_for_non_client_subject() {
        let result = principal_from_claims(
            Claims {
                sub: "auth0|user-id".into(),
                azp: None,
                permissions: vec!["device".into()],
            },
            "dashboard-client-id",
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_admin_claims_from_another_client() {
        let result = principal_from_claims(
            Claims {
                sub: "auth0|user-id".into(),
                azp: Some("another-client-id".into()),
                permissions: vec!["admin".into()],
            },
            "dashboard-client-id",
        );

        assert!(result.is_err());
    }

    #[test]
    fn policy_can_allow_multiple_principal_kinds() {
        let policy =
            super::AuthorizationPolicy::any_of([PrincipalKind::Admin, PrincipalKind::Device]);
        let principal = Principal::Admin {
            subject: "auth0|user-id".into(),
        };

        assert!(policy.allows(&principal));
    }

    #[test]
    fn admin_policy_rejects_device_principal() {
        let policy = super::AuthorizationPolicy::only(PrincipalKind::Admin);
        let principal = Principal::Device {
            client_id: "device-client".into(),
        };

        assert!(!policy.allows(&principal));
    }
}
