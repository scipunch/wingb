use promptpunch::{llm::chat_gpt::ChatGptModel, prelude::*};
use sqlx::AnyPool;
use wingb::{web, DatabaseOrbiter};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::any::install_default_drivers();
    let db_url = secrets
        .get("DATABASE_URL")
        .expect("Failed to get DATABASE_URL");

    std::env::set_var(
        "OPENAI_API_KEY",
        secrets
            .get("OPENAI_API_KEY")
            .expect("Failed to get OPENAI_API_KEY"),
    );

    let llm = ChatGpt::from_env().with_model(ChatGptModel::Mini4o);
    let pool = AnyPool::connect(&db_url)
        .await
        .expect("Failed to create pool");
    let orbiter = DatabaseOrbiter::new(llm, pool);

    std::env::set_var(
        "USER_NAME",
        secrets.get("USER_NAME").expect("Failed to get USER_NAME"),
    );
    std::env::set_var(
        "USER_PASSWORD",
        secrets
            .get("USER_PASSWORD")
            .expect("Failed to get USER_PASSWORD"),
    );
    let app = web::create_app(orbiter).await?;
    Ok(app.into())
}
