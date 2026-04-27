#[derive(Debug, Clone, Copy, PartialEq)]
pub enum G4TokenKind {
    Grammar,
    Lexer,
    Parser,
    Fragment,
    Protected,
    Public,
    Private,
    Returns,
    Locals,
    Throws,
    Options,
    Tokens,
    Channels,
    Import,
    Mode,

    Id,
    StringLit,
    IntLit,

    Colon,
    Semi,
    Comma,
    Dot,
    DotDot,
    Assign,
    PlusAssign,
    Question,
    Star,
    Plus,
    Tilde,
    Pipe,
    Rarrow,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Lbrack,
    Rbrack,
    Hash,
    At,
    Bang,
    Dollar,
    Caret,

    Action,
    CharsetContent,

    Eof,
}

impl std::fmt::Display for G4TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            G4TokenKind::Grammar => write!(f, "'grammar'"),
            G4TokenKind::Lexer => write!(f, "'lexer'"),
            G4TokenKind::Parser => write!(f, "'parser'"),
            G4TokenKind::Fragment => write!(f, "'fragment'"),
            G4TokenKind::Protected => write!(f, "'protected'"),
            G4TokenKind::Public => write!(f, "'public'"),
            G4TokenKind::Private => write!(f, "'private'"),
            G4TokenKind::Returns => write!(f, "'returns'"),
            G4TokenKind::Locals => write!(f, "'locals'"),
            G4TokenKind::Throws => write!(f, "'throws'"),
            G4TokenKind::Options => write!(f, "'options'"),
            G4TokenKind::Tokens => write!(f, "'tokens'"),
            G4TokenKind::Channels => write!(f, "'channels'"),
            G4TokenKind::Import => write!(f, "'import'"),
            G4TokenKind::Mode => write!(f, "'mode'"),
            G4TokenKind::Id => write!(f, "identifier"),
            G4TokenKind::StringLit => write!(f, "string literal"),
            G4TokenKind::IntLit => write!(f, "integer literal"),
            G4TokenKind::Colon => write!(f, "':'"),
            G4TokenKind::Semi => write!(f, "';'"),
            G4TokenKind::Comma => write!(f, "','"),
            G4TokenKind::Dot => write!(f, "'.'"),
            G4TokenKind::DotDot => write!(f, "'..'"),
            G4TokenKind::Assign => write!(f, "'='"),
            G4TokenKind::PlusAssign => write!(f, "'+='"),
            G4TokenKind::Question => write!(f, "'?'"),
            G4TokenKind::Star => write!(f, "'*'"),
            G4TokenKind::Plus => write!(f, "'+'"),
            G4TokenKind::Tilde => write!(f, "'~'"),
            G4TokenKind::Pipe => write!(f, "'|'"),
            G4TokenKind::Rarrow => write!(f, "'->'"),
            G4TokenKind::Lparen => write!(f, "'('"),
            G4TokenKind::Rparen => write!(f, "')'"),
            G4TokenKind::Lbrace => write!(f, "'{{'"),
            G4TokenKind::Rbrace => write!(f, "'}}'"),
            G4TokenKind::Lbrack => write!(f, "'['"),
            G4TokenKind::Rbrack => write!(f, "']'"),
            G4TokenKind::Hash => write!(f, "'#'"),
            G4TokenKind::At => write!(f, "'@'"),
            G4TokenKind::Bang => write!(f, "'!'"),
            G4TokenKind::Dollar => write!(f, "'$'"),
            G4TokenKind::Caret => write!(f, "'^'"),
            G4TokenKind::Action => write!(f, "action"),
            G4TokenKind::CharsetContent => write!(f, "charset"),
            G4TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct G4Token {
    pub kind: G4TokenKind,
    pub text: String,
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl G4Token {
    pub fn new(kind: G4TokenKind, text: &str, line: usize, col: usize, offset: usize) -> Self {
        Self {
            kind,
            text: text.to_string(),
            line,
            col,
            offset,
        }
    }

    pub fn eof(line: usize, col: usize, offset: usize) -> Self {
        Self {
            kind: G4TokenKind::Eof,
            text: String::new(),
            line,
            col,
            offset,
        }
    }
}

static KEYWORDS: &[(&str, G4TokenKind)] = &[
    ("grammar", G4TokenKind::Grammar),
    ("lexer", G4TokenKind::Lexer),
    ("parser", G4TokenKind::Parser),
    ("fragment", G4TokenKind::Fragment),
    ("protected", G4TokenKind::Protected),
    ("public", G4TokenKind::Public),
    ("private", G4TokenKind::Private),
    ("returns", G4TokenKind::Returns),
    ("locals", G4TokenKind::Locals),
    ("throws", G4TokenKind::Throws),
    ("options", G4TokenKind::Options),
    ("tokens", G4TokenKind::Tokens),
    ("channels", G4TokenKind::Channels),
    ("import", G4TokenKind::Import),
    ("mode", G4TokenKind::Mode),
];

pub struct G4Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    tokens: Vec<G4Token>,
}

impl G4Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<G4Token>, String> {
        while self.pos < self.input.len() {
            self.skip_whitespace_and_comments()?;
            if self.pos >= self.input.len() {
                break;
            }

            let c = self.input[self.pos];
            let offset = self.pos;
            let line = self.line;
            let col = self.col;

            if c == '\'' {
                let tok = self.read_string_literal()?;
                self.tokens.push(tok);
            } else if c == '{' {
                let tok = self.read_action_block(offset, line, col)?;
                self.tokens.push(tok);
            } else if c == '[' {
                let saved_pos = self.pos;
                let saved_line = self.line;
                let saved_col = self.col;
                let content = self.read_bracket_content();
                if self.looks_like_charset(&content) {
                    self.tokens.push(G4Token::new(
                        G4TokenKind::CharsetContent,
                        &content,
                        saved_line,
                        saved_col,
                        saved_pos,
                    ));
                } else {
                    self.tokens.push(G4Token::new(G4TokenKind::Lbrack, "[", saved_line, saved_col, saved_pos));
                    self.tokens.push(G4Token::new(
                        G4TokenKind::Id,
                        &content.trim(),
                        saved_line,
                        saved_col + 1,
                        saved_pos + 1,
                    ));
                    let end_pos = self.pos - 1;
                    self.tokens.push(G4Token::new(G4TokenKind::Rbrack, "]", self.line, self.col - 1, end_pos));
                }
            } else if c.is_ascii_digit() {
                let tok = self.read_number(offset, line, col);
                self.tokens.push(tok);
            } else if c.is_alphabetic() || c == '_' {
                let tok = self.read_identifier(offset, line, col);
                self.tokens.push(tok);
            } else {
                let tok = self.read_symbol(offset, line, col)?;
                self.tokens.push(tok);
            }
        }

        self.tokens.push(G4Token::eof(self.line, self.col, self.pos));
        Ok(self.tokens)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let c = self.input[self.pos];
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            if c.is_whitespace() {
                self.advance();
            } else if c == '/' && self.peek_at(1) == Some('/') {
                while self.pos < self.input.len() && self.input[self.pos] != '\n' {
                    self.advance();
                }
            } else if c == '/' && self.peek_at(1) == Some('*') {
                self.advance();
                self.advance();
                let mut depth = 1;
                while self.pos < self.input.len() && depth > 0 {
                    if self.input[self.pos] == '/' && self.peek_at(1) == Some('*') {
                        depth += 1;
                        self.advance();
                        self.advance();
                    } else if self.input[self.pos] == '*' && self.peek_at(1) == Some('/') {
                        depth -= 1;
                        self.advance();
                        self.advance();
                    } else {
                        self.advance();
                    }
                }
                if depth > 0 {
                    return Err("Unterminated block comment".to_string());
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn read_string_literal(&mut self) -> Result<G4Token, String> {
        let offset = self.pos;
        let line = self.line;
        let col = self.col;
        self.advance();
        let mut text = String::new();
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                '\'' => {
                    if self.peek_at(1) == Some('\'') {
                        text.push('\'');
                        self.advance();
                        self.advance();
                    } else {
                        self.advance();
                        return Ok(G4Token::new(G4TokenKind::StringLit, &text, line, col, offset));
                    }
                }
                '\\' => {
                    self.advance();
                    if self.pos < self.input.len() {
                        let c = self.input[self.pos];
                        match c {
                            'n' => text.push('\n'),
                            'r' => text.push('\r'),
                            't' => text.push('\t'),
                            '\\' => text.push('\\'),
                            '\'' => text.push('\''),
                            _ => text.push(c),
                        }
                        self.advance();
                    }
                }
                '\n' => {
                    return Err(format!("Unterminated string literal at line {}", line));
                }
                c => {
                    text.push(c);
                    self.advance();
                }
            }
        }
        Err(format!("Unterminated string literal at line {}", line))
    }

    fn read_action_block(&mut self, offset: usize, line: usize, col: usize) -> Result<G4Token, String> {
        let mut depth = 0;
        let mut content = String::new();
        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            match c {
                '{' => {
                    depth += 1;
                    if depth > 1 {
                        content.push(c);
                    }
                    self.advance();
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        self.advance();
                        return Ok(G4Token::new(G4TokenKind::Action, content.trim(), line, col, offset));
                    }
                    content.push(c);
                    self.advance();
                }
                '\'' => {
                    content.push('\'');
                    self.advance();
                    while self.pos < self.input.len() && self.input[self.pos] != '\'' {
                        if self.input[self.pos] == '\\' {
                            content.push(self.input[self.pos]);
                            self.advance();
                            if self.pos < self.input.len() {
                                content.push(self.input[self.pos]);
                                self.advance();
                            }
                        } else {
                            content.push(self.input[self.pos]);
                            self.advance();
                        }
                    }
                    if self.pos < self.input.len() {
                        content.push('\'');
                        self.advance();
                    }
                }
                '"' => {
                    content.push('"');
                    self.advance();
                    while self.pos < self.input.len() && self.input[self.pos] != '"' {
                        if self.input[self.pos] == '\\' {
                            content.push(self.input[self.pos]);
                            self.advance();
                            if self.pos < self.input.len() {
                                content.push(self.input[self.pos]);
                                self.advance();
                            }
                        } else {
                            content.push(self.input[self.pos]);
                            self.advance();
                        }
                    }
                    if self.pos < self.input.len() {
                        content.push('"');
                        self.advance();
                    }
                }
                '/' if self.peek_at(1) == Some('*') => {
                    content.push('/');
                    self.advance();
                    content.push('*');
                    self.advance();
                    while self.pos < self.input.len() {
                        if self.input[self.pos] == '*' && self.peek_at(1) == Some('/') {
                            content.push('*');
                            self.advance();
                            content.push('/');
                            self.advance();
                            break;
                        }
                        content.push(self.input[self.pos]);
                        self.advance();
                    }
                }
                _ => {
                    content.push(c);
                    self.advance();
                }
            }
        }
        Err(format!("Unterminated action block at line {}", line))
    }

    fn read_bracket_content(&mut self) -> String {
        self.advance();
        let mut content = String::new();
        let mut depth = 1;
        while self.pos < self.input.len() && depth > 0 {
            let c = self.input[self.pos];
            match c {
                '[' => {
                    depth += 1;
                    content.push(c);
                    self.advance();
                }
                ']' => {
                    depth -= 1;
                    if depth > 0 {
                        content.push(c);
                    }
                    self.advance();
                }
                _ => {
                    content.push(c);
                    self.advance();
                }
            }
        }
        content
    }

    fn looks_like_charset(&self, content: &str) -> bool {
        if content.is_empty() {
            return false;
        }
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return false;
        }
        let has_type_syntax = trimmed.contains("int ")
            || trimmed.contains("String ")
            || trimmed.contains("bool ")
            || trimmed.contains("float ")
            || trimmed.contains("double ")
            || trimmed.contains("char ")
            || trimmed.contains("void ")
            || trimmed.contains("List<")
            || trimmed.contains("Map<")
            || trimmed.contains("[]");
        if has_type_syntax {
            return false;
        }
        for c in trimmed.chars() {
            if c.is_ascii_alphabetic()
                || c.is_ascii_digit()
                || c == '-'
                || c == '_'
                || c == '\\'
                || c == '&'
                || c.is_whitespace()
                || c == '~'
                || c == '^'
            {
                continue;
            }
            return false;
        }
        true
    }

    fn read_number(&mut self, offset: usize, line: usize, col: usize) -> G4Token {
        let mut text = String::new();
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            text.push(self.input[self.pos]);
            self.advance();
        }
        G4Token::new(G4TokenKind::IntLit, &text, line, col, offset)
    }

    fn read_identifier(&mut self, offset: usize, line: usize, col: usize) -> G4Token {
        let mut text = String::new();
        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let kind = KEYWORDS
            .iter()
            .find(|(kw, _)| *kw == text)
            .map(|(_, kind)| *kind)
            .unwrap_or(G4TokenKind::Id);
        G4Token::new(kind, &text, line, col, offset)
    }

    fn read_symbol(&mut self, offset: usize, line: usize, col: usize) -> Result<G4Token, String> {
        let c = self.input[self.pos];
        let kind = match c {
            ':' => {
                if self.peek_at(1) == Some(':') {
                    self.advance();
                    G4TokenKind::Colon
                } else {
                    G4TokenKind::Colon
                }
            }
            ';' => G4TokenKind::Semi,
            ',' => G4TokenKind::Comma,
            '.' => {
                if self.peek_at(1) == Some('.') {
                    self.advance();
                    G4TokenKind::DotDot
                } else {
                    G4TokenKind::Dot
                }
            }
            '=' => {
                if self.peek_at(1) == Some('=') {
                    self.advance();
                    G4TokenKind::Assign
                } else {
                    G4TokenKind::Assign
                }
            }
            '+' => {
                if self.peek_at(1) == Some('=') {
                    self.advance();
                    G4TokenKind::PlusAssign
                } else {
                    G4TokenKind::Plus
                }
            }
            '?' => G4TokenKind::Question,
            '*' => G4TokenKind::Star,
            '~' => G4TokenKind::Tilde,
            '|' => G4TokenKind::Pipe,
            '(' => G4TokenKind::Lparen,
            ')' => G4TokenKind::Rparen,
            '}' => G4TokenKind::Rbrace,
            '#' => G4TokenKind::Hash,
            '@' => G4TokenKind::At,
            '!' => G4TokenKind::Bang,
            '$' => G4TokenKind::Dollar,
            '^' => G4TokenKind::Caret,
            '>' => G4TokenKind::Rbrack,
            '-' => {
                if self.peek_at(1) == Some('>') {
                    self.advance();
                    G4TokenKind::Rarrow
                } else {
                    return Err(format!(
                        "Unexpected character '{}' at line {} col {}",
                        c, self.line, self.col
                    ));
                }
            }
            _ => {
                return Err(format!(
                    "Unexpected character '{}' at line {} col {}",
                    c, self.line, self.col
                ));
            }
        };
        self.advance();
        Ok(G4Token::new(kind, &c.to_string(), line, col, offset))
    }
}
