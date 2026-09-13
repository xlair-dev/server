use std::{collections::HashSet, env};

pub fn host() -> String {
    env::var("HOST").expect("HOST must be set")
}

/// Returns the HTTP port. Defaults to 8080 when `APP_PORT` is not provided.
pub fn app_port() -> String {
    env::var("APP_PORT").unwrap_or_else(|_| "8080".into())
}

pub fn allowed_origin() -> String {
    #[cfg(test)]
    {
        env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_owned())
    }

    #[cfg(not(test))]
    env::var("ALLOWED_ORIGIN").expect("ALLOWED_ORIGIN must be set")
}

pub fn auth0_issuer() -> String {
    env::var("AUTH0_ISSUER").expect("AUTH0_ISSUER must be set")
}

pub fn auth0_audience() -> String {
    env::var("AUTH0_AUDIENCE").expect("AUTH0_AUDIENCE must be set")
}

/// Returns the Auth0 client ID trusted for GitHub-authenticated dashboard users.
pub fn auth0_dashboard_client_id() -> String {
    env::var("AUTH0_DASHBOARD_CLIENT_ID").expect("AUTH0_DASHBOARD_CLIENT_ID must be set")
}

pub fn auth0_admin_subjects() -> HashSet<String> {
    let value = env::var("AUTH0_ADMIN_SUBJECTS").expect("AUTH0_ADMIN_SUBJECTS must be set");
    let subjects = value
        .split(',')
        .map(str::trim)
        .filter(|subject| !subject.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();
    if subjects.is_empty() {
        panic!("AUTH0_ADMIN_SUBJECTS must contain at least one subject");
    }
    subjects
}

pub fn postgres_host() -> String {
    env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set")
}

pub fn postgres_port() -> String {
    env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set")
}

/// Builds a PostgreSQL connection URL. Implicitly depends on `POSTGRES_HOST` and `POSTGRES_PORT` being configured alongside the credential variables.
pub fn postgres_url() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let host = postgres_host();
    let port = postgres_port();
    let mut url = url::Url::parse("postgres://localhost").expect("valid PostgreSQL URL base");
    url.set_username(&user).expect("valid PostgreSQL username");
    url.set_password(Some(&password))
        .expect("valid PostgreSQL password");
    url.set_host(Some(&host)).expect("valid PostgreSQL host");
    url.set_port(Some(port.parse().expect("POSTGRES_PORT must be numeric")))
        .expect("valid PostgreSQL port");
    url.set_path(&db);
    url.to_string()
}

pub fn r2_endpoint() -> String {
    env::var("R2_ENDPOINT").expect("R2_ENDPOINT must be set")
}

pub fn r2_bucket() -> String {
    env::var("R2_BUCKET").expect("R2_BUCKET must be set")
}

pub fn r2_access_key_id() -> String {
    env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID must be set")
}

pub fn r2_secret_access_key() -> String {
    env::var("R2_SECRET_ACCESS_KEY").expect("R2_SECRET_ACCESS_KEY must be set")
}

pub fn r2_public_base_url() -> String {
    env::var("R2_PUBLIC_BASE_URL").expect("R2_PUBLIC_BASE_URL must be set")
}
