use promptpunch::{llm::chat_gpt::ChatGptModel, prelude::*};
use sqlx::AnyPool;
use tokio::net::TcpListener;
use wingb::{web, DatabaseOrbiter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sqlx::any::install_default_drivers();
    let db_url = std::env::var("DATABASE_URL").expect("Failed to get DATABASE_URL");

    let llm = ChatGpt::from_env().with_model(ChatGptModel::Mini4o);
    let pool = AnyPool::connect(&db_url)
        .await
        .expect("Failed to create pool");
    let orbiter = DatabaseOrbiter::new(llm, pool);

    let host = match std::env::var("WINGB_HOST") {
        Ok(host) => host,
        Err(_) => {
            tracing::warn!("WINGB_HOST env var not set, using default");
            "localhost:8080".to_string()
        }
    };
    tracing::info!("Starting app on {}", host);

    let app = web::create_app(orbiter).await?;
    let listener = TcpListener::bind(host).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
