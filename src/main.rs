use promptpunch::{llm::chat_gpt::ChatGptModel, prelude::*};
use sqlx::PgPool;
use wingdb::DatabaseOrbiter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sqlx::any::install_default_drivers();
    let db_url = std::env::var("DATABASE_URL")?;
    let prompt = "Select all customers that were created on 09.12.2024. Add generated podcasts amount as well. Mark which customers started using new TTS provider";
    let llm = ChatGpt::from_env().with_model(ChatGptModel::Mini4o);

    let pool = PgPool::connect(&db_url).await?;
    let orbiter = DatabaseOrbiter::new(llm, pool.into());
    let result = orbiter.request_db(&prompt).await?;
    for row in result {
        println!("{:?}", row);
    }

    Ok(())
}
