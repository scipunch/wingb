use std::fmt::Display;

use promptpunch::{
    llm::LlmProvider,
    prelude::ChatGpt,
    prompt::{read_markdown_prompt_from_file, InjectableData},
    PromptBuilder,
};
use sqlx::{postgres::PgRow, PgPool};

pub struct DatabaseOrbiter {
    llm: ChatGpt,
    pool: PgPool,
}

impl DatabaseOrbiter {
    pub fn new(llm: ChatGpt, pool: PgPool) -> Self {
        Self { llm, pool }
    }

    pub async fn request_db(&self, prompt: impl Display) -> anyhow::Result<Vec<PgRow>> {
        let gpt_query = self.generate_sql(prompt).await?;

        println!("Got query: {}", gpt_query);
        let rows = sqlx::query(&gpt_query)
            .fetch_all(&self.pool)
            .await
            .map_err(anyhow::Error::from)?;

        Ok(rows)
    }

    async fn generate_sql(&self, prompt: impl Display) -> anyhow::Result<String> {
        let to_inject = [InjectableData::new(
        "{{user_request}}",
        prompt
    ), InjectableData::new(
        "{{table_context}}",
        "customer_tts_provider_log exists for each customer by default. The default tts_provider is Google (1), the new one provider is Cartesia (2). The active tts_provider for the customer - latest row in this table"
    )];
        let prompt = PromptBuilder::default()
            .messages(read_markdown_prompt_from_file(
                "static/prompt.md",
                &to_inject,
            )?)
            .build()?;
        let result = self.llm.complete_chat(prompt).await?;
        let sql = result.last_assistant_response()?;
        Ok(sql)
    }
}
