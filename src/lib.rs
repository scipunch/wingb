pub mod web;

use std::fmt::Display;

use promptpunch::{
    llm::LlmProvider,
    prelude::ChatGpt,
    prompt::{read_markdown_prompt, read_markdown_prompt_from_file, InjectableData},
    PromptBuilder,
};
use serde::Serialize;
use serde_json::Value;
use sqlx::{any::AnyRow, AnyPool, Column, Row};
use sqlx_core::any::AnyValueKind;
use tracing::info;

#[derive(Clone)]
pub struct DatabaseOrbiter {
    llm: ChatGpt,
    pool: AnyPool,
}

#[derive(Debug, Default, Serialize)]
pub struct Table {
    head: Vec<String>,
    body: Vec<Vec<serde_json::Value>>,
    sql_query: String,
}

impl DatabaseOrbiter {
    pub fn new(llm: ChatGpt, pool: AnyPool) -> Self {
        Self { llm, pool }
    }

    pub async fn request_db(&self, prompt: impl Display) -> anyhow::Result<Table> {
        let sql_query = self.generate_sql(prompt).await?;
        info!("Generated sql_query: {}", sql_query.replace("/n", ""));
        let rows = sqlx::query(&sql_query)
            .fetch_all(&self.pool)
            .await
            .map_err(anyhow::Error::from)?;

        if rows.is_empty() {
            return Ok(Table::default());
        }

        let head = rows[0]
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect::<Vec<_>>();
        info!("Got colummns: {} with {} rows", head.join(", "), rows.len());

        let body = rows
            .iter()
            .map(any_row_to_json)
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        Ok(Table {
            head,
            body,
            sql_query,
        })
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
            .messages(read_markdown_prompt(
                std::include_str!("../static/prompt.md").lines(),
                &to_inject,
            )?)
            .build()?;
        let result = self.llm.complete_chat(prompt).await?;
        let sql = result.last_assistant_response()?;
        Ok(sql)
    }
}

fn any_row_to_json(row: &AnyRow) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut result = vec![];
    for value in &row.values {
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
        result.push(value);
    }
    Ok(result)
}
