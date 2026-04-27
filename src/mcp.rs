//! MCP (Model Context Protocol) server for pargen
//!
//! This module exposes pargen functionality via the MCP protocol using rmcp.
//! It allows AI assistants and other tools to interact with pargen programmatically.

use crate::analysis::ProcessedGrammar;
use rmcp::{
    ErrorData as McpError,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// MCP server for pargen
#[derive(Clone)]
pub struct PargenServer {
    tool_router: ToolRouter<Self>,
}

impl PargenServer {
    /// Create a new pargen MCP server
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for PargenServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for parse_grammar tool
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ParseGrammarRequest {
    /// The grammar text to parse
    pub grammar_text: String,
    /// Filename for error reporting (default: "grammar.g4")
    #[serde(default = "default_filename")]
    pub filename: String,
}

fn default_filename() -> String {
    "grammar.g4".to_string()
}

/// Request for validate_grammar tool
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ValidateGrammarRequest {
    /// The grammar text to validate
    pub grammar_text: String,
}

/// Request for generate_code tool
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GenerateCodeRequest {
    /// The grammar text to generate code from
    pub grammar_text: String,
    /// Target language (rust, python, typescript, go, java, c, cpp, treesitter)
    pub target_language: String,
}

/// Request for get_grammar_info tool
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetGrammarInfoRequest {
    /// The grammar text to analyze
    pub grammar_text: String,
}

/// Response item for list_target_languages
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct LanguageInfo {
    pub name: String,
    pub description: String,
}

#[tool_router]
impl PargenServer {
    /// Parse a grammar specification and return the AST summary
    #[tool(description = "Parse a grammar specification and return the AST")]
    pub async fn parse_grammar(
        &self,
        params: Parameters<ParseGrammarRequest>,
    ) -> Result<CallToolResult, McpError> {
        match crate::parse_grammar_source(&params.0.grammar_text) {
            Ok(grammar) => {
                let result = serde_json::json!({
                    "success": true,
                    "grammar": {
                        "name": grammar.name,
                        "type": format!("{:?}", grammar.kind),
                        "rules_count": grammar.rules.len(),
                        "parser_rules_count": grammar.rules.iter().filter(|r| r.is_parser_rule()).count(),
                        "lexer_rules_count": grammar.rules.iter().filter(|r| r.is_lexer_rule()).count(),
                        "options": grammar.options,
                    }
                });
                Ok(CallToolResult::success(vec![Content::json(result)?]))
            }
            Err(e) => {
                let result = serde_json::json!({
                    "success": false,
                    "error": format!("{}", e)
                });
                Ok(CallToolResult::success(vec![Content::json(result)?]))
            }
        }
    }

    /// Validate a grammar specification
    #[tool(description = "Validate a grammar specification and return diagnostics")]
    pub async fn validate_grammar(
        &self,
        params: Parameters<ValidateGrammarRequest>,
    ) -> Result<CallToolResult, McpError> {
        match crate::parse_grammar_source(&params.0.grammar_text) {
            Ok(grammar) => {
                match ProcessedGrammar::process(grammar) {
                    Ok(_) => {
                        let result = serde_json::json!({
                            "success": true,
                            "has_errors": false,
                            "has_warnings": false,
                            "diagnostics": []
                        });
                        Ok(CallToolResult::success(vec![Content::json(result)?]))
                    }
                    Err(e) => {
                        let result = serde_json::json!({
                            "success": false,
                            "has_errors": true,
                            "has_warnings": false,
                            "error": format!("{}", e),
                            "diagnostics": [{
                                "severity": "Error",
                                "message": format!("{}", e)
                            }]
                        });
                        Ok(CallToolResult::success(vec![Content::json(result)?]))
                    }
                }
            }
            Err(e) => {
                let result = serde_json::json!({
                    "success": false,
                    "has_errors": true,
                    "has_warnings": false,
                    "error": format!("Parse error: {}", e),
                    "diagnostics": [{
                        "severity": "Error",
                        "message": format!("{}", e)
                    }]
                });
                Ok(CallToolResult::success(vec![Content::json(result)?]))
            }
        }
    }

    /// Generate parser code from a grammar specification
    #[tool(description = "Generate parser code from a grammar specification")]
    pub async fn generate_code(
        &self,
        params: Parameters<GenerateCodeRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Parse grammar
        let grammar = crate::parse_grammar_source(&params.0.grammar_text)
            .map_err(|e| McpError::invalid_params(format!("Parse error: {}", e), None))?;

        // Analyze grammar
        let processed = ProcessedGrammar::process(grammar)
            .map_err(|e| McpError::internal_error(format!("Analysis error: {}", e), None))?;

        // Generate code
        let generator = crate::codegen::get_generator(&params.0.target_language)
            .map_err(|e| McpError::invalid_params(format!("{}", e), None))?;

        let code = generator.generate(&processed);

        let result = serde_json::json!({
            "success": true,
            "code": code,
            "grammar_name": processed.grammar.name,
            "target_language": params.0.target_language
        });

        Ok(CallToolResult::success(vec![Content::json(result)?]))
    }

    /// Get detailed information about a parsed grammar
    #[tool(description = "Get detailed information about a parsed grammar")]
    pub async fn get_grammar_info(
        &self,
        params: Parameters<GetGrammarInfoRequest>,
    ) -> Result<CallToolResult, McpError> {
        let grammar = crate::parse_grammar_source(&params.0.grammar_text)
            .map_err(|e| McpError::invalid_params(format!("Parse error: {}", e), None))?;

        let parser_rules: Vec<String> = grammar.rules.iter()
            .filter(|r| r.is_parser_rule())
            .map(|r| r.name.clone())
            .collect();

        let lexer_rules: Vec<String> = grammar.rules.iter()
            .filter(|r| r.is_lexer_rule())
            .map(|r| r.name.clone())
            .collect();

        let result = serde_json::json!({
            "name": grammar.name,
            "type": format!("{:?}", grammar.kind),
            "parser_rules": parser_rules,
            "lexer_rules": lexer_rules,
            "options": grammar.options,
            "rules_count": grammar.rules.len(),
            "parser_rules_count": parser_rules.len(),
            "lexer_rules_count": lexer_rules.len(),
        });

        Ok(CallToolResult::success(vec![Content::json(result)?]))
    }

    /// List all supported target languages for code generation
    #[tool(description = "List all supported target languages for code generation")]
    pub async fn list_target_languages(
        &self,
        _params: Parameters<()>,
    ) -> Result<CallToolResult, McpError> {
        let languages = vec![
            LanguageInfo { name: "rust".to_string(), description: "Rust language".to_string() },
            LanguageInfo { name: "python".to_string(), description: "Python 3 language".to_string() },
            LanguageInfo { name: "typescript".to_string(), description: "TypeScript language".to_string() },
            LanguageInfo { name: "go".to_string(), description: "Go language".to_string() },
            LanguageInfo { name: "java".to_string(), description: "Java language".to_string() },
            LanguageInfo { name: "c".to_string(), description: "C language".to_string() },
            LanguageInfo { name: "cpp".to_string(), description: "C++ language".to_string() },
            LanguageInfo { name: "treesitter".to_string(), description: "Tree-sitter grammar.js".to_string() },
        ];

        let result = serde_json::json!({
            "languages": languages
        });

        Ok(CallToolResult::success(vec![Content::json(result)?]))
    }
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for PargenServer {}

/// Create a new pargen MCP server instance
pub fn create_server() -> PargenServer {
    PargenServer::new()
}
