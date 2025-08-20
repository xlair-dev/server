use std::env;

pub fn host() -> String {
    env::var("HOST").unwrap_or("127.0.0.1".into())
}

pub fn port() -> String {
    env::var("PORT").unwrap_or("8080".into())
}
