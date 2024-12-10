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
use serde::{Deserialize, Serialize};
use sqlx::AnyPool;
use tokio::{net::TcpListener, time::sleep};

use crate::DatabaseOrbiter;

type AppEngine = Engine<Environment<'static>>;

// Define your application shared state
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
        // Create the application state
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
    // let prompt = "Select all customers that were created on 12-09-2024. Add generated podcasts amount as well. Mark which customers started using new TTS provider. Add customer created_at date as well";
    let table = state.orbiter.request_db(&data.prompt).await.unwrap();

    RenderHtml(key, state.engine, table)
}

// Our handler differentiates full-page GET requests from htmx-based ones by looking at the `hx-request`
// requestheader.
//
// The middleware sees the usage of the `HxRequest` extractor and automatically adds the
// `Vary: hx-request` response header.
async fn get_root(HxRequest(hx_request): HxRequest) -> Html<&'static str> {
    if hx_request {
        // For htmx-based GET request, it returns a partial page update
        sleep(Duration::from_secs(3)).await;
        return Html("htmx response");
    }
    // While for a normal GET request, it returns the whole page
    Html(std::include_str!("../static/index.html"))
}
