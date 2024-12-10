use std::fmt::Display;

use promptpunch::{
    llm::LlmProvider,
    prelude::ChatGpt,
    prompt::{read_markdown_prompt_from_file, InjectableData},
    PromptBuilder,
};
use sqlx::{Column, Row};

pub struct DatabaseOrbiter {
    llm: ChatGpt,
    pool: sqlx::AnyPool,
}

impl DatabaseOrbiter {
    pub fn new(llm: ChatGpt, pool: sqlx::AnyPool) -> Self {
        Self { llm, pool }
    }

    pub async fn request_db(&self, prompt: impl Display) -> anyhow::Result<Vec<Vec<String>>> {
        let gpt_query = self.generate_sql(prompt).await?;
        let rows = sqlx::query(&gpt_query)
            .fetch_all(&self.pool)
            .await
            .map_err(anyhow::Error::from)?;

        if rows.is_empty() {
            return Ok(vec![]);
        }

        let columns = rows[0]
            .columns()
            .into_iter()
            .map(|col| col.name())
            .collect::<Vec<_>>();

        let mut result = vec![];
        for row in &rows {
            result.push(
                (0..columns.len())
                    .into_iter()
                    .map(|idx| row.get::<String, _>(idx))
                    .collect::<Vec<_>>(),
            );
        }

        Ok(result)
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
