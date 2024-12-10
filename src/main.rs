use promptpunch::{llm::chat_gpt::ChatGptModel, prelude::*};
use sqlx::AnyPool;
use wingdb::{web, DatabaseOrbiter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sqlx::any::install_default_drivers();
    let db_url = std::env::var("DATABASE_URL")?;

    let llm = ChatGpt::from_env().with_model(ChatGptModel::Mini4o);
    let pool = AnyPool::connect(&db_url).await?;
    let orbiter = DatabaseOrbiter::new(llm, pool);

    web::serve(orbiter).await?;

    Ok(())
}
