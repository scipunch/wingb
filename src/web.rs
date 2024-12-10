use std::time::Duration;

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

use tokio::{net::TcpListener, time::sleep};

use crate::DatabaseOrbiter;

type AppEngine = Engine<Environment<'static>>;

#[derive(Clone)]
struct AppState {
    engine: AppEngine,
    orbiter: DatabaseOrbiter,
}

pub async fn serve(orbiter: DatabaseOrbiter) -> anyhow::Result<()> {
    let mut jinja = Environment::new();
    jinja
        .add_template("/generate", std::include_str!("../static/sql-table.html"))
        .unwrap();

    let app = Router::new()
        .route("/", get(get_root))
        .route("/generate", post(post_generate))
        .layer(AutoVaryLayer)
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
        sleep(Duration::from_secs(3)).await;
        return Html("htmx response");
    }
    Html(std::include_str!("../static/index.html"))
}
