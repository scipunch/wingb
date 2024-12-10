pub mod web;

use std::fmt::Display;

use promptpunch::{
    llm::LlmProvider,
    prelude::ChatGpt,
    prompt::{read_markdown_prompt_from_file, InjectableData},
    PromptBuilder,
};
use serde_json::Value;
use sqlx::{any::AnyRow, AnyPool, Column, Row};
use sqlx_core::any::AnyValueKind;

pub struct DatabaseOrbiter {
    llm: ChatGpt,
    pool: AnyPool,
}

impl DatabaseOrbiter {
    pub fn new(llm: ChatGpt, pool: AnyPool) -> Self {
        Self { llm, pool }
    }

    pub async fn request_db(&self, prompt: impl Display) -> anyhow::Result<Vec<serde_json::Value>> {
        let gpt_query = self.generate_sql(prompt).await?;

        println!("Got query: {}", gpt_query);
        let rows = sqlx::query(&gpt_query)
            .fetch_all(&self.pool)
            .await
            .map_err(anyhow::Error::from)?;

        if rows.is_empty() {
            return Ok(vec![]);
        }

        let columns = rows[0].columns();
        println!(
            "{}",
            columns
                .iter()
                .map(|col| col.name())
                .collect::<Vec<_>>()
                .join(", ")
        );

        for row in &rows {
            println!("{:?}", any_row_to_json(row));
        }

        Ok(vec![])
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

fn any_row_to_json(row: &AnyRow) -> anyhow::Result<serde_json::Value> {
    let mut result = serde_json::Map::new();
    for ((name, _), value) in row.column_names.iter().zip(&row.values) {
        let value = match &value.kind {
            AnyValueKind::BigInt(v) => serde_json::json!(v),
            AnyValueKind::Null(_) => Value::Null,
            AnyValueKind::Bool(v) => serde_json::json!(v),
            AnyValueKind::SmallInt(v) => serde_json::json!(v),
            AnyValueKind::Integer(v) => serde_json::json!(v),
            AnyValueKind::Real(v) => serde_json::json!(v),
            AnyValueKind::Double(v) => serde_json::json!(v),
            AnyValueKind::Text(v) => serde_json::json!(v),
            AnyValueKind::Blob(v) => serde_json::json!(v),
            _ => anyhow::bail!("Got unexpected value kind"),
        };
        result.insert(name.to_string(), value);
    }
    let value = serde_json::Value::from(result);
    Ok(value)
}
