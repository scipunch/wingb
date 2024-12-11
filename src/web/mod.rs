use app_state::AppState;
use auth::{Backend, User};
use axum::routing::Router;
use axum_htmx::AutoVaryLayer;
use tower_sessions::MemoryStore;

use crate::DatabaseOrbiter;
use axum_login::{
    login_required,
    tower_sessions::{Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};

use time::Duration;

pub mod app_state;
mod auth;
mod handler;
mod htmx;

pub async fn create_app(orbiter: DatabaseOrbiter) -> anyhow::Result<Router> {
    let session_store = MemoryStore::default();
    let key = tower_sessions::cookie::Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);

    let backend = Backend::new(User::from_env());

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
        .merge(handler::sql_prompt::router())
        .layer(AutoVaryLayer)
        .route_layer(login_required!(Backend, login_url = "/login"))
        .merge(handler::auth::router())
        .layer(auth_layer)
        .with_state(AppState { orbiter });

    Ok(app)
}
