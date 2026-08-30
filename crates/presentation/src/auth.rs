use std::sync::Arc;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrincipalKind {
    Admin,
    Device,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Principal {
    pub kind: PrincipalKind,
    pub subject: String,
    pub client_id: Option<String>,
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
        self.allowed_kinds.contains(&principal.kind)
    }
}

#[derive(Clone)]
pub struct Authenticator {
    client: Client,
    issuer: String,
    audience: String,
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
    pub fn new(client: Client, issuer: String, audience: String) -> Self {
        let issuer = issuer.trim_end_matches('/').to_owned();
        let jwks_uri = format!("{issuer}/.well-known/jwks.json");
        Self {
            client,
            issuer,
            audience,
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
        principal_from_claims(claims)
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
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let principal = authenticator
        .authenticate(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(principal);
    Ok(next.run(request).await)
}

pub async fn authorize(
    axum::extract::State(policy): axum::extract::State<AuthorizationPolicy>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match request.extensions().get::<Principal>() {
        Some(principal) if policy.allows(principal) => Ok(next.run(request).await),
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

fn find_jwk<'a>(set: &'a JwkSet, kid: Option<&str>) -> Option<&'a jsonwebtoken::jwk::Jwk> {
    set.keys
        .iter()
        .find(|jwk| jwk.common.key_id.as_deref() == kid)
}

fn principal_from_claims(claims: Claims) -> Result<Principal, AuthError> {
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
        return Ok(Principal {
            kind: PrincipalKind::Device,
            subject: claims.sub,
            client_id: Some(client_id),
        });
    }

    if claims
        .permissions
        .iter()
        .any(|permission| permission == "admin")
    {
        return Ok(Principal {
            kind: PrincipalKind::Admin,
            subject: claims.sub,
            client_id: None,
        });
    }

    Err(AuthError::InvalidToken)
}

#[cfg(test)]
mod tests {
    use super::{Claims, Principal, PrincipalKind, principal_from_claims};

    #[test]
    fn converts_device_claims_to_device_principal() {
        let principal = principal_from_claims(Claims {
            sub: "client-id@clients".into(),
            azp: Some("client-id".into()),
            permissions: vec!["device".into()],
        })
        .unwrap();

        assert_eq!(
            principal,
            Principal {
                kind: PrincipalKind::Device,
                subject: "client-id@clients".into(),
                client_id: Some("client-id".into())
            }
        );
    }

    #[test]
    fn converts_admin_claims_to_admin_principal() {
        let principal = principal_from_claims(Claims {
            sub: "auth0|user-id".into(),
            azp: None,
            permissions: vec!["admin".into()],
        })
        .unwrap();

        assert_eq!(
            principal,
            Principal {
                kind: PrincipalKind::Admin,
                subject: "auth0|user-id".into(),
                client_id: None
            }
        );
    }

    #[test]
    fn rejects_device_permission_for_non_client_subject() {
        let result = principal_from_claims(Claims {
            sub: "auth0|user-id".into(),
            azp: None,
            permissions: vec!["device".into()],
        });

        assert!(result.is_err());
    }

    #[test]
    fn policy_can_allow_multiple_principal_kinds() {
        let policy =
            super::AuthorizationPolicy::any_of([PrincipalKind::Admin, PrincipalKind::Device]);
        let principal = Principal {
            kind: PrincipalKind::Admin,
            subject: "auth0|user-id".into(),
            client_id: None,
        };

        assert!(policy.allows(&principal));
    }
}
