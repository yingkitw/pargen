use crate::core::Error;
use crate::grammar::ast::*;
use crate::grammar::lexer::{G4Token, G4TokenKind};

pub struct G4Parser {
    tokens: Vec<G4Token>,
    pos: usize,
    source: String,
}

impl G4Parser {
    pub fn new(tokens: Vec<G4Token>, source: String) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
        }
    }

    pub fn parse(mut self) -> crate::core::Result<Grammar> {
        let grammar = self.parse_grammar()?;
        if !self.at_end() {
            let tok = self.current();
            return Err(Error::parser(
                tok.line, tok.col,
                format!("Unexpected token {:?} ('{}'), expected end of input", tok.kind, tok.text)
            ));
        }
        Ok(grammar)
    }

    fn current(&self) -> &G4Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens.last().unwrap()
        })
    }

    fn peek_kind(&self) -> G4TokenKind {
        self.current().kind
    }

    fn at_end(&self) -> bool {
        self.peek_kind() == G4TokenKind::Eof
    }

    fn advance(&mut self) -> G4Token {
        let tok = self.current().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: G4TokenKind) -> crate::core::Result<G4Token> {
        if self.peek_kind() == kind {
            Ok(self.advance())
        } else {
            let tok = self.current();
            Err(Error::parser(
                tok.line, tok.col,
                format!("Expected {} but found {:?} ('{}')", kind, tok.kind, tok.text)
            ))
        }
    }

    fn match_kind(&mut self, kind: G4TokenKind) -> Option<G4Token> {
        if self.peek_kind() == kind {
            Some(self.advance())
        } else {
            None
        }
    }

    fn parse_grammar(&mut self) -> crate::core::Result<Grammar> {
        let mut kind = GrammarKind::Combined;
        if self.peek_kind() == G4TokenKind::Lexer {
            self.advance();
            kind = GrammarKind::Lexer;
        } else if self.peek_kind() == G4TokenKind::Parser {
            self.advance();
            kind = GrammarKind::Parser;
        }

        self.expect(G4TokenKind::Grammar)?;
        let name_tok = self.expect(G4TokenKind::Id)?;
        let name = name_tok.text;
        self.expect(G4TokenKind::Semi)?;

        let mut grammar = Grammar {
            name,
            kind,
            options: std::collections::HashMap::new(),
            token_specs: Vec::new(),
            channel_specs: Vec::new(),
            actions: Vec::new(),
            rules: Vec::new(),
        };

        while !self.at_end() {
            match self.peek_kind() {
                G4TokenKind::Options => {
                    self.parse_options(&mut grammar)?;
                }
                G4TokenKind::Tokens => {
                    self.parse_tokens(&mut grammar)?;
                }
                G4TokenKind::Channels => {
                    self.parse_channels(&mut grammar)?;
                }
                G4TokenKind::Import => {
                    self.parse_import()?;
                }
                G4TokenKind::At => {
                    self.parse_action(&mut grammar)?;
                }
                G4TokenKind::Mode => {
                    self.parse_mode(&mut grammar)?;
                }
                _ => {
                    let rule = self.parse_rule()?;
                    grammar.rules.push(rule);
                }
            }
        }

        Ok(grammar)
    }

    #[cfg(test)]
    fn test_parse(source: &str) -> crate::core::Result<Grammar> {
        let lexer = super::lexer::G4Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Self::new(tokens, source.to_string());
        parser.parse()
    }

    fn parse_options(&mut self, grammar: &mut Grammar) -> crate::core::Result<()> {
        self.expect(G4TokenKind::Options)?;
        self.expect(G4TokenKind::Lbrace)?;
        while self.peek_kind() != G4TokenKind::Rbrace && !self.at_end() {
            let key = self.expect(G4TokenKind::Id)?.text;
            self.expect(G4TokenKind::Assign)?;
            let value = match self.peek_kind() {
                G4TokenKind::Id => self.advance().text,
                G4TokenKind::StringLit => self.advance().text,
                G4TokenKind::IntLit => self.advance().text,
                _ => {
                    let tok = self.current();
                    return Err(Error::parser(
                        tok.line, tok.col,
                        "Expected option value"
                    ))
                }
            };
            self.expect(G4TokenKind::Semi)?;
            grammar.options.insert(key, value);
        }
        self.expect(G4TokenKind::Rbrace)?;
        Ok(())
    }

    fn parse_tokens(&mut self, grammar: &mut Grammar) -> crate::core::Result<()> {
        self.expect(G4TokenKind::Tokens)?;
        self.expect(G4TokenKind::Lbrace)?;
        while self.peek_kind() != G4TokenKind::Rbrace && !self.at_end() {
            let id = self.expect(G4TokenKind::Id)?.text;
            grammar.token_specs.push(id);
            let _ = self.match_kind(G4TokenKind::Comma);
        }
        self.expect(G4TokenKind::Rbrace)?;
        Ok(())
    }

    fn parse_channels(&mut self, grammar: &mut Grammar) -> crate::core::Result<()> {
        self.expect(G4TokenKind::Channels)?;
        self.expect(G4TokenKind::Lbrace)?;
        while self.peek_kind() != G4TokenKind::Rbrace && !self.at_end() {
            let id = self.expect(G4TokenKind::Id)?.text;
            grammar.channel_specs.push(id);
            let _ = self.match_kind(G4TokenKind::Comma);
        }
        self.expect(G4TokenKind::Rbrace)?;
        Ok(())
    }

    fn parse_import(&mut self) -> crate::core::Result<()> {
        self.expect(G4TokenKind::Import)?;
        loop {
            self.expect(G4TokenKind::Id)?;
            if self.match_kind(G4TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(G4TokenKind::Semi)?;
        Ok(())
    }

    fn parse_action(&mut self, grammar: &mut Grammar) -> crate::core::Result<()> {
        self.expect(G4TokenKind::At)?;
        let scope = if self.peek_kind() == G4TokenKind::Id
            && self.tokens.get(self.pos + 1).map(|t| t.kind) == Some(G4TokenKind::Colon)
        {
            let s = self.advance().text;
            self.advance();
            Some(s)
        } else {
            None
        };
        let name = self.expect(G4TokenKind::Id)?.text;
        let content = self.expect(G4TokenKind::Action)?.text;
        grammar.actions.push(GrammarAction {
            scope,
            name,
            content,
        });
        Ok(())
    }

    fn parse_mode(&mut self, _grammar: &mut Grammar) -> crate::core::Result<()> {
        self.advance();
        let _name = self.expect(G4TokenKind::Id)?;
        self.expect(G4TokenKind::Semi)?;
        Ok(())
    }

    fn parse_rule(&mut self) -> crate::core::Result<Rule> {
        let mut modifiers = Vec::new();
        let mut is_fragment = false;

        while matches!(
            self.peek_kind(),
            G4TokenKind::Fragment | G4TokenKind::Protected | G4TokenKind::Public | G4TokenKind::Private
        ) {
            match self.peek_kind() {
                G4TokenKind::Fragment => {
                    self.advance();
                    is_fragment = true;
                    modifiers.push(RuleModifier::Fragment);
                }
                G4TokenKind::Protected => {
                    self.advance();
                    modifiers.push(RuleModifier::Protected);
                }
                G4TokenKind::Public => {
                    self.advance();
                    modifiers.push(RuleModifier::Public);
                }
                G4TokenKind::Private => {
                    self.advance();
                    modifiers.push(RuleModifier::Private);
                }
                _ => unreachable!(),
            }
        }

        let name = self.expect(G4TokenKind::Id)?.text;
        let is_lexer = is_fragment || name.chars().next().map_or(false, |c| c.is_uppercase());

        let mut return_type = None;
        let mut locals_decl = None;
        let mut throws = Vec::new();

        if !is_lexer {
            if self.match_kind(G4TokenKind::Lbrack).is_some() {
                let content = self.read_until_matching_bracket()?;
                return_type = Some(content);
                self.expect(G4TokenKind::Rbrack)?;
            }

            if self.match_kind(G4TokenKind::Returns).is_some() {
                self.expect(G4TokenKind::Lbrack)?;
                let content = self.read_until_matching_bracket()?;
                return_type = Some(content);
                self.expect(G4TokenKind::Rbrack)?;
            }

            if self.peek_kind() == G4TokenKind::Throws {
                self.advance();
                loop {
                    throws.push(self.expect(G4TokenKind::Id)?.text);
                    if self.match_kind(G4TokenKind::Comma).is_none() {
                        break;
                    }
                }
            }

            if self.peek_kind() == G4TokenKind::Locals {
                self.advance();
                self.expect(G4TokenKind::Lbrack)?;
                let content = self.read_until_matching_bracket()?;
                locals_decl = Some(content);
                self.expect(G4TokenKind::Rbrack)?;
            }

            while self.peek_kind() == G4TokenKind::At {
                self.advance();
                self.expect(G4TokenKind::Id)?;
                self.expect(G4TokenKind::Action)?;
            }
        }

        self.expect(G4TokenKind::Colon)?;

        let alternatives = if is_lexer {
            self.parse_lexer_alternatives()?
        } else {
            self.parse_parser_alternatives()?
        };

        let mut commands = Vec::new();
        if self.match_kind(G4TokenKind::Rarrow).is_some() {
            loop {
                let cmd_name = self.expect(G4TokenKind::Id)?.text;
                let arg = if self.match_kind(G4TokenKind::Lparen).is_some() {
                    let arg = self.expect(G4TokenKind::Id)?.text;
                    self.expect(G4TokenKind::Rparen)?;
                    Some(arg)
                } else {
                    None
                };
                commands.push(LexerCommand::new(&cmd_name, arg));
                if self.match_kind(G4TokenKind::Comma).is_none() {
                    break;
                }
            }
        }

        self.expect(G4TokenKind::Semi)?;

        Ok(Rule {
            name,
            is_fragment,
            modifiers,
            return_type,
            locals_decl,
            throws,
            alternatives,
            commands,
        })
    }

    fn read_until_matching_bracket(&mut self) -> crate::core::Result<String> {
        let mut content = String::new();
        let mut depth = 0;
        while !self.at_end() {
            match self.peek_kind() {
                G4TokenKind::Lbrack => {
                    depth += 1;
                    content.push_str(&self.advance().text);
                }
                G4TokenKind::Rbrack => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    content.push_str(&self.advance().text);
                }
                _ => {
                    content.push_str(&self.advance().text);
                }
            }
        }
        Ok(content.trim().to_string())
    }

    fn parse_lexer_alternatives(&mut self) -> crate::core::Result<Vec<Alternative>> {
        let mut alts = Vec::new();
        alts.push(self.parse_lexer_alternative()?);
        while self.match_kind(G4TokenKind::Pipe).is_some() {
            alts.push(self.parse_lexer_alternative()?);
        }
        Ok(alts)
    }

    fn parse_lexer_alternative(&mut self) -> crate::core::Result<Alternative> {
        let mut elements = Vec::new();
        while !self.at_end() && !matches!(self.peek_kind(), G4TokenKind::Pipe | G4TokenKind::Semi | G4TokenKind::Rarrow) {
            if let Some(elem) = self.parse_lexer_element()? {
                elements.push(elem);
            } else {
                break;
            }
        }
        Ok(Alternative::new(elements))
    }

    fn parse_lexer_element(&mut self) -> crate::core::Result<Option<Element>> {
        let kind = match self.peek_kind() {
            G4TokenKind::StringLit => {
                let tok = self.advance();
                Some(ElementKind::StringLiteral(tok.text))
            }
            G4TokenKind::Dot => {
                self.advance();
                Some(ElementKind::Dot)
            }
            G4TokenKind::Tilde => {
                self.advance();
                let inner = self.parse_lexer_atom()?;
                if let Some(inner) = inner {
                    Some(ElementKind::Not(Box::new(inner.kind)))
                } else {
                    let tok = self.current();
                    return Err(Error::parser(tok.line, tok.col, "Expected element after '~'"));
                }
            }
            G4TokenKind::CharsetContent => {
                let tok = self.advance();
                let charset = self.parse_charset_content(&tok.text)?;
                Some(ElementKind::CharSet(charset))
            }
            G4TokenKind::Lparen => {
                self.advance();
                let alts = self.parse_lexer_alternatives()?;
                self.expect(G4TokenKind::Rparen)?;
                let block = AltBlock::new(alts);
                let kind = self.parse_block_suffix(block)?;
                return Ok(Some(Element::new(kind)));
            }
            G4TokenKind::Id => {
                let tok = self.advance();
                let k = if tok.text.chars().next().map_or(false, |c| c.is_uppercase()) {
                    ElementKind::TokenRef(tok.text)
                } else {
                    ElementKind::RuleRef(tok.text)
                };
                Some(k)
            }
            G4TokenKind::Action => {
                self.advance();
                return Ok(None);
            }
            _ => return Ok(None),
        };
        let kind = self.parse_suffix(kind.unwrap())?;
        Ok(Some(Element::new(kind)))
    }

    fn parse_lexer_atom(&mut self) -> crate::core::Result<Option<Element>> {
        match self.peek_kind() {
            G4TokenKind::StringLit => {
                let tok = self.advance();
                Ok(Some(Element::new(ElementKind::StringLiteral(tok.text))))
            }
            G4TokenKind::CharsetContent => {
                let tok = self.advance();
                let charset = self.parse_charset_content(&tok.text)?;
                Ok(Some(Element::new(ElementKind::CharSet(charset))))
            }
            G4TokenKind::Dot => {
                self.advance();
                Ok(Some(Element::new(ElementKind::Dot)))
            }
            G4TokenKind::Id => {
                let tok = self.advance();
                let kind = if tok.text.chars().next().map_or(false, |c| c.is_uppercase()) {
                    ElementKind::TokenRef(tok.text)
                } else {
                    ElementKind::RuleRef(tok.text)
                };
                Ok(Some(Element::new(kind)))
            }
            _ => Ok(None),
        }
    }

    fn parse_suffix(&mut self, kind: ElementKind) -> crate::core::Result<ElementKind> {
        match self.peek_kind() {
            G4TokenKind::Question => {
                self.advance();
                let block = AltBlock::single(Alternative::new(vec![Element::new(kind)]));
                Ok(ElementKind::Optional(block))
            }
            G4TokenKind::Star => {
                self.advance();
                let block = AltBlock::single(Alternative::new(vec![Element::new(kind)]));
                Ok(ElementKind::ZeroOrMore(block))
            }
            G4TokenKind::Plus => {
                self.advance();
                let block = AltBlock::single(Alternative::new(vec![Element::new(kind)]));
                Ok(ElementKind::OneOrMore(block))
            }
            _ => Ok(kind),
        }
    }

    fn parse_block_suffix(&mut self, block: AltBlock) -> crate::core::Result<ElementKind> {
        match self.peek_kind() {
            G4TokenKind::Question => {
                self.advance();
                let _greedy = self.match_kind(G4TokenKind::Question);
                Ok(ElementKind::Optional(block))
            }
            G4TokenKind::Star => {
                self.advance();
                let _greedy = self.match_kind(G4TokenKind::Question);
                Ok(ElementKind::ZeroOrMore(block))
            }
            G4TokenKind::Plus => {
                self.advance();
                let _greedy = self.match_kind(G4TokenKind::Question);
                Ok(ElementKind::OneOrMore(block))
            }
            _ => Ok(ElementKind::Group(block)),
        }
    }

    fn parse_charset_content(&self, content: &str) -> crate::core::Result<CharSetDef> {
        let mut ranges = Vec::new();
        let mut negated = false;
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if c.is_whitespace() {
                i += 1;
                continue;
            }
            if c == '~' || c == '^' {
                negated = true;
                i += 1;
                continue;
            }
            if c == '\\' && i + 1 < chars.len() {
                let escaped = chars[i + 1];
                let ch = match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '-' => '-',
                    ']' => ']',
                    _ => escaped,
                };
                if i + 2 < chars.len() && chars[i + 2] == '-' && i + 3 < chars.len() {
                    let end_ch = if chars[i + 3] == '\\' && i + 4 < chars.len() {
                        match chars[i + 4] {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '\\' => '\\',
                            '-' => '-',
                            ']' => ']',
                            _ => chars[i + 4],
                        }
                    } else {
                        chars[i + 3]
                    };
                    ranges.push(CharRange::range(ch, end_ch));
                    i += if chars[i + 3] == '\\' { 5 } else { 4 };
                } else {
                    ranges.push(CharRange::single(ch));
                    i += 2;
                }
                continue;
            }
            if i + 1 < chars.len() && chars[i + 1] == '-' && i + 2 < chars.len() {
                let end_ch = if chars[i + 2] == '\\' && i + 3 < chars.len() {
                    match chars[i + 3] {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        _ => chars[i + 3],
                    }
                } else {
                    chars[i + 2]
                };
                ranges.push(CharRange::range(c, end_ch));
                i += if chars[i + 2] == '\\' { 4 } else { 3 };
            } else {
                ranges.push(CharRange::single(c));
                i += 1;
            }
        }

        Ok(CharSetDef::new(ranges, negated))
    }

    fn parse_parser_alternatives(&mut self) -> crate::core::Result<Vec<Alternative>> {
        let mut alts = Vec::new();
        alts.push(self.parse_parser_alternative()?);
        while self.match_kind(G4TokenKind::Pipe).is_some() {
            alts.push(self.parse_parser_alternative()?);
        }
        Ok(alts)
    }

    fn parse_parser_alternative(&mut self) -> crate::core::Result<Alternative> {
        let mut elements = Vec::new();
        while !self.at_end()
            && !matches!(
                self.peek_kind(),
                G4TokenKind::Pipe | G4TokenKind::Semi | G4TokenKind::Rparen
            )
        {
            if let Some(elem) = self.parse_parser_element()? {
                elements.push(elem);
            } else {
                break;
            }
        }
        let mut alt = Alternative::new(elements);
        if self.match_kind(G4TokenKind::Hash).is_some() {
            alt.label = Some(self.expect(G4TokenKind::Id)?.text);
        }
        Ok(alt)
    }

    fn parse_parser_element(&mut self) -> crate::core::Result<Option<Element>> {
        let mut label = None;
        if self.peek_kind() == G4TokenKind::Id {
            let next = self.tokens.get(self.pos + 1);
            if next.map_or(false, |t| {
                t.kind == G4TokenKind::Assign || t.kind == G4TokenKind::PlusAssign
            }) {
                label = Some(self.advance().text);
                self.advance();
            }
        }

        match self.peek_kind() {
            G4TokenKind::Id => {
                let tok = self.advance();
                let kind = if tok.text.chars().next().map_or(false, |c| c.is_uppercase()) {
                    ElementKind::TokenRef(tok.text)
                } else {
                    ElementKind::RuleRef(tok.text)
                };
                Ok(Some(Element { kind, label }))
            }
            G4TokenKind::StringLit => {
                let tok = self.advance();
                Ok(Some(Element {
                    kind: ElementKind::StringLiteral(tok.text),
                    label,
                }))
            }
            G4TokenKind::Dot => {
                self.advance();
                Ok(Some(Element {
                    kind: ElementKind::Dot,
                    label,
                }))
            }
            G4TokenKind::Tilde => {
                self.advance();
                if let Some(inner) = self.parse_lexer_atom()? {
                    Ok(Some(Element::new(ElementKind::Not(Box::new(inner.kind)))))
                } else {
                    let tok = self.current();
                    Err(Error::parser(tok.line, tok.col, "Expected element after '~'"))
                }
            }
            G4TokenKind::CharsetContent => {
                let tok = self.advance();
                let charset = self.parse_charset_content(&tok.text)?;
                Ok(Some(Element::new(ElementKind::CharSet(charset))))
            }
            G4TokenKind::Lparen => {
                self.advance();
                let alts = self.parse_parser_alternatives()?;
                self.expect(G4TokenKind::Rparen)?;
                let block = AltBlock::new(alts);
                let kind = self.parse_block_suffix(block)?;
                Ok(Some(Element { kind, label }))
            }
            G4TokenKind::Action => {
                let tok = self.advance();
                if self.match_kind(G4TokenKind::Question).is_some() {
                    Ok(Some(Element::new(ElementKind::Predicate(tok.text))))
                } else {
                    Ok(Some(Element::new(ElementKind::Action(tok.text))))
                }
            }
            G4TokenKind::Lbrace => {
                self.advance();
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_fails() {
        let result = G4Parser::test_parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_simple_grammar() {
        let grammar = G4Parser::test_parse("grammar Calc;").unwrap();
        assert_eq!(grammar.name, "Calc");
        assert_eq!(grammar.kind, GrammarKind::Combined);
        assert!(grammar.rules.is_empty());
    }

    #[test]
    fn test_parse_lexer_grammar() {
        let grammar = G4Parser::test_parse("lexer grammar MyLexer;").unwrap();
        assert_eq!(grammar.name, "MyLexer");
        assert_eq!(grammar.kind, GrammarKind::Lexer);
    }

    #[test]
    fn test_parse_parser_grammar() {
        let grammar = G4Parser::test_parse("parser grammar MyParser;").unwrap();
        assert_eq!(grammar.name, "MyParser");
        assert_eq!(grammar.kind, GrammarKind::Parser);
    }

    #[test]
    fn test_parse_single_rule() {
        let grammar = G4Parser::test_parse("grammar Test; start: EOF;").unwrap();
        assert_eq!(grammar.rules.len(), 1);
        assert_eq!(grammar.rules[0].name, "start");
    }

    #[test]
    fn test_parse_multiple_rules() {
        let source = r#"grammar Test;
expr: term '+' term;
term: factor;
factor: NUMBER;
NUMBER: [0-9]+;
"#;
        let grammar = G4Parser::test_parse(source).unwrap();
        assert_eq!(grammar.rules.len(), 4);
        assert_eq!(grammar.rules[0].name, "expr");
        assert_eq!(grammar.rules[1].name, "term");
        assert_eq!(grammar.rules[2].name, "factor");
        assert_eq!(grammar.rules[3].name, "NUMBER");
    }

    #[test]
    fn test_parse_lexer_rule_with_charset() {
        let grammar = G4Parser::test_parse("grammar Test; DIGIT: [0-9];").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.name, "DIGIT");
        assert!(rule.is_lexer_rule());
        assert_eq!(rule.alternatives.len(), 1);
    }

    #[test]
    fn test_parse_rule_with_alternatives() {
        let grammar = G4Parser::test_parse("grammar Test; value: NUMBER | STRING;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives.len(), 2);
    }

    #[test]
    fn test_parse_rule_with_optional() {
        let grammar = G4Parser::test_parse("grammar Test; decl: type ID ('=' expr)?;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives.len(), 1);
        assert_eq!(rule.alternatives[0].elements.len(), 3);
    }

    #[test]
    fn test_parse_rule_with_star() {
        let grammar = G4Parser::test_parse("grammar Test; list: (item)*;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives.len(), 1);
        assert!(matches!(rule.alternatives[0].elements[0].kind, ElementKind::ZeroOrMore(_)));
    }

    #[test]
    fn test_parse_rule_with_plus() {
        let grammar = G4Parser::test_parse("grammar Test; list: (item)+;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives.len(), 1);
        assert!(matches!(rule.alternatives[0].elements[0].kind, ElementKind::OneOrMore(_)));
    }

    #[test]
    fn test_parse_rule_with_group() {
        let grammar = G4Parser::test_parse("grammar Test; expr: term (('+'|'-') term)*;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives.len(), 1);
        assert_eq!(rule.alternatives[0].elements.len(), 2);
    }

    #[test]
    fn test_parse_fragment_rule() {
        let grammar = G4Parser::test_parse("grammar Test; fragment DIGIT: [0-9];").unwrap();
        let rule = &grammar.rules[0];
        assert!(rule.is_fragment);
        assert!(rule.is_lexer_rule());
    }

    #[test]
    fn test_parse_rule_with_skip_command() {
        let grammar = G4Parser::test_parse("grammar Test; WS: [ \\t\\n]+ -> skip;").unwrap();
        let rule = &grammar.rules[0];
        assert!(rule.is_skip());
    }

    #[test]
    fn test_parse_rule_with_channel_command() {
        let grammar = G4Parser::test_parse("grammar Test; COMMENT: '//' ~[\\n]* -> channel(HIDDEN);").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.channel(), Some("HIDDEN"));
    }

    #[test]
    fn test_parse_import() {
        let grammar = G4Parser::test_parse("grammar Test; import CommonLexer; start: EOF;").unwrap();
        assert_eq!(grammar.name, "Test");
    }

    #[test]
    fn test_parse_named_action() {
        let grammar = G4Parser::test_parse(r#"grammar Test;
@header { package foo; }
start: EOF;
"#).unwrap();
        assert_eq!(grammar.actions.len(), 1);
        assert_eq!(grammar.actions[0].name, "header");
        assert_eq!(grammar.actions[0].content, "package foo;");
    }

    #[test]
    fn test_parse_labeled_element() {
        let grammar = G4Parser::test_parse("grammar Test; expr: left=term '+' right=term;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives[0].elements[0].label, Some("left".to_string()));
        assert_eq!(rule.alternatives[0].elements[2].label, Some("right".to_string()));
    }

    #[test]
    fn test_parse_list_label() {
        let grammar = G4Parser::test_parse("grammar Test; args: arg+=ID (',' arg+=ID)*;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives[0].elements[0].label, Some("arg".to_string()));
    }

    #[test]
    fn test_parse_alternative_label() {
        let grammar = G4Parser::test_parse("grammar Test; expr: term #add | factor #mul;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives[0].label, Some("add".to_string()));
        assert_eq!(rule.alternatives[1].label, Some("mul".to_string()));
    }

    #[test]
    fn test_parse_rule_with_returns() {
        let grammar = G4Parser::test_parse("grammar Test; expr returns [int value]: term;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.return_type, Some("int value".to_string()));
    }

    #[test]
    fn test_parse_rule_with_locals() {
        let grammar = G4Parser::test_parse("grammar Test; expr locals [int count = 0]: term;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.locals_decl, Some("int count = 0".to_string()));
    }

    #[test]
    fn test_parse_not_predicate() {
        let grammar = G4Parser::test_parse("grammar Test; expr: ~'(' term;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives[0].elements.len(), 2);
    }

    #[test]
    fn test_parse_predicate() {
        let grammar = G4Parser::test_parse("grammar Test; expr: { $a > 0 }? term;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives[0].elements.len(), 2);
        assert!(matches!(rule.alternatives[0].elements[0].kind, ElementKind::Predicate(_)));
    }

    #[test]
    fn test_unexpected_token_error() {
        let result = G4Parser::test_parse("grammar Test start");
        assert!(result.is_err());
    }

    #[test]
    fn test_trailing_tokens_error() {
        let result = G4Parser::test_parse("grammar Test; start: EOF; extra");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_dot_wildcard() {
        let grammar = G4Parser::test_parse("grammar Test; ANY: . ;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives[0].elements.len(), 1);
        assert!(matches!(rule.alternatives[0].elements[0].kind, ElementKind::Dot));
    }

    #[test]
    fn test_parse_greedy_optional() {
        let grammar = G4Parser::test_parse("grammar Test; opt: (item)??;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives.len(), 1);
        assert!(matches!(rule.alternatives[0].elements[0].kind, ElementKind::Optional(_)));
    }

    #[test]
    fn test_parse_greedy_star() {
        let grammar = G4Parser::test_parse("grammar Test; list: (item)*?;").unwrap();
        let rule = &grammar.rules[0];
        assert_eq!(rule.alternatives.len(), 1);
        assert!(matches!(rule.alternatives[0].elements[0].kind, ElementKind::ZeroOrMore(_)));
    }
}
