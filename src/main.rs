use promptpunch::{
    llm::chat_gpt::ChatGptModel, prelude::*, prompt::read_markdown_prompt_from_file,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sqlx::any::install_default_drivers();
    let to_inject = [InjectableData::new(
        "{{user_request}}",
        "Select all customers that were created on 09.12.2024. Add generated podcasts amount as well. Mark which customers started using new TTS provider",
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
    let llm = ChatGpt::from_env().with_model(ChatGptModel::Turbo35);
    let result = llm.complete_chat(prompt).await?;
    println!("{}", result.last_assistant_response()?);
    Ok(())
}
