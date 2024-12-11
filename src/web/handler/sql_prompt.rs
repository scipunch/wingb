use crate::web::app_state::AppState;
use askama_axum::IntoResponse;
use askama_axum::Template;
use axum::extract::{Form, State};
use axum::routing::{get, post, Router};
use serde::Deserialize;
use tracing::info;

use crate::Table;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get::root))
        .route("/generate", post(post::generate))
}

mod post {
    use super::*;
    #[derive(Debug, Deserialize)]
    pub struct GeneratePrompt {
        prompt: String,
    }

    #[derive(Template)]
    #[template(path = "component/sql-table.html")]
    struct PromptResponse {
        head: Vec<String>,
        body: Vec<Vec<serde_json::Value>>,
        sql_query: String,
    }

    pub async fn generate(
        State(state): State<AppState>,
        Form(data): Form<GeneratePrompt>,
    ) -> impl IntoResponse {
        info!(data.prompt);
        let table = state.orbiter.request_db(&data.prompt).await.unwrap();
        PromptResponse::from(table)
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
}

mod get {
    use super::*;

    #[derive(Template)]
    #[template(path = "page/index.html")]
    struct Root {}

    pub async fn root() -> impl IntoResponse {
        Root {}
    }
}
