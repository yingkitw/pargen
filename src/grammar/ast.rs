use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grammar {
    pub name: String,
    pub kind: GrammarKind,
    pub options: HashMap<String, String>,
    pub token_specs: Vec<String>,
    pub channel_specs: Vec<String>,
    pub actions: Vec<GrammarAction>,
    pub rules: Vec<Rule>,
}

impl Grammar {
    pub fn lexer_rules(&self) -> Vec<&Rule> {
        self.rules.iter().filter(|r| r.is_lexer_rule()).collect()
    }

    pub fn parser_rules(&self) -> Vec<&Rule> {
        self.rules.iter().filter(|r| r.is_parser_rule()).collect()
    }

    pub fn find_rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.name == name)
    }

    pub fn start_rule(&self) -> Option<&Rule> {
        self.parser_rules().first().copied()
    }

    pub fn all_token_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.token_specs.clone();
        for rule in self.lexer_rules() {
            if !names.contains(&rule.name) {
                names.push(rule.name.clone());
            }
        }
        names
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GrammarKind {
    Combined,
    Lexer,
    Parser,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarAction {
    pub scope: Option<String>,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub is_fragment: bool,
    pub modifiers: Vec<RuleModifier>,
    pub return_type: Option<String>,
    pub locals_decl: Option<String>,
    pub throws: Vec<String>,
    pub alternatives: Vec<Alternative>,
    pub commands: Vec<LexerCommand>,
}

impl Rule {
    pub fn is_lexer_rule(&self) -> bool {
        self.is_fragment || self.name.chars().next().map_or(false, |c| c.is_uppercase())
    }

    pub fn is_parser_rule(&self) -> bool {
        !self.is_lexer_rule()
    }

    pub fn is_skip(&self) -> bool {
        self.commands.iter().any(|c| c.name == "skip")
    }

    pub fn channel(&self) -> Option<&str> {
        self.commands.iter().find_map(|c| {
            if c.name == "channel" {
                c.arg.as_deref()
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RuleModifier {
    Public,
    Private,
    Protected,
    Fragment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternative {
    pub elements: Vec<Element>,
    pub label: Option<String>,
}

impl Alternative {
    pub fn new(elements: Vec<Element>) -> Self {
        Self {
            elements,
            label: None,
        }
    }

    pub fn labeled(label: &str, elements: Vec<Element>) -> Self {
        Self {
            elements,
            label: Some(label.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub kind: ElementKind,
    pub label: Option<String>,
}

impl Element {
    pub fn new(kind: ElementKind) -> Self {
        Self { kind, label: None }
    }

    pub fn labeled(label: &str, kind: ElementKind) -> Self {
        Self {
            kind,
            label: Some(label.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementKind {
    RuleRef(String),
    TokenRef(String),
    StringLiteral(String),
    Range(String, String),
    CharSet(CharSetDef),
    Optional(AltBlock),
    ZeroOrMore(AltBlock),
    OneOrMore(AltBlock),
    Group(AltBlock),
    Action(String),
    Predicate(String),
    Dot,
    Not(Box<ElementKind>),
}

impl ElementKind {
    pub fn is_rule_ref(&self) -> bool {
        matches!(self, ElementKind::RuleRef(_))
    }

    pub fn is_token_ref(&self) -> bool {
        matches!(self, ElementKind::TokenRef(_) | ElementKind::StringLiteral(_))
    }

    pub fn rule_name(&self) -> Option<&str> {
        match self {
            ElementKind::RuleRef(name) => Some(name),
            _ => None,
        }
    }

    pub fn token_name(&self) -> Option<&str> {
        match self {
            ElementKind::TokenRef(name) => Some(name),
            ElementKind::StringLiteral(_) => None,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AltBlock {
    pub alternatives: Vec<Alternative>,
}

impl AltBlock {
    pub fn new(alternatives: Vec<Alternative>) -> Self {
        Self { alternatives }
    }

    pub fn single(alt: Alternative) -> Self {
        Self {
            alternatives: vec![alt],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharSetDef {
    pub ranges: Vec<CharRange>,
    pub negated: bool,
}

impl CharSetDef {
    pub fn new(ranges: Vec<CharRange>, negated: bool) -> Self {
        Self { ranges, negated }
    }

    pub fn matches(&self, c: char) -> bool {
        let in_range = self.ranges.iter().any(|r| r.contains(c));
        if self.negated { !in_range } else { in_range }
    }

    pub fn to_regex_class(&self) -> String {
        let mut s = String::from("[");
        if self.negated {
            s.push('^');
        }
        for range in &self.ranges {
            if range.start == range.end {
                s.push_str(&escape_char_for_regex(range.start));
            } else {
                s.push_str(&escape_char_for_regex(range.start));
                s.push('-');
                s.push_str(&escape_char_for_regex(range.end));
            }
        }
        s.push(']');
        s
    }
}

fn escape_char_for_regex(c: char) -> String {
    match c {
        '\\' => "\\\\".to_string(),
        ']' => "\\]".to_string(),
        '[' => "\\[".to_string(),
        '^' => "\\^".to_string(),
        '-' => "\\-".to_string(),
        '.' => "\\.".to_string(),
        '*' => "\\*".to_string(),
        '+' => "\\+".to_string(),
        '?' => "\\?".to_string(),
        '(' => "\\(".to_string(),
        ')' => "\\)".to_string(),
        '{' => "\\{".to_string(),
        '}' => "\\}".to_string(),
        '|' => "\\|".to_string(),
        '$' => "\\$".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        c => c.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CharRange {
    pub start: char,
    pub end: char,
}

impl CharRange {
    pub fn single(c: char) -> Self {
        Self { start: c, end: c }
    }

    pub fn range(start: char, end: char) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, c: char) -> bool {
        c >= self.start && c <= self.end
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LexerCommand {
    pub name: String,
    pub arg: Option<String>,
}

impl LexerCommand {
    pub fn new(name: &str, arg: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            arg,
        }
    }

    pub fn skip() -> Self {
        Self::new("skip", None)
    }

    pub fn channel(name: &str) -> Self {
        Self::new("channel", Some(name.to_string()))
    }

    pub fn r#type(name: &str) -> Self {
        Self::new("type", Some(name.to_string()))
    }

    pub fn push_mode(name: &str) -> Self {
        Self::new("pushMode", Some(name.to_string()))
    }

    pub fn pop_mode() -> Self {
        Self::new("popMode", None)
    }

    pub fn mode(name: &str) -> Self {
        Self::new("mode", Some(name.to_string()))
    }
}

#[derive(Debug)]
pub struct GrammarVisitor<'a> {
    pub grammar: &'a Grammar,
    pub rule_index: HashMap<String, usize>,
}

impl<'a> GrammarVisitor<'a> {
    pub fn new(grammar: &'a Grammar) -> Self {
        let rule_index: HashMap<String, usize> = grammar
            .rules
            .iter()
            .enumerate()
            .map(|(i, r)| (r.name.clone(), i))
            .collect();
        Self {
            grammar,
            rule_index,
        }
    }

    pub fn collect_string_literals(&self) -> HashSet<String> {
        let mut literals = HashSet::new();
        for rule in &self.grammar.rules {
            for alt in &rule.alternatives {
                self.collect_literals_from_elements(&alt.elements, &mut literals);
            }
        }
        literals
    }

    fn collect_literals_from_elements(&self, elements: &[Element], literals: &mut HashSet<String>) {
        for elem in elements {
            match &elem.kind {
                ElementKind::StringLiteral(s) => {
                    literals.insert(s.clone());
                }
                ElementKind::Optional(block)
                | ElementKind::ZeroOrMore(block)
                | ElementKind::OneOrMore(block)
                | ElementKind::Group(block) => {
                    for alt in &block.alternatives {
                        self.collect_literals_from_elements(&alt.elements, literals);
                    }
                }
                _ => {}
            }
        }
    }
}
