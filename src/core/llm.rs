use std::cell::OnceCell;

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LlmProvider {
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "openai")]
    OpenAi,
}

#[derive(Debug, Error)]
pub enum LlmProviderError {
    #[error("Invalid LLM provider: {0}")]
    InvalidValue(String),
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProvider::Ollama => write!(f, "ollama"),
            LlmProvider::OpenAi => write!(f, "openai"),
        }
    }
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::Ollama
    }
}

impl TryFrom<&str> for LlmProvider {
    type Error = LlmProviderError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ollama" => Ok(LlmProvider::Ollama),
            "openai" => Ok(LlmProvider::OpenAi),
            _ => Err(LlmProviderError::InvalidValue(value.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Text generation error: {0}")]
    TextGeneration(String),

    #[error("Embedding generation error: {0}")]
    EmbeddingGeneration(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub(crate) trait Llm: DynClone {
    /// Generates a response from the LLM.
    ///
    /// # Arguments
    /// * `prompt` - The prompt to generate a response for.
    /// * `system_prompt` - The system prompt to use for the generation.
    ///
    /// # Returns
    /// A [Result] containing the response from the LLM or an error if there was a problem.
    ///
    fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, LlmError>;

    /// Generates an embedding from the LLM.
    ///
    /// # Arguments
    /// * `prompt` - The item to generate an embedding for.
    ///
    /// # Returns
    ///
    /// A [Result] containing the embedding or an error if there was a problem.
    fn generate_embedding(&self, prompt: &str) -> Result<Vec<f32>, LlmError>;

    /// Returns the provider of the LLM.
    fn provider(&self) -> LlmProvider;

    /// Returns the name of the model used for text completions.
    fn text_completion_model_name(&self) -> String;

    /// Returns the name of the model used for embeddings.
    fn embedding_model_name(&self) -> String;
}

#[derive(Debug)]
pub(crate) struct SystemPromptResponseOption {
    pub scenario: String,
    pub type_name: String,
    pub response: String,
    pub schema: Vec<SystemPromptCommandSchemaField>,
}

#[derive(Debug)]
pub(crate) struct SystemPromptCommandSchemaField {
    pub name: String,
    pub description: String,
    pub typ: String,
    pub example: String,
}

#[derive(Debug, Error)]
pub enum LlmOrchestrationError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    #[error("Parsing error: {0}")]
    Parsing(String),
}

pub trait LlmOrchestration<TOption> {
    /// Parses the response from the LLM.
    ///
    /// # Arguments
    /// * `response` - The response from the LLM.
    fn parse_response(&self, response: &str, type_name: &str) -> Result<TOption, LlmOrchestrationError>;

    /// Preamble for the model instructions
    /// (e.g., "You are a helpful assistant which helps the user to find the best commands to run given their request.")
    fn preamble(&self) -> String;

    /// Response options for the model instructions.
    fn response_options() -> OnceCell<Vec<SystemPromptResponseOption>>;

    fn run(&self, llm: Box<dyn Llm>, prompt: &str) -> Result<TOption, LlmOrchestrationError> {
        let response = llm
            .generate(prompt, &self.model_instructions())
            .map_err(LlmOrchestrationError::Llm)?;
        let parsed_response: serde_json::Value =
            serde_json::from_str(&response).map_err(|e| LlmOrchestrationError::Parsing(e.to_string()))?;
        let type_name = parsed_response
            .get("type")
            .ok_or(LlmOrchestrationError::Parsing(
                "Response from LLM does not contain a 'type' field".to_string(),
            ))?
            .as_str()
            .ok_or(LlmOrchestrationError::Parsing(
                "Response from LLM does contains a 'type' field, but it is not a string".to_string(),
            ))?;

        self.parse_response(&response, type_name)
    }

    /// Instructions for the model.
    fn model_instructions(&self) -> String {
        let preamble = Self::preamble(self);
        let response_options_cell = Self::response_options();
        let response_options = response_options_cell.get().unwrap();

        let all_types = response_options.iter().map(|option| option.type_name.clone()).collect::<Vec<_>>();

        let response_options_text = response_options
            .iter()
            .map(|option| {
                let mut schema_text = String::new();
                let mut schema_example = "{".to_string();
                let type_field = SystemPromptCommandSchemaField {
                    name: "type".to_string(),
                    description: format!("The type of the response (\"{}\" in this case)", option.type_name).to_string(),
                    typ: "string".to_string(),
                    example: all_types.first().unwrap().to_string(),
                };

                for (i, field) in option.schema.iter().chain(std::iter::once(&type_field)).enumerate() {
                    schema_text.push_str(&format!(
                        "  - `{}` of type {} (description: {})\n\n",
                        field.name, field.typ, field.description
                    ));
                    schema_example.push_str(&format!("\"{}\": \"{}\"", field.name, field.example));

                    if i < option.schema.len() - 1 {
                        schema_example.push(',');
                    }
                }
                schema_example.push('}');

                format!(
                    "SCENARIO: {}\nRESPONSE: {}\nSCHEMA:\n{}\nEXAMPLE RESPONSE: {}\n\n\n",
                    option.scenario, option.response, schema_text, schema_example
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let system_prompt = format!(
            "{preamble}
                    You have {choices_len} choices to respond, in a JSON format:
                    {response_options_text}
                ",
            preamble = preamble,
            choices_len = response_options.len(),
            response_options_text = response_options_text
        )
        .trim()
        .to_string();
        system_prompt
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone)]
    struct MockLlm;

    impl Llm for MockLlm {
        fn generate(&self, _: &str, _: &str) -> Result<String, LlmError> {
            Ok(serde_json::to_string(&serde_json::json!({
                "type": "city",
                "name": "Bucharest",
                "country": "Romania",
            }))
            .unwrap())
        }

        fn generate_embedding(&self, _prompt: &str) -> Result<Vec<f32>, LlmError> {
            todo!()
        }

        fn provider(&self) -> LlmProvider {
            LlmProvider::Ollama
        }

        fn text_completion_model_name(&self) -> String {
            "Mock LLM".to_string()
        }

        fn embedding_model_name(&self) -> String {
            "Mock LLM".to_string()
        }
    }

    #[test]
    fn test_orchestration() {
        struct TestOrchestration;

        #[derive(Debug, Serialize, Deserialize)]
        struct CityOptionVariant {
            name: String,
            country: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct CountryOptionVariant {
            name: String,
        }

        enum TestOption {
            City(CityOptionVariant),
            Country(CountryOptionVariant),
        }

        impl LlmOrchestration<TestOption> for TestOrchestration {
            fn parse_response(&self, response: &str, type_name: &str) -> Result<TestOption, LlmOrchestrationError> {
                match type_name {
                    "city" => Ok(TestOption::City(serde_json::from_str::<CityOptionVariant>(response).unwrap())),
                    "country" => Ok(TestOption::Country(serde_json::from_str::<CountryOptionVariant>(response).unwrap())),
                    _ => Err(LlmOrchestrationError::Parsing(format!("Unknown type name: {}", type_name))),
                }
            }

            fn preamble(&self) -> String {
                "You are a helpful assistant who identifies whether the user supplied a city or a country.".to_string()
            }

            fn response_options() -> OnceCell<Vec<SystemPromptResponseOption>> {
                let response_options = vec![
                    SystemPromptResponseOption {
                        scenario: "User supplied a city".to_string(),
                        response: "Information about the city".to_string(),
                        type_name: "city".to_string(),
                        schema: vec![
                            SystemPromptCommandSchemaField {
                                name: "name".to_string(),
                                description: "Name of the city".to_string(),
                                typ: "string".to_string(),
                                example: "Paris".to_string(),
                            },
                            SystemPromptCommandSchemaField {
                                name: "country".to_string(),
                                description: "Country of the city".to_string(),
                                typ: "string".to_string(),
                                example: "France".to_string(),
                            },
                        ],
                    },
                    SystemPromptResponseOption {
                        scenario: "User supplied a country".to_string(),
                        response: "Information about the country".to_string(),
                        type_name: "country".to_string(),
                        schema: vec![SystemPromptCommandSchemaField {
                            name: "name".to_string(),
                            description: "Name of the country".to_string(),
                            typ: "string".to_string(),
                            example: "France".to_string(),
                        }],
                    },
                ];
                let cell = OnceCell::new();
                cell.get_or_init(|| response_options);
                cell
            }
        }

        let orchestration = TestOrchestration;
        let llm = MockLlm;
        let prompt = "Bucharest";
        let response = orchestration.run(Box::new(llm), prompt);
        assert!(response.is_ok());
        let response = response.unwrap();
        match response {
            TestOption::City(city) => {
                assert_eq!(city.name, "Bucharest");
                assert_eq!(city.country, "Romania");
            }
            TestOption::Country(country) => {
                panic!("Expected a city, got a country: {:?}", country);
            }
        }
    }
}
