use askama::Template;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use axum_htmx::AutoVaryLayer;
use serde::Deserialize;
use tower_sessions::MemoryStore;

use crate::{
    app_state::AppState,
    auth::{self, Backend, User},
    DatabaseOrbiter, Table,
};
use axum_login::{
    login_required,
    tower_sessions::{Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};

use time::Duration;
use tokio::net::TcpListener;

pub async fn serve(orbiter: DatabaseOrbiter) -> anyhow::Result<()> {
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
        .with_state(AppState { orbiter });

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Debug, Deserialize)]
struct GeneratePrompt {
    prompt: String,
}

#[derive(Template)]
#[template(path = "component/sql-table.html")]
struct PromptResponse {
    head: Vec<String>,
    body: Vec<Vec<serde_json::Value>>,
    sql_query: String,
}

impl From<Table> for PromptResponse {
    fn from(value: Table) -> Self {
        Self {
            head: value.head,
            body: value.body,
            sql_query: value.sql_query,
        }
    }
}

async fn post_generate(
    State(state): State<AppState>,
    Form(data): Form<GeneratePrompt>,
) -> impl IntoResponse {
    let table = state.orbiter.request_db(&data.prompt).await.unwrap();
    PromptResponse::from(table).render().unwrap()
}

#[derive(Template)]
#[template(path = "page/index.html")]
struct Root {}

async fn get_root() -> impl IntoResponse {
    let root = Root {};
    root.render().unwrap()
}
