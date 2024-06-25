pub(crate) trait Llm {
    /// Generates a response from the LLM.
    ///
    /// # Arguments
    /// * `prompt` - The prompt to generate a response for.
    /// * `system_prompt` - The system prompt to use for the generation.
    ///
    /// # Returns
    /// A [Result] containing the response from the LLM or an error if there was a problem.
    ///
    fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// Generates an embedding from the LLM.
    ///
    /// # Arguments
    /// * `prompt` - The item to generate an embedding for.
    ///
    /// # Returns
    ///
    /// A [Result] containing the embedding or an error if there was a problem.
    fn generate_embedding(&self, prompt: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>>;
}
