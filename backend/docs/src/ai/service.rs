use super::claude_client::ClaudeClient;
use crate::common::ApiError;
use serde::{Deserialize, Serialize};

pub struct DocsAIService {
    claude: Option<ClaudeClient>,
}

impl DocsAIService {
    pub fn new() -> Self {
        Self {
            claude: ClaudeClient::from_env(),
        }
    }

    fn require_claude(&self) -> Result<&ClaudeClient, ApiError> {
        self.claude.as_ref().ok_or_else(|| {
            ApiError::bad_request(
                "AI features require ANTHROPIC_API_KEY to be configured",
            )
        })
    }

    pub async fn smart_compose(&self, context: &str) -> Result<String, ApiError> {
        let claude = self.require_claude()?;
        let prompt = format!(
            "Continue this text naturally with one sentence completion (just the continuation, no explanation):\n\n{context}"
        );
        claude.complete(&prompt, 100).await
    }

    pub async fn grammar_check(&self, text: &str) -> Result<Vec<GrammarIssue>, ApiError> {
        let claude = self.require_claude()?;
        let prompt = format!(
            "Check this text for grammar and style issues. Return a JSON array of objects with fields: offset (character position), length, message, suggestion. Only return the JSON array, no other text.\n\nText:\n{text}"
        );
        let response = claude.complete(&prompt, 500).await?;
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        let issues: Vec<GrammarIssue> = serde_json::from_str(cleaned).unwrap_or_default();
        Ok(issues)
    }

    pub async fn translate(&self, content: &str, target_lang: &str) -> Result<String, ApiError> {
        let claude = self.require_claude()?;
        let prompt = format!(
            "Translate this document content to {target_lang}. If it's JSON (TipTap format), translate only the text values while preserving the JSON structure. Return only the translated content.\n\nContent:\n{content}"
        );
        claude.complete(&prompt, 2000).await
    }

    pub async fn help_me_write(&self, description: &str) -> Result<String, ApiError> {
        let claude = self.require_claude()?;
        let prompt = format!(
            "Write a document based on this description: \"{description}\". Return the content as plain text formatted with markdown. Be comprehensive and well-structured."
        );
        claude.complete(&prompt, 1500).await
    }

    pub async fn summarize(&self, content: &str) -> Result<String, ApiError> {
        let claude = self.require_claude()?;
        let text = if content.len() > 3000 {
            &content[..3000]
        } else {
            content
        };
        let prompt = format!("Summarize this document in 3-5 bullet points:\n\n{text}");
        claude.complete(&prompt, 300).await
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GrammarIssue {
    pub offset: usize,
    pub length: usize,
    pub message: String,
    pub suggestion: String,
}
