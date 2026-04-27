use std::path::Path;

use crate::core::Result;
use crate::grammar::Grammar;
use crate::analysis::ProcessedGrammar;

pub trait GrammarParser {
    type Output;
    fn parse_file(&self, path: &Path) -> Result<Self::Output>;
    fn parse_string(&self, source: &str, filename: &str) -> Result<Self::Output>;
}

pub trait SemanticAnalyzer {
    type Output;
    fn analyze(&self, grammar: &Grammar) -> Result<Self::Output>;
}

pub trait CodeGenerator {
    fn target_language(&self) -> &str;
    fn generate(&self, grammar: &ProcessedGrammar) -> Result<String>;
}
