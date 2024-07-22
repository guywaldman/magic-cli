use orch::{
    execution::StructuredExecutorBuilder,
    lm::LanguageModel,
    response::{variants, Variant, Variants},
};
use thiserror::Error;

#[derive(Variants, serde::Deserialize)]
pub enum AskResponseOption {
    Suggestion(SuggestionResponse),
    Ask(AskResponse),
    Success(SuccessResponse),
    Fail(FailResponse),
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Suggestion",
    scenario = "You are confident enough in the commands you want to suggest",
    description = "Suggestion for a command to run and its explanation"
)]
pub struct SuggestionResponse {
    #[schema(description = "The command to run", example = "kubectl get pods")]
    pub command: String,
    #[schema(description = "The explanation for the command", example = "List all Kubernetes pods")]
    pub explanation: String,
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Ask",
    scenario = "You are missing context and need the user to run a command in the terminal to examine its output",
    description = "Request for the user to run a command"
)]
pub struct AskResponse {
    #[schema(description = "The command to run", example = "cat README.md")]
    pub command: String,
    #[schema(
        description = "The rationale for running the command, explaining why further context is needed",
        example = "I need to see the instructions from the README file"
    )]
    pub rationale: String,
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Success",
    scenario = "You believe that the user request has been fulfilled",
    description = "Success"
)]
pub struct SuccessResponse {
    #[schema(description = "Whether the user request has been fulfilled", example = "true")]
    pub success: bool,
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Fail",
    scenario = "You are not confident enough in the command you want to suggest",
    description = "Fail"
)]
pub struct FailResponse {
    #[schema(
        description = "The error message, should be descriptive",
        example = "'I could not understand the instructions' or 'I could not find a suitable command'"
    )]
    pub error: String,
}

#[derive(Debug, Error)]
pub enum AskEngineError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

pub struct AskEngine {
    lm: Box<dyn LanguageModel>,
}

impl AskEngine {
    const PREAMBLE: &'static str = "
        You are a helpful assistant which helps the user to find the best commands to run given their request.
        You are an expert in the terminal and can run commands in the current shell session.
        You should not assume anything and request the user to run commands which would help you to achieve the goal.
    ";

    pub fn new(lm: Box<dyn LanguageModel>) -> Self {
        Self { lm }
    }

    pub async fn ask_command(&self, prompt: &str) -> Result<AskResponseOption, AskEngineError> {
        let executor = StructuredExecutorBuilder::new()
            .with_lm(&*self.lm)
            .with_preamble(Self::PREAMBLE)
            .with_options(Box::new(variants!(AskResponseOption)))
            .try_build()
            .map_err(|e| AskEngineError::Configuration(e.to_string()))?;
        let response = executor
            .execute(prompt)
            .await
            .map_err(|e| AskEngineError::Execution(e.to_string()))?;
        Ok(response.content)
    }

    pub async fn ask_command_with_context(&self, prompt: &str, context: &str) -> Result<AskResponseOption, AskEngineError> {
        let prompt = format!("ORIGINAL PROMPT: {}\nCONTEXT: {}", prompt, context);
        let executor = StructuredExecutorBuilder::new()
            .with_lm(&*self.lm)
            .with_preamble(Self::PREAMBLE)
            .with_options(Box::new(variants!(AskResponseOption)))
            .try_build()
            .map_err(|e| AskEngineError::Configuration(e.to_string()))?;
        let response = executor
            .execute(&prompt)
            .await
            .map_err(|e| AskEngineError::Execution(e.to_string()))?;
        Ok(response.content)
    }
}
