pub mod rust;
pub mod go;
pub mod typescript;
pub mod python;
pub mod java;
pub mod c;
pub mod cpp;
pub mod treesitter;

use crate::analysis::ProcessedGrammar;
use anyhow::{anyhow, Result};
use std::path::Path;

pub trait CodeGenerator {
    fn lang_name(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn generate(&self, grammar: &ProcessedGrammar) -> String;
}

pub fn get_generator(lang: &str) -> Result<Box<dyn CodeGenerator>> {
    match lang {
        "rust" => Ok(Box::new(rust::RustGenerator)),
        "go" => Ok(Box::new(go::GoGenerator)),
        "typescript" | "ts" => Ok(Box::new(typescript::TypeScriptGenerator)),
        "python" | "py" => Ok(Box::new(python::PythonGenerator)),
        "java" => Ok(Box::new(java::JavaGenerator)),
        "c" => Ok(Box::new(c::CGenerator)),
        "cpp" | "c++" | "cplusplus" => Ok(Box::new(cpp::CppGenerator)),
        "treesitter" | "tree-sitter" => Ok(Box::new(treesitter::TreeSitterGenerator)),
        _ => Err(anyhow!("Unsupported language: {}. Supported: rust, go, typescript, python, java, c, cpp, treesitter", lang)),
    }
}

pub fn generate_output(grammar: &ProcessedGrammar, lang: &str, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;
    let generator = get_generator(lang)?;
    let code = generator.generate(grammar);
    let filename = format!("{}_parser{}", grammar.grammar.name.to_lowercase(), generator.file_extension());
    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, &code)?;
    tracing::info!("Generated {} parser: {}", generator.lang_name(), output_path.display());
    Ok(())
}

struct CodeWriter {
    lines: Vec<String>,
    indent_level: usize,
    indent_str: String,
}

impl CodeWriter {
    fn new(indent_str: &str) -> Self {
        Self {
            lines: Vec::new(),
            indent_level: 0,
            indent_str: indent_str.to_string(),
        }
    }

    fn line(&mut self, s: &str) {
        if s.is_empty() {
            self.lines.push(String::new());
        } else {
            self.lines.push(format!("{}{}", self.indent_str.repeat(self.indent_level), s));
        }
    }

    fn raw_line(&mut self, s: &str) {
        self.lines.push(s.to_string());
    }

    fn blank(&mut self) {
        self.lines.push(String::new());
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}

pub(crate) fn format_rule_name(name: &str) -> String {
    name.to_lowercase()
}

pub(crate) fn format_token_name(name: &str) -> String {
    name.to_uppercase()
}

pub(crate) fn snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

pub(crate) fn pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in name.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

pub(crate) fn camel_case(name: &str) -> String {
    let pascal = pascal_case(name);
    if let Some(c) = pascal.chars().next() {
        format!("{}{}", c.to_ascii_lowercase(), &pascal[c.len_utf8()..])
    } else {
        pascal
    }
}

pub(crate) fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

pub(crate) fn collect_tokens(grammar: &ProcessedGrammar) -> Vec<String> {
    let mut tokens = Vec::new();
    for rule in &grammar.grammar.rules {
        if rule.is_lexer_rule() {
            tokens.push(rule.name.clone());
        }
    }
    for tok in &grammar.implicit_tokens {
        if !tokens.contains(tok) {
            tokens.push(tok.clone());
        }
    }
    tokens
}
