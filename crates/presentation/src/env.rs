use std::env;

pub fn host() -> String {
    env::var("HOST").expect("HOST must be set")
}

/// Returns the HTTP port. Defaults to 8080 when `APP_PORT` is not provided.
pub fn app_port() -> String {
    env::var("APP_PORT").unwrap_or_else(|_| "8080".into())
}

/// allowed cors origin
pub fn allowed_origin() -> String {
    env::var("ALLOWED_ORIGIN").expect("ALLOWED_ORIGIN must be set")
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
    format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, db)
}
