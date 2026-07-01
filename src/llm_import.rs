use crate::error::AppError;
use crate::model::NewIngredientLine;
use crate::recipe;
use base64::Engine;
use genai::adapter::AdapterKind;
use genai::resolver::{AuthData, Endpoint, ProviderConfig};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct LlmImage {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderInfo {
    pub id: String,
    pub name: String,
    pub env_var: String,
    pub configured: bool,
    pub supports_custom_endpoint: bool,
}

#[derive(serde::Serialize)]
pub struct LlmProvidersResponse {
    pub providers: Vec<LlmProviderInfo>,
}

#[derive(serde::Serialize)]
pub struct LlmModelsResponse {
    pub models: Vec<String>,
}

// ---------------------------------------------------------------------------
// Provider detection
// ---------------------------------------------------------------------------

/// Returns the list of LLM providers and whether they are configured.
/// Providers with a configured API key env var are marked `configured: true`.
/// Ollama is always `configured: true` (no API key; local server).
/// A synthetic "custom" provider for OpenAI-compatible endpoints is appended.
pub fn list_providers() -> Vec<LlmProviderInfo> {
    let kinds: &[(AdapterKind, &str)] = &[
        (AdapterKind::OpenAI, "OpenAI"),
        (AdapterKind::Anthropic, "Anthropic"),
        (AdapterKind::Gemini, "Gemini"),
        (AdapterKind::Groq, "Groq"),
        (AdapterKind::Ollama, "Ollama"),
        (AdapterKind::DeepSeek, "DeepSeek"),
        (AdapterKind::Xai, "xAI"),
    ];

    let mut providers: Vec<LlmProviderInfo> = kinds
        .iter()
        .map(|(kind, name)| {
            let env_var = kind.default_key_env_name().unwrap_or("").to_string();
            let configured = if matches!(kind, AdapterKind::Ollama) {
                true
            } else {
                std::env::var(&env_var).is_ok()
            };
            LlmProviderInfo {
                id: kind.as_lower_str().to_string(),
                name: name.to_string(),
                env_var,
                configured,
                supports_custom_endpoint: false,
            }
        })
        .collect();

    // Append the synthetic "custom" OpenAI-compatible endpoint provider
    providers.push(LlmProviderInfo {
        id: "custom".to_string(),
        name: "Custom OpenAI-compatible".to_string(),
        env_var: String::new(),
        configured: true,
        supports_custom_endpoint: true,
    });

    providers
}

// ---------------------------------------------------------------------------
// Model listing
// ---------------------------------------------------------------------------

/// Lists the available model names for a given provider by querying the
/// provider's API.
///
/// For standard providers, auth is resolved from env vars and the default
/// endpoint is used. For `provider_id == "custom"`, `base_url` is required
/// and `api_key` is optional.
///
/// Uses a 15-second timeout to avoid hanging the UI.
pub async fn list_models(
    provider_id: &str,
    base_url: Option<&str>,
    api_key: Option<&str>,
) -> Result<Vec<String>, AppError> {
    let client = genai::Client::default();

    let (adapter_kind, provider_config) = if provider_id == "custom" {
        let base_url = base_url
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest("base_url is required for custom provider".into())
            })?;
        let base_url = if base_url.ends_with('/') {
            base_url.to_string()
        } else {
            format!("{base_url}/")
        };
        let endpoint = Endpoint::from_owned(base_url);
        let auth = match api_key.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            Some(key) => AuthData::from_single(key),
            None => AuthData::None,
        };
        (AdapterKind::OpenAI, ProviderConfig::from((endpoint, auth)))
    } else {
        let kind = AdapterKind::from_lower_str(provider_id)
            .ok_or_else(|| AppError::BadRequest(format!("unknown provider: {provider_id}")))?;
        (kind, ProviderConfig::default())
    };

    let models_fut = client.all_model_names(adapter_kind, provider_config);
    let models = tokio::time::timeout(std::time::Duration::from_secs(15), models_fut)
        .await
        .map_err(|_| {
            AppError::Llm(
                "Model listing timed out after 15 seconds".into(),
                "llm_timeout",
            )
        })?
        .map_err(map_genai_error)?;
    Ok(models)
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TOOL_NAME: &str = "extract_recipe";

const SYSTEM_PROMPT: &str = "You are a recipe extraction assistant. Extract the recipe from the user's input (an image of a meal, a text description, a recipe URL, or any combination) and call the extract_recipe tool with the result. Always call the tool.";

fn recipe_tool() -> genai::chat::Tool {
    genai::chat::Tool::new(TOOL_NAME)
        .with_description("Extract a structured recipe from the user's input.")
        .with_schema(serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The recipe name" },
                "ingredients": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Ingredient name" },
                            "quantity": { "type": "string", "description": "Quantity e.g. '200g', '2 cups'" }
                        },
                        "required": ["name"]
                    }
                },
                "instructions": { "type": "string", "description": "Cooking instructions" }
            },
            "required": ["name", "ingredients"]
        }))
}

// ---------------------------------------------------------------------------
// User content builder
// ---------------------------------------------------------------------------

fn build_user_content(hint: Option<&str>, image: Option<&LlmImage>) -> genai::chat::MessageContent {
    let mut parts = Vec::new();
    if let Some(h) = hint.map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(genai::chat::ContentPart::from_text(h));
    }
    if let Some(img) = image {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&img.bytes);
        parts.push(genai::chat::ContentPart::from_binary_base64(
            &img.content_type,
            b64,
            Some("image".to_string()),
        ));
    }
    genai::chat::MessageContent::from_parts(parts)
}

// ---------------------------------------------------------------------------
// Main import function
// ---------------------------------------------------------------------------

pub async fn import_via_llm(
    model: &str,
    hint: Option<&str>,
    image: Option<LlmImage>,
    base_url: Option<&str>,
    api_key: Option<&str>,
) -> Result<recipe::ImportDraft, AppError> {
    let client = genai::Client::default();
    let user_content = build_user_content(hint, image.as_ref());

    let chat_req = genai::chat::ChatRequest::new(vec![
        genai::chat::ChatMessage::system(SYSTEM_PROMPT),
        genai::chat::ChatMessage::user(user_content),
    ])
    .with_tools(vec![recipe_tool()]);
    use genai::ModelSpec;
    let model_spec: ModelSpec =
        if let Some(base_url) = base_url.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let base_url = if base_url.ends_with('/') {
                base_url.to_string()
            } else {
                format!("{base_url}/")
            };
            let endpoint = Endpoint::from_owned(base_url);
            let auth = match api_key.map(|s| s.trim()).filter(|s| !s.is_empty()) {
                Some(key) => AuthData::from_single(key),
                None => AuthData::None,
            };
            genai::ServiceTarget {
                endpoint,
                auth,
                model: genai::ModelIden::new(AdapterKind::OpenAI, model),
            }
            .into()
        } else {
            model.into()
        };
    let chat_fut = client.exec_chat(model_spec, chat_req, None);

    let chat_res = match tokio::time::timeout(std::time::Duration::from_secs(60), chat_fut).await {
        Ok(r) => r,
        Err(_) => {
            return Err(AppError::Llm(
                "LLM request timed out after 60 seconds".into(),
                "llm_timeout",
            ));
        }
    };
    let chat_res = chat_res.map_err(map_genai_error)?;

    let tool_calls = chat_res.into_tool_calls();
    let first = tool_calls.first().ok_or_else(|| {
        AppError::Llm(
            "could not parse a recipe from input".into(),
            "llm_parse_failed",
        )
    })?;
    build_draft_from_tool_args(&first.fn_arguments)
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn map_genai_error(err: genai::Error) -> AppError {
    match &err {
        genai::Error::RequiresApiKey { model_iden }
        | genai::Error::NoAuthResolver { model_iden }
        | genai::Error::NoAuthData { model_iden } => {
            let env_var = model_iden
                .adapter_kind
                .default_key_env_name()
                .unwrap_or("the provider's API key environment variable");
            AppError::Llm(
                format!(
                    "API key not configured for provider '{}': set the {} environment variable",
                    model_iden.adapter_kind, env_var
                ),
                "llm_api_key_missing",
            )
        }
        genai::Error::Resolver { model_iden, .. }
        | genai::Error::ModelMapperFailed { model_iden, .. } => AppError::Llm(
            format!("model '{}' could not be resolved: {err}", model_iden),
            "llm_model_not_found",
        ),
        _ => AppError::Llm(format!("LLM request failed: {err}"), "llm_request_failed"),
    }
}

// ---------------------------------------------------------------------------
// Tool output → ImportDraft
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct LlmRecipeDraft {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    ingredients: Vec<LlmIngredient>,
    #[serde(default)]
    instructions: Option<String>,
}

#[derive(serde::Deserialize)]
struct LlmIngredient {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    quantity: Option<String>,
}

fn build_draft_from_tool_args(args: &serde_json::Value) -> Result<recipe::ImportDraft, AppError> {
    let draft: LlmRecipeDraft = serde_json::from_value(args.clone()).map_err(|e| {
        AppError::Llm(
            format!("could not parse a recipe from input: {e}"),
            "llm_parse_failed",
        )
    })?;
    let name = draft.name.map(|s| s.trim().to_string()).unwrap_or_default();
    let ingredients: Vec<NewIngredientLine> = draft
        .ingredients
        .into_iter()
        .map(|i| NewIngredientLine {
            name: i.name.map(|s| s.trim().to_string()).unwrap_or_default(),
            quantity: i.quantity.filter(|s| !s.trim().is_empty()),
        })
        .collect();
    if name.is_empty() || ingredients.iter().all(|i| i.name.is_empty()) {
        return Err(AppError::Llm(
            "could not parse a recipe from input".into(),
            "llm_parse_failed",
        ));
    }
    let ingredients: Vec<_> = ingredients
        .into_iter()
        .filter(|i| !i.name.is_empty())
        .collect();
    Ok(recipe::ImportDraft {
        name,
        ingredients,
        instructions: recipe::sanitize_instructions(&draft.instructions.unwrap_or_default()),
        image_base64: None,
    })
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_name_when_build_draft_then_422() {
        let args = serde_json::json!({"name": "", "ingredients": [{"name": "x"}]});
        let result = build_draft_from_tool_args(&args);
        assert!(result.is_err());
        match result {
            Err(AppError::Llm(msg, code)) => {
                assert!(msg.contains("could not parse a recipe from input"));
                assert_eq!(code, "llm_parse_failed");
            }
            _ => panic!("expected Llm error, got {:?}", result),
        }
    }

    #[test]
    fn given_no_ingredients_when_build_draft_then_422() {
        let args = serde_json::json!({"name": "Pasta", "ingredients": []});
        let result = build_draft_from_tool_args(&args);
        assert!(result.is_err());
        match result {
            Err(AppError::Llm(msg, code)) => {
                assert!(msg.contains("could not parse a recipe from input"));
                assert_eq!(code, "llm_parse_failed");
            }
            _ => panic!("expected Llm error, got {:?}", result),
        }
    }

    #[test]
    fn given_all_empty_ingredient_names_when_build_draft_then_422() {
        let args =
            serde_json::json!({"name": "Pasta", "ingredients": [{"name": ""}, {"name": "  "}]});
        let result = build_draft_from_tool_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn given_valid_args_when_build_draft_then_returns_draft() {
        let args = serde_json::json!({
            "name": "Pasta",
            "ingredients": [{"name": "flour", "quantity": "200g"}],
            "instructions": "Boil"
        });
        let draft = build_draft_from_tool_args(&args).expect("should succeed");
        assert_eq!(draft.name, "Pasta");
        assert_eq!(draft.ingredients.len(), 1);
        assert_eq!(draft.ingredients[0].name, "flour");
        assert_eq!(draft.ingredients[0].quantity.as_deref(), Some("200g"));
        assert_eq!(draft.instructions, "Boil");
        assert!(draft.image_base64.is_none());
    }

    #[test]
    fn given_missing_instructions_when_build_draft_then_empty_string() {
        let args = serde_json::json!({
            "name": "Pasta",
            "ingredients": [{"name": "flour"}]
        });
        let draft = build_draft_from_tool_args(&args).expect("should succeed");
        assert_eq!(draft.instructions, "");
    }

    #[test]
    fn given_quantity_blank_when_build_draft_then_quantity_none() {
        let args = serde_json::json!({
            "name": "Pasta",
            "ingredients": [{"name": "flour", "quantity": "  "}]
        });
        let draft = build_draft_from_tool_args(&args).expect("should succeed");
        assert_eq!(draft.ingredients[0].quantity, None);
    }

    #[test]
    fn list_providers_includes_all_providers() {
        let providers = list_providers();
        let ids: Vec<&str> = providers.iter().map(|p| p.id.as_str()).collect();
        for expected in &[
            "openai",
            "anthropic",
            "gemini",
            "groq",
            "ollama",
            "deepseek",
            "xai",
            "custom",
        ] {
            assert!(
                ids.contains(expected),
                "expected provider '{expected}' not found in: {ids:?}"
            );
        }
    }

    #[test]
    fn list_providers_ollama_always_configured() {
        let providers = list_providers();
        let ollama = providers
            .iter()
            .find(|p| p.id == "ollama")
            .expect("ollama should exist");
        assert!(ollama.configured, "ollama should always be configured");
    }

    #[test]
    fn list_providers_custom_supports_custom_endpoint() {
        let providers = list_providers();
        let custom = providers
            .iter()
            .find(|p| p.id == "custom")
            .expect("custom should exist");
        assert!(custom.supports_custom_endpoint);
        assert!(custom.env_var.is_empty());
    }
}
