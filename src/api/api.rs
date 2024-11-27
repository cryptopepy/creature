use crate::api::openrouter::OpenRouterClient;
use crate::api::huggingface::HuggingFaceClient;
use crate::api::openai::OpenAIClient;

//API Enum
pub enum ApiClient {
    OpenAI(OpenAIClient),
    HuggingFace(HuggingFaceClient),
    OpenRouter(OpenRouterClient),
}

pub fn create_api_client(api_key: &str) -> Result<ApiClient, Box<dyn std::error::Error>> {
    if api_key.starts_with("hf_") {
        HuggingFaceClient::new(api_key.to_string())
            .map(ApiClient::HuggingFace) // Wrap in the HuggingFace variant
            .map_err(|e| e.into()) // Convert error to Box<dyn Error>
    } else if api_key.starts_with("sk-") {
        OpenAIClient::new(api_key.to_string())
            .map(ApiClient::OpenAI)
            .map_err(|e| e.into())
    } else {
        OpenRouterClient::new(api_key.to_string())
            .map(ApiClient::OpenRouter) // Wrap in the OpenRouter variant
            .map_err(|e| e.into()) // Convert error to Box<dyn Error>
    }
}
