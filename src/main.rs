use promptpunch::{llm::chat_gpt::ChatGptModel, prelude::*};
use sqlx::AnyPool;
use wingdb::DatabaseOrbiter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sqlx::any::install_default_drivers();
    let db_url = std::env::var("DATABASE_URL")?;
    let prompt = "Select all customers that were created on 12-09-2024. Add generated podcasts amount as well. Mark which customers started using new TTS provider. Add customer created_at date as well";
    let llm = ChatGpt::from_env().with_model(ChatGptModel::Mini4o);

    let pool = AnyPool::connect(&db_url).await?;
    let orbiter = DatabaseOrbiter::new(llm, pool.into());
    let result = orbiter.request_db(&prompt).await?;
    for row in result {
        println!("{:?}", row);
    }

    Ok(())
}
