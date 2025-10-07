use std::env;

pub fn host() -> String {
    env::var("HOST").expect("HOST must be set")
}

pub fn port() -> String {
    env::var("PORT").expect("PORT must be set")
}

pub fn postgres_url() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");
    format!(
        "postgres://{}:{}@{}:{}/{}",
        user,
        password,
        host(),
        port,
        db
    )
}
