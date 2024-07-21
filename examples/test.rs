//! This example demonstrates how to use the `Executor` to generate a structured response from the LLM.
//! Run like so: `cargo run --example structured_data_generation_capital -- France`

#![allow(dead_code)]

use orch::execution::*;
use orch::lm::*;
use orch::response::*;

#[derive(Variants, serde::Deserialize)]
pub enum ResponseVariants {
    Answer(AnswerResponseVariant),
    Fail(FailResponseVariant),
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Answer",
    scenario = "You know the capital city of the country",
    description = "Capital city of the country"
)]
pub struct AnswerResponseVariant {
    #[schema(description = "Capital city of the received country", example = "London")]
    pub capital: String,
}

#[derive(Variant, serde::Deserialize)]
#[variant(
    variant = "Fail",
    scenario = "You don't know the capital city of the country",
    description = "Reason why the capital city is not known"
)]
pub struct FailResponseVariant {
    #[schema(
        description = "Reason why the capital city is not known",
        example = "Country 'foobar' does not exist"
    )]
    pub reason: String,
}

#[tokio::main]
async fn main() {
    // ! Change this to use a different provider.
    let provider = LanguageModelProvider::Ollama;

    let args = std::env::args().collect::<Vec<_>>();
    let prompt = args.get(1).unwrap_or_else(|| {
        eprintln!("ERROR: Please provide a country name");
        std::process::exit(1);
    });

    // Use a different language model, per the `provider` variable (feel free to change it).
    let open_ai_api_key = {
        if provider == LanguageModelProvider::OpenAi {
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| panic!("OPENAI_API_KEY environment variable not set"))
        } else {
            String::new()
        }
    };
    let lm: Box<dyn LanguageModel> = match provider {
        LanguageModelProvider::Ollama => Box::new(OllamaBuilder::new().try_build().unwrap()),
        LanguageModelProvider::OpenAi => Box::new(OpenAiBuilder::new().with_api_key(open_ai_api_key).try_build().unwrap()),
    };

    let executor = StructuredExecutorBuilder::new()
        .with_lm(&*lm)
        .with_preamble(
            "
            You are a geography expert who helps users understand the capital city of countries around the world.
            You will receive a country name, and you will need to provide the capital city of that country.
            ",
        )
        .with_options(&variants!(ResponseVariants))
        .try_build()
        .unwrap();
    let response = executor.execute(prompt).await.expect("Execution failed");

    match response.content {
        ResponseVariants::Answer(answer) => {
            println!("Capital city: {}", answer.capital);
        }
        ResponseVariants::Fail(fail) => {
            println!("Model failed to generate a response: {}", fail.reason);
        }
    }
}
