use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use axum_htmx::{AutoVaryLayer, HxRequest};
use axum_template::{engine::Engine, Key, RenderHtml};
use minijinja::Environment;
use serde::Deserialize;
use tower_sessions::MemoryStore;

use crate::{
    app_state::AppState,
    auth::{self, Backend, User},
    DatabaseOrbiter,
};
use axum_login::{
    login_required,
    tower_sessions::{Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};

use time::Duration;
use tokio::{net::TcpListener, time::sleep};

pub async fn serve(orbiter: DatabaseOrbiter) -> anyhow::Result<()> {
    let mut jinja = Environment::new();
    jinja
        .add_template("/generate", std::include_str!("../static/sql-table.html"))
        .unwrap();

    let session_store = MemoryStore::default();
    let key = tower_sessions::cookie::Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);

    let backend = Backend::new(User::from_env());

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let app = Router::new()
        .route("/", get(get_root))
        .route("/generate", post(post_generate))
        .layer(AutoVaryLayer)
        .route_layer(login_required!(Backend, login_url = "/login"))
        .merge(auth::router())
        .layer(auth_layer)
        .with_state(AppState {
            engine: Engine::from(jinja),
            orbiter,
        });

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Debug, Deserialize)]
struct GeneratePrompt {
    prompt: String,
}

async fn post_generate(
    State(state): State<AppState>,
    Key(key): Key,
    Form(data): Form<GeneratePrompt>,
) -> impl IntoResponse {
    println!("Got prompt data={:?}", data);
    let table = state.orbiter.request_db(&data.prompt).await.unwrap();

    RenderHtml(key, state.engine, table)
}

async fn get_root(HxRequest(hx_request): HxRequest) -> Html<&'static str> {
    if hx_request {
        sleep(std::time::Duration::from_secs(3)).await;
        return Html("htmx response");
    }
    Html(std::include_str!("../static/index.html"))
}
