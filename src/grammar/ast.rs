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

    /// Return the names of all lexer modes referenced by this grammar,
    /// including the implied default mode (`DEFAULT_MODE`). Order is
    /// stable: default first, then named modes in alphabetical order.
    pub fn lexer_modes(&self) -> Vec<String> {
        use std::collections::BTreeSet;
        let mut named: BTreeSet<String> = BTreeSet::new();
        for rule in &self.rules {
            if let Some(m) = &rule.mode {
                named.insert(m.clone());
            }
            for cmd in &rule.commands {
                if matches!(cmd.name.as_str(), "pushMode" | "mode") {
                    if let Some(arg) = &cmd.arg {
                        named.insert(arg.clone());
                    }
                }
            }
        }
        let mut out = vec![DEFAULT_MODE.to_string()];
        out.extend(named);
        out
    }

    /// Return the lexer rules belonging to a given mode (`None` for
    /// the default mode).
    pub fn lexer_rules_in_mode(&self, mode: Option<&str>) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.is_lexer_rule() && r.mode.as_deref() == mode)
            .collect()
    }

    /// Walk the grammar and return every `{ ... }?` semantic predicate
    /// found in any alternative, at any nesting depth (inside
    /// `( ... )*`, `( ... )?`, `( ... )+`, `( ... )` groups).
    ///
    /// The reported indices are useful for diagnostics, MCP tooling,
    /// and (eventually) feeding a user-supplied predicate solver. The
    /// generated parsers themselves still treat predicates as no-ops;
    /// see `ARCHITECTURE.md` for the staged rollout plan.
    pub fn predicates(&self) -> Vec<PredicateInfo> {
        fn walk(elements: &[Element], out: &mut Vec<PredicateInfo>, rule: &str, alt_idx: usize) {
            for (elem_idx, elem) in elements.iter().enumerate() {
                match &elem.kind {
                    ElementKind::Predicate(src) => {
                        out.push(PredicateInfo {
                            rule_name: rule.to_string(),
                            alternative_index: alt_idx,
                            element_index: elem_idx,
                            source: src.clone(),
                        });
                    }
                    ElementKind::Optional(block)
                    | ElementKind::ZeroOrMore(block)
                    | ElementKind::OneOrMore(block)
                    | ElementKind::Group(block) => {
                        for inner_alt in &block.alternatives {
                            walk(&inner_alt.elements, out, rule, alt_idx);
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut out = Vec::new();
        for rule in &self.rules {
            for (alt_idx, alt) in rule.alternatives.iter().enumerate() {
                walk(&alt.elements, &mut out, &rule.name, alt_idx);
            }
        }
        out
    }

    /// Report every lexer channel in this grammar (declared or implicitly
    /// referenced) paired with the lexer-rule names that send tokens to
    /// it via `-> channel(NAME)`.
    ///
    /// - Declared channels come from the `channels { HIDDEN, ERROR }`
    ///   block (`Grammar::channel_specs`), in declaration order.
    /// - Channels only referenced via per-rule `-> channel(NAME)`
    ///   commands but never declared are appended in alphabetical
    ///   order, mirroring ANTLR's implicit-channel behavior.
    ///
    /// Today this is **reporting only** — generated lexers do not
    /// route tokens by channel. See `ARCHITECTURE.md` for the staged
    /// rollout plan.
    pub fn channels(&self) -> Vec<ChannelInfo> {
        let mut infos: Vec<ChannelInfo> = self
            .channel_specs
            .iter()
            .map(|name| ChannelInfo {
                name: name.clone(),
                assigned_rules: Vec::new(),
            })
            .collect();

        // Collect every channel name referenced by a lexer rule,
        // inserting any not-yet-declared channel before assigning rules.
        for rule in &self.rules {
            if !rule.is_lexer_rule() {
                continue;
            }
            if let Some(channel) = rule.channel() {
                let exists = infos.iter().any(|i| i.name == channel);
                if !exists {
                    infos.push(ChannelInfo {
                        name: channel.to_string(),
                        assigned_rules: Vec::new(),
                    });
                }
                if let Some(info) = infos.iter_mut().find(|i| i.name == channel) {
                    info.assigned_rules.push(rule.name.clone());
                }
            }
        }

        infos
    }
}

/// Name of the implicit default lexer mode used for rules that are not
/// declared inside any `mode NAME;` block.
pub const DEFAULT_MODE: &str = "DEFAULT_MODE";

/// One semantic predicate (`{ ... }?`) discovered in a grammar. The
/// struct is the public reporting shape — codegen still treats the
/// predicate as a no-op, but tooling can use these fields to surface
/// predicates to the user or to feed them to an external solver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredicateInfo {
    /// Name of the parser rule that contains the predicate.
    pub rule_name: String,
    /// Zero-based index of the alternative within the rule.
    pub alternative_index: usize,
    /// Zero-based index of the predicate element within the alternative.
    pub element_index: usize,
    /// Raw predicate source text (the contents of the `{ ... }` braces
    /// without the trailing `?`).
    pub source: String,
}

/// One lexer channel declared in a grammar (`channels { HIDDEN }`)
/// plus the lexer-rule names that route tokens to it via
/// `-> channel(NAME)`. Reporting only today; generated lexers do not
/// filter or expose this information yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// Channel name as declared in the grammar (`HIDDEN`, etc.).
    pub name: String,
    /// Lexer-rule names whose tokens are routed to this channel.
    /// Empty when no rule targets the channel (the channel is declared
    /// but unused).
    pub assigned_rules: Vec<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
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

    /// Return the target mode name if this command switches modes, else
    /// `None`. Recognises `pushMode(...)`, `popMode` (resets to default),
    /// and `mode(...)`.
    pub fn mode_target(&self) -> Option<ModeTarget> {
        match self.name.as_str() {
            "pushMode" | "mode" => self.arg.as_ref().map(|s| ModeTarget::Named(s.clone())),
            "popMode" => Some(ModeTarget::Default),
            _ => None,
        }
    }
}

/// How a lexer command changes the current mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModeTarget {
    /// Switch to the named mode.
    Named(String),
    /// Switch back to the implicit default mode.
    Default,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a tiny grammar by hand for unit-testing the predicate
    /// detector without going through the full lexer+parser pipeline.
    fn fixture(rule: Rule) -> Grammar {
        Grammar {
            name: "Test".to_string(),
            kind: GrammarKind::Combined,
            options: HashMap::new(),
            token_specs: Vec::new(),
            channel_specs: Vec::new(),
            actions: Vec::new(),
            rules: vec![rule],
        }
    }

    fn parser_rule(name: &str, alts: Vec<Alternative>) -> Rule {
        Rule {
            name: name.to_string(),
            is_fragment: false,
            modifiers: Vec::new(),
            return_type: None,
            locals_decl: None,
            throws: Vec::new(),
            alternatives: alts,
            commands: Vec::new(),
            mode: None,
        }
    }

    fn alt(elements: Vec<Element>) -> Alternative {
        Alternative {
            elements,
            label: None,
        }
    }

    fn token_ref(name: &str) -> Element {
        Element::new(ElementKind::TokenRef(name.to_string()))
    }

    fn predicate(src: &str) -> Element {
        Element::new(ElementKind::Predicate(src.to_string()))
    }

    fn group(elements: Vec<Element>) -> Element {
        Element::new(ElementKind::Group(AltBlock {
            alternatives: vec![alt(elements)],
        }))
    }

    #[test]
    fn predicates_empty_when_none() {
        let g = fixture(parser_rule(
            "expr",
            vec![alt(vec![token_ref("ID")])],
        ));
        assert!(g.predicates().is_empty());
    }

    #[test]
    fn predicates_top_level() {
        let g = fixture(parser_rule(
            "expr",
            vec![alt(vec![predicate("$a > 0"), token_ref("ID")])],
        ));
        let p = &g.predicates();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].rule_name, "expr");
        assert_eq!(p[0].alternative_index, 0);
        assert_eq!(p[0].element_index, 0);
        assert_eq!(p[0].source, "$a > 0");
    }

    #[test]
    fn predicates_nested_in_group() {
        let g = fixture(parser_rule(
            "expr",
            vec![alt(vec![group(vec![predicate("nested"), token_ref("ID")])])],
        ));
        let p = g.predicates();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].rule_name, "expr");
        assert_eq!(p[0].source, "nested");
    }

    #[test]
    fn predicates_count_across_alternatives() {
        let g = fixture(parser_rule(
            "expr",
            vec![
                alt(vec![predicate("a"), token_ref("A")]),
                alt(vec![token_ref("B"), predicate("b")]),
                alt(vec![predicate("c")]),
            ],
        ));
        let p = g.predicates();
        assert_eq!(p.len(), 3);
        // First predicate at element 0 of alt 0
        assert_eq!(p[0].alternative_index, 0);
        assert_eq!(p[0].element_index, 0);
        // Second predicate at element 1 of alt 1
        assert_eq!(p[1].alternative_index, 1);
        assert_eq!(p[1].element_index, 1);
        // Third predicate at element 0 of alt 2
        assert_eq!(p[2].alternative_index, 2);
        assert_eq!(p[2].element_index, 0);
    }

    // --- channels() detection tests --------------------------------------

    fn lexer_rule(name: &str, commands: Vec<LexerCommand>) -> Rule {
        let mut r = parser_rule(name, vec![alt(vec![token_ref("X")])]);
        r.commands = commands;
        r
    }

    fn grammar_with(channels: Vec<&str>, rules: Vec<Rule>) -> Grammar {
        Grammar {
            name: "Test".to_string(),
            kind: GrammarKind::Combined,
            options: HashMap::new(),
            token_specs: Vec::new(),
            channel_specs: channels.into_iter().map(|s| s.to_string()).collect(),
            actions: Vec::new(),
            rules,
        }
    }

    #[test]
    fn channels_empty_when_none_declared() {
        let g = grammar_with(vec![], vec![lexer_rule("WS", vec![LexerCommand::skip()])]);
        assert!(g.channels().is_empty());
    }

    #[test]
    fn channels_declared_with_no_assigned_rules() {
        let g = grammar_with(vec!["HIDDEN", "ERROR"], vec![lexer_rule("WS", vec![])]);
        let chans = g.channels();
        assert_eq!(chans.len(), 2);
        assert_eq!(chans[0].name, "HIDDEN");
        assert!(chans[0].assigned_rules.is_empty());
        assert_eq!(chans[1].name, "ERROR");
        assert!(chans[1].assigned_rules.is_empty());
    }

    #[test]
    fn channels_assigns_targeting_rule() {
        let g = grammar_with(
            vec!["HIDDEN"],
            vec![lexer_rule(
                "COMMENT",
                vec![LexerCommand::channel("HIDDEN")],
            )],
        );
        let chans = g.channels();
        assert_eq!(chans.len(), 1);
        assert_eq!(chans[0].name, "HIDDEN");
        assert_eq!(chans[0].assigned_rules, vec!["COMMENT"]);
    }

    #[test]
    fn channels_multiple_rules_to_same_channel() {
        let g = grammar_with(
            vec!["HIDDEN"],
            vec![
                lexer_rule("COMMENT", vec![LexerCommand::channel("HIDDEN")]),
                lexer_rule("WHITESPACE", vec![LexerCommand::channel("HIDDEN")]),
            ],
        );
        let chans = g.channels();
        assert_eq!(chans.len(), 1);
        assert_eq!(chans[0].assigned_rules, vec!["COMMENT", "WHITESPACE"]);
    }

    #[test]
    fn channels_implicit_when_undeclared() {
        // Rule targets a channel that was never declared via
        // `channels { ... }` — we still surface it (mirroring ANTLR's
        // implicit-channel behavior).
        let g = grammar_with(
            vec![],
            vec![lexer_rule(
                "COMMENT",
                vec![LexerCommand::channel("HIDDEN")],
            )],
        );
        let chans = g.channels();
        assert_eq!(chans.len(), 1);
        assert_eq!(chans[0].name, "HIDDEN");
        assert_eq!(chans[0].assigned_rules, vec!["COMMENT"]);
    }

    #[test]
    fn channels_declared_take_precedence_over_implicit_order() {
        // Declared channels come first (declaration order), implicit
        // channels append in the order they're first referenced.
        let g = grammar_with(
            vec!["ERROR"],
            vec![
                lexer_rule("BAD", vec![LexerCommand::channel("ERROR")]),
                lexer_rule("COMMENT", vec![LexerCommand::channel("HIDDEN")]),
            ],
        );
        let chans = g.channels();
        assert_eq!(chans.len(), 2);
        // Declared channel first.
        assert_eq!(chans[0].name, "ERROR");
        assert_eq!(chans[0].assigned_rules, vec!["BAD"]);
        // Implicit channel next, in the order first referenced.
        assert_eq!(chans[1].name, "HIDDEN");
        assert_eq!(chans[1].assigned_rules, vec!["COMMENT"]);
    }
}
