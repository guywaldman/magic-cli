pub(crate) trait Llm {
    fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, Box<dyn std::error::Error>>;
}
