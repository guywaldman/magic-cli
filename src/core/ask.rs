use super::{Llm, LlmOrchestration, LlmOrchestrationError, SystemPromptCommandSchemaField, SystemPromptResponseOption};
use serde::{Deserialize, Serialize};
use std::cell::OnceCell;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AskEngineError {
    #[error("LLM orchestration error: {0}")]
    LlmOrchestration(#[from] LlmOrchestrationError),
}

pub struct AskEngine {
    llm: Box<dyn Llm>,
}

impl LlmOrchestration<AskResponse> for AskEngine {
    fn parse_response(&self, response: &str, type_name: &str) -> Result<AskResponse, super::LlmOrchestrationError> {
        match type_name {
            "success" => Ok(AskResponse::Success(
                serde_json::from_str::<SuccessResponseVariant>(response).unwrap(),
            )),
            "suggest" => Ok(AskResponse::Suggest(
                serde_json::from_str::<SuggestResponseVariant>(response).unwrap(),
            )),
            "ask" => Ok(AskResponse::Ask(serde_json::from_str::<AskResponseVariant>(response).unwrap())),
            _ => Err(LlmOrchestrationError::Parsing(format!("Unknown type name: {}", type_name))),
        }
    }

    fn preamble(&self) -> String {
        "You are a helpful assistant which helps the user to find the best commands to run given their request.
        You are an expert in the terminal and can run commands in the current shell session.
        You should not assume anything and request the user to run commands which would help you to achieve the goal."
            .to_string()
    }

    fn response_options() -> OnceCell<Vec<SystemPromptResponseOption>> {
        let response_options = vec![
            SystemPromptResponseOption {
                scenario: "You are confident enough in the commands you want to suggest".to_string(),
                response: "Suggestion for a command to run and its explanation".to_string(),
                type_name: "suggest".to_string(),
                schema: vec![
                    SystemPromptCommandSchemaField {
                        name: "command".to_string(),
                        description: "The command to run".to_string(),
                        typ: "string".to_string(),
                        example: "kubectl get pods".to_string(),
                    },
                    SystemPromptCommandSchemaField {
                        name: "explanation".to_string(),
                        description: "The explanation for the command".to_string(),
                        typ: "string".to_string(),
                        example: "List all Kubernetes pods".to_string(),
                    },
                ],
            },
            SystemPromptResponseOption {
                scenario: "You are missing context and need the user to run a command in the terminal to examine its output".to_string(),
                response: "Request for the user to run a command".to_string(),
                type_name: "ask".to_string(),
                schema: vec![
                    SystemPromptCommandSchemaField {
                        name: "command".to_string(),
                        description: "The command to run".to_string(),
                        typ: "string".to_string(),
                        example: "cat README.md".to_string(),
                    },
                    SystemPromptCommandSchemaField {
                        name: "rationale".to_string(),
                        description: "The rationale for running the command, explaining why further context is needed".to_string(),
                        typ: "string".to_string(),
                        example: "I need to see the instructions from the README file".to_string(),
                    },
                ],
            },
            SystemPromptResponseOption {
                scenario: "You believe that the user request has been fulfilled".to_string(),
                response: "Success".to_string(),
                type_name: "success".to_string(),
                schema: vec![SystemPromptCommandSchemaField {
                    name: "success".to_string(),
                    description: "Whether the user request has been fulfilled".to_string(),
                    typ: "boolean".to_string(),
                    example: "true".to_string(),
                }],
            },
        ];
        let cell = OnceCell::new();
        cell.get_or_init(|| response_options);
        cell
    }
}

impl AskEngine {
    pub fn new(llm: Box<dyn Llm>) -> Self {
        Self { llm }
    }

    pub fn ask_command(&self, prompt: &str) -> Result<AskResponse, AskEngineError> {
        let response = self.run(dyn_clone::clone_box(&*self.llm), prompt)?;
        Ok(response)
    }

    pub fn ask_command_with_context(&self, prompt: &str, context: &str) -> Result<AskResponse, AskEngineError> {
        let prompt = format!("ORIGINAL PROMPT: {}\nCONTEXT: {}", prompt, context);
        let response = self.run(dyn_clone::clone_box(&*self.llm), &prompt)?;
        Ok(response)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestResponseVariant {
    pub command: String,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AskResponseVariant {
    pub command: String,
    pub rationale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponseVariant {
    pub success: bool,
}

#[derive(Debug)]
pub enum AskResponse {
    Suggest(SuggestResponseVariant),
    Ask(AskResponseVariant),
    Success(SuccessResponseVariant),
}
