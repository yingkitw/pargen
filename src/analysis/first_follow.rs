use crate::grammar::ast::*;
use std::collections::{HashMap, HashSet};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ProcessedGrammar {
    pub grammar: Grammar,
    pub first_sets: HashMap<String, HashSet<String>>,
    pub follow_sets: HashMap<String, HashSet<String>>,
    pub transformed_rules: Vec<Rule>,
    pub implicit_tokens: Vec<String>,
}

impl ProcessedGrammar {
    pub fn process(grammar: Grammar) -> Result<Self> {
        let implicit_tokens = Self::collect_implicit_tokens(&grammar);
        let first_sets = compute_first_sets(&grammar, &implicit_tokens);
        let follow_sets = compute_follow_sets(&grammar, &first_sets, &implicit_tokens);
        let transformed_rules = eliminate_left_recursion(&grammar.rules.iter().filter(|r| r.is_parser_rule()).cloned().collect::<Vec<_>>());

        Ok(Self {
            grammar,
            first_sets,
            follow_sets,
            transformed_rules,
            implicit_tokens,
        })
    }

    pub fn parser_rules(&self) -> Vec<&Rule> {
        self.transformed_rules.iter().chain(
            self.grammar.rules.iter().filter(|r| r.is_parser_rule() && !self.transformed_rules.iter().any(|tr| tr.name == r.name))
        ).collect()
    }

    pub fn lexer_rules(&self) -> Vec<&Rule> {
        self.grammar.lexer_rules()
    }

    pub fn all_tokens(&self) -> Vec<String> {
        let mut tokens = self.grammar.token_specs.clone();
        for rule in self.lexer_rules() {
            if !tokens.contains(&rule.name) {
                tokens.push(rule.name.clone());
            }
        }
        for tok in &self.implicit_tokens {
            if !tokens.contains(tok) {
                tokens.push(tok.clone());
            }
        }
        tokens
    }

    fn collect_implicit_tokens(grammar: &Grammar) -> Vec<String> {
        let mut implicit = Vec::new();
        let visitor = GrammarVisitor::new(grammar);
        let literals = visitor.collect_string_literals();
        for lit in &literals {
            let name = string_literal_to_token_name(lit);
            if !grammar.rules.iter().any(|r| r.name == name) {
                implicit.push(name);
            }
        }
        implicit
    }
}

pub fn string_literal_to_token_name(lit: &str) -> String {
    if lit.is_empty() {
        return "EMPTY".to_string();
    }
    let mut name = String::new();
    for c in lit.chars() {
        match c {
            '+' => name.push_str("Plus"),
            '-' => name.push_str("Minus"),
            '*' => name.push_str("Star"),
            '/' => name.push_str("Slash"),
            '=' => name.push_str("Assign"),
            '!' => name.push_str("Bang"),
            '<' => name.push_str("Lt"),
            '>' => name.push_str("Gt"),
            '&' => name.push_str("Amp"),
            '|' => name.push_str("Pipe"),
            '^' => name.push_str("Caret"),
            '%' => name.push_str("Percent"),
            '(' => name.push_str("Lparen"),
            ')' => name.push_str("Rparen"),
            '[' => name.push_str("Lbrack"),
            ']' => name.push_str("Rbrack"),
            '{' => name.push_str("Lbrace"),
            '}' => name.push_str("Rbrace"),
            ';' => name.push_str("Semi"),
            ':' => name.push_str("Colon"),
            ',' => name.push_str("Comma"),
            '.' => name.push_str("Dot"),
            '@' => name.push_str("At"),
            '#' => name.push_str("Hash"),
            '~' => name.push_str("Tilde"),
            '?' => name.push_str("Question"),
            ' ' | '\t' | '\n' | '\r' => {}
            c if c.is_alphanumeric() => name.push(c.to_ascii_uppercase()),
            _ => {
                name.push('U');
                for b in (c as u32).to_be_bytes() {
                    name.push_str(&format!("{:02X}", b));
                }
            }
        }
    }
    if name.is_empty() {
        "EMPTY".to_string()
    } else {
        name
    }
}

pub fn compute_first_sets(
    grammar: &Grammar,
    implicit_tokens: &[String],
) -> HashMap<String, HashSet<String>> {
    let mut first: HashMap<String, HashSet<String>> = HashMap::new();

    for rule in &grammar.rules {
        if rule.is_lexer_rule() {
            first.entry(rule.name.clone()).or_default().insert(rule.name.clone());
        }
    }
    for tok in implicit_tokens {
        first.entry(tok.clone()).or_default().insert(tok.clone());
    }
    for tok in &grammar.token_specs {
        first.entry(tok.clone()).or_default().insert(tok.clone());
    }
    for rule in &grammar.rules {
        if rule.is_parser_rule() {
            first.entry(rule.name.clone()).or_default();
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for rule in &grammar.rules {
            if rule.is_lexer_rule() {
                continue;
            }
            let rule_name = &rule.name;
            for alt in &rule.alternatives {
                let alt_first = first_of_alternative(&alt.elements, &first, implicit_tokens);
                let entry = first.entry(rule_name.clone()).or_default();
                for t in &alt_first {
                    if !entry.contains(t) {
                        entry.insert(t.clone());
                        changed = true;
                    }
                }
            }
        }
    }

    first
}

pub fn first_of_alternative(
    elements: &[Element],
    first_sets: &HashMap<String, HashSet<String>>,
    implicit_tokens: &[String],
) -> HashSet<String> {
    let mut result = HashSet::new();
    let mut all_nullable = true;

    for elem in elements {
        match &elem.kind {
            ElementKind::RuleRef(name) => {
                if let Some(s) = first_sets.get(name) {
                    result.extend(s.iter().cloned());
                }
                all_nullable = false;
                break;
            }
            ElementKind::TokenRef(name) => {
                result.insert(name.clone());
                all_nullable = false;
                break;
            }
            ElementKind::StringLiteral(lit) => {
                let tok_name = string_literal_to_token_name(lit);
                if implicit_tokens.contains(&tok_name) || !result.contains(&tok_name) {
                    result.insert(tok_name);
                }
                all_nullable = false;
                break;
            }
            ElementKind::Optional(block) | ElementKind::ZeroOrMore(block) => {
                for alt in &block.alternatives {
                    let alt_first = first_of_alternative(&alt.elements, first_sets, implicit_tokens);
                    result.extend(alt_first);
                }
            }
            ElementKind::OneOrMore(block) => {
                for alt in &block.alternatives {
                    let alt_first = first_of_alternative(&alt.elements, first_sets, implicit_tokens);
                    result.extend(alt_first);
                }
                all_nullable = false;
            }
            ElementKind::Group(block) => {
                let mut group_nullable = true;
                for alt in &block.alternatives {
                    let alt_first = first_of_alternative(&alt.elements, first_sets, implicit_tokens);
                    result.extend(alt_first);
                    if !alt.elements.is_empty() {
                        group_nullable = false;
                    }
                }
                if !group_nullable {
                    all_nullable = false;
                }
            }
            ElementKind::Dot => {
                result.insert("__DOT__".to_string());
                all_nullable = false;
                break;
            }
            ElementKind::Action(_) | ElementKind::Predicate(_) => {}
            ElementKind::Range(_, _) => {
                result.insert("__RANGE__".to_string());
                all_nullable = false;
                break;
            }
            ElementKind::CharSet(_) => {
                result.insert("__CHARSET__".to_string());
                all_nullable = false;
                break;
            }
            ElementKind::Not(_) => {
                result.insert("__NOT__".to_string());
                all_nullable = false;
                break;
            }
        }
    }

    if all_nullable && elements.is_empty() {
        result.insert("__EPSILON__".to_string());
    }

    result
}

pub fn compute_follow_sets(
    grammar: &Grammar,
    first_sets: &HashMap<String, HashSet<String>>,
    implicit_tokens: &[String],
) -> HashMap<String, HashSet<String>> {
    let mut follow: HashMap<String, HashSet<String>> = HashMap::new();

    if let Some(start) = grammar.start_rule() {
        follow.entry(start.name.clone()).or_default().insert("$EOF".to_string());
    }

    for rule in &grammar.rules {
        follow.entry(rule.name.clone()).or_default();
    }

    let mut changed = true;
    while changed {
        changed = false;
        for rule in &grammar.rules {
            if rule.is_lexer_rule() {
                continue;
            }
            for alt in &rule.alternatives {
                let elems = &alt.elements;
                for (i, elem) in elems.iter().enumerate() {
                    if let ElementKind::RuleRef(ref_name) = &elem.kind {
                        let rest: Vec<Element> = elems[i + 1..].to_vec();
                        let rest_first = first_of_alternative(
                            &rest,
                            first_sets,
                            implicit_tokens,
                        );

                        let parent_follow = if rest_first.contains("__EPSILON__") || rest.is_empty() {
                            follow.get(&rule.name).cloned().unwrap_or_default()
                        } else {
                            HashSet::new()
                        };

                        let follow_entry = follow.entry(ref_name.clone()).or_default();
                        for t in &rest_first {
                            if t != "__EPSILON__" && !follow_entry.contains(t) {
                                follow_entry.insert(t.clone());
                                changed = true;
                            }
                        }

                        for t in &parent_follow {
                            if !follow_entry.contains(t) {
                                follow_entry.insert(t.clone());
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    follow
}

use super::left_rec::eliminate_left_recursion;
