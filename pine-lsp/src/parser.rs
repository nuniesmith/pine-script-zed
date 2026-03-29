use crate::ast::*;

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse_script(src: &str) -> ParseResult {
    parse_script_impl(src)
}

// ── Token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    Na,
    // Keywords
    Var,
    Varip,
    If,
    Else,
    For,
    To,
    By,
    While,
    Switch,
    Return,
    Break,
    Continue,
    And,
    Or,
    Not,
    Import,
    Export,

    Type,
    Enum,
    Method,
    // Punctuation
    In,
    // Punctuation
    Eq,        // =
    ColonEq,   // :=
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // %
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=
    PercentEq, // %=
    EqEq,      // ==
    BangEq,    // !=
    Lt,        // <
    Le,        // <=
    Gt,        // >
    Ge,        // >=
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    LBrace,    // {
    RBrace,    // }
    Comma,     // ,
    Dot,       // .
    Colon,     // :
    Question,  // ?
    FatArrow,  // =>
    Newline,

    Eof,
}

#[derive(Debug, Clone)]
struct SpannedToken {
    token: Token,
    span: Span,
}

// ── Lexer ─────────────────────────────────────────────────────────────────────

struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    tokens: Vec<SpannedToken>,
    /// Byte offset of the start of each line (for column computation).
    line_starts: Vec<usize>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        let mut line_starts = vec![0usize];
        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            tokens: Vec::new(),
            line_starts,
        }
    }

    fn skip_line_comment(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
            self.pos += 1;
        }
    }

    fn skip_block_comment(&mut self) {
        // Skip past the opening /*
        self.pos += 2;
        while self.pos + 1 < self.bytes.len() {
            if self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/' {
                self.pos += 2;
                return;
            }
            self.pos += 1;
        }
        // Unterminated block comment — skip to end
        self.pos = self.bytes.len();
    }

    fn tokenize(mut self) -> (Vec<SpannedToken>, Vec<usize>) {
        while self.pos < self.bytes.len() {
            self.skip_whitespace_not_newline();

            if self.pos >= self.bytes.len() {
                break;
            }

            let b = self.bytes[self.pos];

            // Handle newlines
            if b == b'\n' {
                let start = self.pos;
                self.pos += 1;
                self.tokens.push(SpannedToken {
                    token: Token::Newline,
                    span: Span::new(start, self.pos),
                });
                continue;
            }

            // Handle carriage return
            if b == b'\r' {
                self.pos += 1;
                continue;
            }

            // Line comments
            if b == b'/' && self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'/' {
                self.skip_line_comment();
                continue;
            }

            // Block comments
            if b == b'/' && self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b'*' {
                self.skip_block_comment();
                continue;
            }

            let start = self.pos;

            // Identifiers and keywords
            if b.is_ascii_alphabetic() || b == b'_' {
                self.lex_ident_or_keyword(start);
                continue;
            }

            // Numbers
            if b.is_ascii_digit() {
                self.lex_number(start);
                continue;
            }

            // Strings
            if b == b'"' || b == b'\'' {
                self.lex_string(start, b);
                continue;
            }

            // Color literals: #rrggbb or #rrggbbaa
            if b == b'#'
                && self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos + 1].is_ascii_hexdigit()
            {
                self.lex_color(start);
                continue;
            }

            // Two-character tokens
            if self.pos + 1 < self.bytes.len() {
                let next = self.bytes[self.pos + 1];
                let two_char = match (b, next) {
                    (b':', b'=') => Some(Token::ColonEq),
                    (b'+', b'=') => Some(Token::PlusEq),
                    (b'-', b'=') => Some(Token::MinusEq),
                    (b'*', b'=') => Some(Token::StarEq),
                    (b'/', b'=') => Some(Token::SlashEq),
                    (b'%', b'=') => Some(Token::PercentEq),
                    (b'=', b'=') => Some(Token::EqEq),
                    (b'!', b'=') => Some(Token::BangEq),
                    (b'<', b'=') => Some(Token::Le),
                    (b'>', b'=') => Some(Token::Ge),
                    (b'=', b'>') => Some(Token::FatArrow),
                    _ => None,
                };
                if let Some(tok) = two_char {
                    self.pos += 2;
                    self.tokens.push(SpannedToken {
                        token: tok,
                        span: Span::new(start, self.pos),
                    });
                    continue;
                }
            }

            // Single-character tokens
            let tok = match b {
                b'=' => Token::Eq,
                b'+' => Token::Plus,
                b'-' => Token::Minus,
                b'*' => Token::Star,
                b'/' => Token::Slash,
                b'%' => Token::Percent,
                b'<' => Token::Lt,
                b'>' => Token::Gt,
                b'(' => Token::LParen,
                b')' => Token::RParen,
                b'[' => Token::LBracket,
                b']' => Token::RBracket,
                b'{' => Token::LBrace,
                b'}' => Token::RBrace,
                b',' => Token::Comma,
                b'.' => Token::Dot,
                b':' => Token::Colon,
                b'?' => Token::Question,
                _ => {
                    // Unknown character — skip it
                    self.pos += 1;
                    continue;
                }
            };
            self.pos += 1;
            self.tokens.push(SpannedToken {
                token: tok,
                span: Span::new(start, self.pos),
            });
        }

        self.tokens.push(SpannedToken {
            token: Token::Eof,
            span: Span::new(self.pos, self.pos),
        });

        (self.tokens, self.line_starts)
    }

    fn skip_whitespace_not_newline(&mut self) {
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b' ' || b == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn lex_ident_or_keyword(&mut self, start: usize) {
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_alphanumeric() || self.bytes[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let word = &self.src[start..self.pos];
        let token = match word {
            "var" => Token::Var,
            "varip" => Token::Varip,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "to" => Token::To,
            "by" => Token::By,
            "in" => Token::In,
            "while" => Token::While,
            "switch" => Token::Switch,
            "return" => Token::Return,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "import" => Token::Import,
            "export" => Token::Export,
            "true" => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            "na" => Token::Na,
            "type" => Token::Type,
            "enum" => Token::Enum,
            "method" => Token::Method,
            _ => Token::Ident(word.to_string()),
        };
        self.tokens.push(SpannedToken {
            token,
            span: Span::new(start, self.pos),
        });
    }

    fn lex_number(&mut self, start: usize) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        // Check for float
        if self.pos < self.bytes.len()
            && self.bytes[self.pos] == b'.'
            && self.pos + 1 < self.bytes.len()
            && self.bytes[self.pos + 1].is_ascii_digit()
        {
            self.pos += 1; // skip '.'
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            let text = &self.src[start..self.pos];
            let val = text.parse::<f64>().unwrap_or(0.0);
            self.tokens.push(SpannedToken {
                token: Token::FloatLit(val),
                span: Span::new(start, self.pos),
            });
        } else {
            let text = &self.src[start..self.pos];
            let val = text.parse::<i64>().unwrap_or(0);
            self.tokens.push(SpannedToken {
                token: Token::IntLit(val),
                span: Span::new(start, self.pos),
            });
        }
    }

    fn lex_string(&mut self, start: usize, quote: u8) {
        self.pos += 1; // skip opening quote
        let mut value = String::new();
        while self.pos < self.bytes.len() && self.bytes[self.pos] != quote {
            if self.bytes[self.pos] == b'\\' && self.pos + 1 < self.bytes.len() {
                self.pos += 1;
                match self.bytes[self.pos] {
                    b'n' => value.push('\n'),
                    b't' => value.push('\t'),
                    b'\\' => value.push('\\'),
                    b'\'' => value.push('\''),
                    b'"' => value.push('"'),
                    other => {
                        value.push('\\');
                        value.push(other as char);
                    }
                }
                self.pos += 1;
            } else if self.bytes[self.pos] == b'\n' {
                // Unterminated string at newline
                break;
            } else {
                value.push(self.bytes[self.pos] as char);
                self.pos += 1;
            }
        }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == quote {
            self.pos += 1; // skip closing quote
        }
        self.tokens.push(SpannedToken {
            token: Token::StringLit(value),
            span: Span::new(start, self.pos),
        });
    }

    fn lex_color(&mut self, start: usize) {
        self.pos += 1; // skip '#'
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_hexdigit() {
            self.pos += 1;
        }
        let text = self.src[start..self.pos].to_string();
        self.tokens.push(SpannedToken {
            token: Token::StringLit(text), // Treat color as string for simplicity
            span: Span::new(start, self.pos),
        });
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

struct Parser<'a> {
    src: &'a str,
    tokens: Vec<SpannedToken>,
    pos: usize,
    errors: Vec<ParseError>,
    /// Byte offset of the start of each line (from the lexer).
    line_starts: Vec<usize>,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        let lexer = Lexer::new(src);
        let (tokens, line_starts) = lexer.tokenize();
        Self {
            src,
            tokens,
            pos: 0,
            errors: Vec::new(),
            line_starts,
        }
    }

    /// Compute the column (0-based) of a byte offset in the original source.
    fn column_of(&self, offset: usize) -> usize {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(ins) => ins.saturating_sub(1),
        };
        let line_start = self.line_starts[line];
        offset.saturating_sub(line_start)
    }

    /// Return the column of the token that is *about to be consumed*.
    fn current_column(&self) -> usize {
        self.column_of(self.peek_span().start)
    }

    fn extract_version(&self) -> Option<u8> {
        // Look for //@version=N in the source
        self.src.lines().take(5).find_map(|l| {
            let l = l.trim();
            l.strip_prefix("//@version=")
                .and_then(|rest| rest.trim().parse::<u8>().ok())
        })
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|st| &st.token)
            .unwrap_or(&Token::Eof)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|st| st.span.clone())
            .unwrap_or_else(|| Span::new(self.src.len(), self.src.len()))
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn advance(&mut self) -> SpannedToken {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            SpannedToken {
                token: Token::Eof,
                span: Span::new(self.src.len(), self.src.len()),
            }
        }
    }

    fn expect(&mut self, expected: &Token) -> Option<SpannedToken> {
        if self.peek() == expected {
            Some(self.advance())
        } else {
            let span = self.peek_span();
            self.errors.push(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, self.peek()),
                span,
            });
            None
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    /// Check if the next non-newline token is a continuation token (and, or, ?, +, -, etc.)
    /// If so, skip the newlines. Used for multi-line expression continuation.
    fn skip_newlines_if_continuation(&mut self) {
        let save = self.pos;
        let mut temp = self.pos;
        while temp < self.tokens.len() && matches!(self.tokens[temp].token, Token::Newline) {
            temp += 1;
        }
        if temp < self.tokens.len() {
            match &self.tokens[temp].token {
                Token::And
                | Token::Or
                | Token::Question
                | Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::EqEq
                | Token::BangEq
                | Token::Lt
                | Token::Le
                | Token::Gt
                | Token::Ge
                | Token::Dot
                | Token::Colon => {
                    // It's a continuation — skip the newlines
                    self.pos = temp;
                }
                _ => {
                    self.pos = save;
                }
            }
        }
    }

    fn consume_newline_or_eof(&mut self) {
        match self.peek() {
            Token::Newline => {
                self.advance();
            }
            Token::Eof => {}
            _ => {
                // Just skip to the next newline for error recovery
            }
        }
    }

    // ── Statement parsing ─────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Option<Spanned<Stmt>> {
        self.skip_newlines();

        if self.at_eof() {
            return None;
        }

        let _start_span = self.peek_span();

        match self.peek().clone() {
            Token::Var | Token::Varip => self.parse_var_decl(),
            Token::If => self.parse_if(),
            Token::For => self.parse_for(),
            Token::While => self.parse_while(),
            Token::Switch => self.parse_switch(),
            Token::Return => self.parse_return(),
            Token::Break => {
                let sp = self.advance();
                self.consume_newline_or_eof();
                Some(Spanned::new(Stmt::Break, sp.span))
            }
            Token::Continue => {
                let sp = self.advance();
                self.consume_newline_or_eof();
                Some(Spanned::new(Stmt::Continue, sp.span))
            }
            Token::Import => self.parse_import(),
            Token::Export => self.parse_export(),
            Token::Type => self.parse_type_def(),
            Token::Enum => self.parse_enum_def(),
            Token::Method => self.parse_method_def(),
            Token::Ident(_) => {
                // Could be: var decl, func def, reassignment, or expression
                self.parse_ident_stmt()
            }
            _ => {
                // Try parsing as an expression statement
                self.parse_expr_stmt()
            }
        }
    }

    fn parse_ident_stmt(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();

        // Save position for backtracking
        let saved_pos = self.pos;

        // Try to detect: name(params) => body  (function definition)
        if let Token::Ident(_) = self.peek() {
            let save = self.pos;
            let name_tok = self.advance();
            let name = if let Token::Ident(n) = &name_tok.token {
                n.clone()
            } else {
                self.pos = save;
                return self.parse_expr_stmt();
            };

            // Check for function def: name(params) =>
            if matches!(self.peek(), Token::LParen) {
                let paren_save = self.pos;
                if let Some(func_def) = self.try_parse_func_def(name.clone(), name_tok.span.clone())
                {
                    return Some(func_def);
                }
                self.pos = paren_save;
            }

            // Check for var decl: name = expr  or  type name = expr
            // But first check if it looks like: ident ident = expr (type annotation)
            // Also handle: type<generic> name = expr (e.g. array<float> x = ...)
            if let Token::Ident(second_name) = self.peek().clone() {
                let _second_tok = self.advance();
                if matches!(self.peek(), Token::Eq) {
                    self.advance(); // consume '='
                    let value = self.parse_expr()?;
                    let end = value.span.clone();
                    self.consume_newline_or_eof();
                    return Some(Spanned::new(
                        Stmt::VarDecl(VarDecl {
                            kind: VarKind::Decl,
                            name: second_name,
                            type_ann: Some(self.resolve_type_name(&name)),
                            value,
                        }),
                        start.merge(&end),
                    ));
                }
                // Not a typed decl, backtrack
                self.pos = saved_pos;
            } else if matches!(self.peek(), Token::Lt) {
                // Could be generic type: array<float> name = ...
                let generic_save = self.pos;
                self.advance(); // skip '<'
                let _inner_type = self.parse_type_ann();
                if matches!(self.peek(), Token::Gt) {
                    self.advance(); // skip '>'
                                    // Now check for: (args) to construct, or name = ... for decl
                    if let Token::Ident(var_name) = self.peek().clone() {
                        self.advance();
                        if matches!(self.peek(), Token::Eq) {
                            self.advance();
                            let value = self.parse_expr()?;
                            let end = value.span.clone();
                            self.consume_newline_or_eof();
                            return Some(Spanned::new(
                                Stmt::VarDecl(VarDecl {
                                    kind: VarKind::Decl,
                                    name: var_name,
                                    type_ann: Some(Type::Named(name)),
                                    value,
                                }),
                                start.merge(&end),
                            ));
                        }
                    }
                }
                self.pos = generic_save;
                // Fall through — backtrack to saved_pos below
                self.pos = saved_pos;
            } else {
                self.pos = saved_pos;
            }

            // Check for simple var decl: name = expr
            {
                let _save2 = self.pos;
                let name_tok2 = self.advance();
                let name2 = if let Token::Ident(n) = &name_tok2.token {
                    n.clone()
                } else {
                    self.pos = saved_pos;
                    return self.parse_expr_stmt();
                };

                if matches!(self.peek(), Token::Eq) {
                    // The lexer already handles == as EqEq, so a bare Eq is always `=`
                    self.advance(); // consume '='
                    let value = self.parse_expr()?;
                    let end = value.span.clone();
                    self.consume_newline_or_eof();
                    return Some(Spanned::new(
                        Stmt::VarDecl(VarDecl {
                            kind: VarKind::Decl,
                            name: name2,
                            type_ann: None,
                            value,
                        }),
                        start.merge(&end),
                    ));
                } else if matches!(self.peek(), Token::ColonEq) {
                    self.advance(); // consume ':='
                    let value = self.parse_expr()?;
                    let end = value.span.clone();
                    self.consume_newline_or_eof();
                    let target = Spanned::new(Expr::Ident(name2), name_tok2.span.clone());
                    return Some(Spanned::new(
                        Stmt::Reassign { target, value },
                        start.merge(&end),
                    ));
                } else if matches!(
                    self.peek(),
                    Token::PlusEq
                        | Token::MinusEq
                        | Token::StarEq
                        | Token::SlashEq
                        | Token::PercentEq
                ) {
                    let op_tok = self.advance();
                    let op = match op_tok.token {
                        Token::PlusEq => BinOp::Add,
                        Token::MinusEq => BinOp::Sub,
                        Token::StarEq => BinOp::Mul,
                        Token::SlashEq => BinOp::Div,
                        Token::PercentEq => BinOp::Mod,
                        _ => unreachable!(),
                    };
                    let rhs = self.parse_expr()?;
                    let end = rhs.span.clone();
                    self.consume_newline_or_eof();
                    let target = Spanned::new(Expr::Ident(name2.clone()), name_tok2.span.clone());
                    let ident_expr = Spanned::new(Expr::Ident(name2), name_tok2.span.clone());
                    let value = Spanned::new(
                        Expr::BinOp {
                            op,
                            lhs: Box::new(ident_expr),
                            rhs: Box::new(rhs),
                        },
                        start.merge(&end),
                    );
                    return Some(Spanned::new(
                        Stmt::Reassign { target, value },
                        start.merge(&end),
                    ));
                }

                // Backtrack and parse as expression
                self.pos = saved_pos;
            }
        }

        self.parse_expr_stmt()
    }

    fn try_parse_func_def(&mut self, name: String, name_span: Span) -> Option<Spanned<Stmt>> {
        let save = self.pos;

        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            if let Token::Ident(pname) = self.peek().clone() {
                self.advance();
                let type_ann = if matches!(self.peek(), Token::Colon) {
                    self.advance();
                    self.parse_type_ann()
                } else {
                    None
                };
                let default = if matches!(self.peek(), Token::Eq) {
                    self.advance();
                    self.parse_expr()
                } else {
                    None
                };
                params.push(Param {
                    name: pname,
                    type_ann,
                    default,
                });
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                }
            } else {
                self.pos = save;
                return None;
            }
        }
        if self.expect(&Token::RParen).is_none() {
            self.pos = save;
            return None;
        }

        if !matches!(self.peek(), Token::FatArrow) {
            self.pos = save;
            return None;
        }
        self.advance(); // consume '=>'

        let body = self.parse_block();

        let end_span = if let Some(last) = body.last() {
            last.span.clone()
        } else {
            self.peek_span()
        };

        Some(Spanned::new(
            Stmt::FuncDef(FuncDef {
                name,
                params,
                ret_type: None,
                body,
            }),
            name_span.merge(&end_span),
        ))
    }

    fn parse_var_decl(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        let kind = match self.peek() {
            Token::Var => {
                self.advance();
                VarKind::Var
            }
            Token::Varip => {
                self.advance();
                VarKind::Varip
            }
            _ => VarKind::Decl,
        };

        // Optional type annotation (handles generic types like array<float>)
        let type_ann = self.try_parse_type_before_name();

        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected variable name".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        if matches!(self.peek(), Token::Eq) {
            self.advance();
        } else {
            self.errors.push(ParseError {
                message: "Expected '=' in variable declaration".into(),
                span: self.peek_span(),
            });
            return None;
        }

        let value = self.parse_expr()?;
        let end = value.span.clone();
        self.consume_newline_or_eof();

        Some(Spanned::new(
            Stmt::VarDecl(VarDecl {
                kind,
                name,
                type_ann,
                value,
            }),
            start.merge(&end),
        ))
    }

    fn try_parse_type_before_name(&mut self) -> Option<Type> {
        if let Token::Ident(type_name) = self.peek().clone() {
            let save = self.pos;
            self.advance();

            // Handle generic types: array<float>, matrix<int>, map<string, float>
            if matches!(self.peek(), Token::Lt) {
                let gen_save = self.pos;
                self.advance(); // skip '<'
                                // Try to parse generic args — skip tokens until we find '>'
                let mut depth = 1;
                while depth > 0 && !self.at_eof() {
                    match self.peek() {
                        Token::Lt => {
                            depth += 1;
                            self.advance();
                        }
                        Token::Gt => {
                            depth -= 1;
                            if depth > 0 {
                                self.advance();
                            }
                        }
                        _ => {
                            self.advance();
                        }
                    }
                }
                if depth == 0 {
                    self.advance(); // skip final '>'
                    if let Token::Ident(_) = self.peek() {
                        // type<...> name — it's a generic type annotation
                        return Some(Type::Named(type_name));
                    }
                    // Could be type<...>() constructor — not a type annotation
                }
                self.pos = gen_save;
            }

            // Simple type: ident ident (first is type, second is name)
            if let Token::Ident(_) = self.peek() {
                return Some(self.resolve_type_name(&type_name));
            }
            self.pos = save;
        }
        None
    }

    fn resolve_type_name(&self, name: &str) -> Type {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "bool" => Type::Bool,
            "string" => Type::String,
            "color" => Type::Color,
            "label" => Type::Label,
            "line" => Type::Line,
            "box" => Type::Box,
            "table" => Type::Table,
            "linefill" => Type::Linefill,
            "polyline" => Type::Polyline,
            _ => Type::Named(name.to_string()),
        }
    }

    fn parse_type_ann(&mut self) -> Option<Type> {
        match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                Some(self.resolve_type_name(&name))
            }
            _ => None,
        }
    }

    fn parse_if(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'if'

        let cond = self.parse_expr()?;
        let then_body = self.parse_block();

        let mut else_ifs = Vec::new();
        let mut else_body = None;

        while matches!(self.peek(), Token::Else) {
            self.advance(); // consume 'else'
            if matches!(self.peek(), Token::If) {
                self.advance(); // consume 'if'
                let ei_cond = self.parse_expr()?;
                let ei_body = self.parse_block();
                else_ifs.push((ei_cond, ei_body));
            } else {
                else_body = Some(self.parse_block());
                break;
            }
        }

        let end = self.peek_span();
        Some(Spanned::new(
            Stmt::If {
                cond,
                then_body,
                else_ifs,
                else_body,
            },
            start.merge(&end),
        ))
    }

    fn parse_for(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'for'

        // Check for: for [key, val] in iterable  (tuple destructuring)
        if matches!(self.peek(), Token::LBracket) {
            self.advance(); // consume '['
            let first = match self.peek().clone() {
                Token::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    self.errors.push(ParseError {
                        message: "Expected variable name in for-in destructuring".into(),
                        span: self.peek_span(),
                    });
                    return None;
                }
            };
            if matches!(self.peek(), Token::Comma) {
                self.advance(); // consume ','
            }
            let second = match self.peek().clone() {
                Token::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    self.errors.push(ParseError {
                        message: "Expected second variable name in for-in destructuring".into(),
                        span: self.peek_span(),
                    });
                    return None;
                }
            };
            self.expect(&Token::RBracket);
            self.expect(&Token::In);
            let iterable = self.parse_expr()?;
            let body = self.parse_block();
            let end = body
                .last()
                .map(|s| s.span.clone())
                .unwrap_or_else(|| iterable.span.clone());
            self.consume_newline_or_eof();
            return Some(Spanned::new(
                Stmt::ForIn {
                    key_var: Some(first),
                    val_var: second,
                    iterable,
                    body,
                },
                start.merge(&end),
            ));
        }

        let var_name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected variable name after 'for'".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        // Check for: for val in iterable
        if matches!(self.peek(), Token::In) {
            self.advance(); // consume 'in'
            let iterable = self.parse_expr()?;
            let body = self.parse_block();
            let end = body
                .last()
                .map(|s| s.span.clone())
                .unwrap_or_else(|| iterable.span.clone());
            self.consume_newline_or_eof();
            return Some(Spanned::new(
                Stmt::ForIn {
                    key_var: None,
                    val_var: var_name,
                    iterable,
                    body,
                },
                start.merge(&end),
            ));
        }

        // Check for `for x in iterable` vs `for x = from to to`
        if matches!(self.peek(), Token::Ident(s) if s == "in") {
            self.advance(); // consume 'in'
            let iterable = self.parse_expr()?;
            let body = self.parse_block();
            let end = self.peek_span();
            return Some(Spanned::new(
                Stmt::ForIn {
                    key_var: None,
                    val_var: var_name,
                    iterable,
                    body,
                },
                start.merge(&end),
            ));
        }

        // Check for [key, val] in
        if matches!(self.peek(), Token::Comma) {
            self.advance(); // consume ','
            let val_name = match self.peek().clone() {
                Token::Ident(n) => {
                    self.advance();
                    n
                }
                _ => {
                    self.errors.push(ParseError {
                        message: "Expected second variable name".into(),
                        span: self.peek_span(),
                    });
                    return None;
                }
            };
            if matches!(self.peek(), Token::Ident(s) if s == "in") {
                self.advance(); // consume 'in'
                let iterable = self.parse_expr()?;
                let body = self.parse_block();
                let end = self.peek_span();
                return Some(Spanned::new(
                    Stmt::ForIn {
                        key_var: Some(var_name),
                        val_var: val_name,
                        iterable,
                        body,
                    },
                    start.merge(&end),
                ));
            }
        }

        // for x = from to to [by step]
        self.expect(&Token::Eq);
        let from = self.parse_expr()?;
        self.expect(&Token::To);
        let to = self.parse_expr()?;
        let step = if matches!(self.peek(), Token::By) {
            self.advance();
            self.parse_expr()
        } else {
            None
        };
        let body = self.parse_block();
        let end = self.peek_span();

        Some(Spanned::new(
            Stmt::For {
                var: var_name,
                from,
                to,
                step,
                body,
            },
            start.merge(&end),
        ))
    }

    fn parse_while(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'while'

        let cond = self.parse_expr()?;
        let body = self.parse_block();
        let end = self.peek_span();

        Some(Spanned::new(Stmt::While { cond, body }, start.merge(&end)))
    }

    fn parse_switch(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'switch'

        // Optional expression
        let expr = if !matches!(self.peek(), Token::Newline | Token::Eof) {
            self.parse_expr()
        } else {
            None
        };

        self.skip_newlines();

        let mut arms = Vec::new();
        // Parse switch arms (simplified — just parse until we hit a dedent or known delimiter)
        while !self.at_eof()
            && !matches!(
                self.peek(),
                Token::Eof | Token::Ident(_) if false
            )
        {
            self.skip_newlines();
            if self.at_eof() {
                break;
            }
            // Try to parse a case or break
            if matches!(self.peek(), Token::FatArrow) {
                // default => body
                self.advance();
                let body = self.parse_block();
                arms.push(SwitchArm::Default(body));
                break;
            } else if let Some(case_expr) = self.parse_expr() {
                if matches!(self.peek(), Token::FatArrow) {
                    self.advance();
                    let body = self.parse_block();
                    arms.push(SwitchArm::Case(case_expr, body));
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let end = self.peek_span();
        Some(Spanned::new(Stmt::Switch { expr, arms }, start.merge(&end)))
    }

    fn parse_return(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'return'

        let value = if matches!(self.peek(), Token::Newline | Token::Eof) {
            None
        } else {
            self.parse_expr()
        };

        let end = value
            .as_ref()
            .map(|v| v.span.clone())
            .unwrap_or_else(|| start.clone());
        self.consume_newline_or_eof();

        Some(Spanned::new(Stmt::Return(value), start.merge(&end)))
    }

    fn parse_import(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'import'

        // Expect a string or dotted path
        let path = match self.peek().clone() {
            Token::StringLit(s) => {
                self.advance();
                s
            }
            Token::Ident(s) => {
                self.advance();
                let mut path = s;
                while matches!(self.peek(), Token::Slash | Token::Dot) {
                    let sep = self.advance();
                    path.push(if sep.token == Token::Slash { '/' } else { '.' });
                    if let Token::Ident(next) = self.peek().clone() {
                        self.advance();
                        path.push_str(&next);
                    } else if let Token::IntLit(n) = self.peek().clone() {
                        self.advance();
                        path.push_str(&n.to_string());
                    }
                }
                path
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected import path".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        // Optional 'as' alias
        let alias = if matches!(self.peek(), Token::Ident(s) if s == "as") {
            self.advance();
            match self.peek().clone() {
                Token::Ident(a) => {
                    self.advance();
                    Some(a)
                }
                _ => None,
            }
        } else {
            None
        };

        let end = self.peek_span();
        self.consume_newline_or_eof();

        Some(Spanned::new(
            Stmt::Import(ImportDef { path, alias }),
            start.merge(&end),
        ))
    }

    fn parse_export(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'export'

        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected name after 'export'".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        let end = self.peek_span();
        self.consume_newline_or_eof();

        Some(Spanned::new(Stmt::Export(name), start.merge(&end)))
    }

    fn parse_type_def(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'type'

        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected type name".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        self.skip_newlines();

        let mut fields = Vec::new();
        // Parse fields (simplified: expect indented lines with `name: type`)
        while let Token::Ident(_) = self.peek() {
            if let Some(field) = self.parse_type_field() {
                fields.push(field);
            } else {
                break;
            }
            self.skip_newlines();
        }

        let end = self.peek_span();

        Some(Spanned::new(
            Stmt::TypeDef(TypeDef {
                export: false,
                name,
                fields,
            }),
            start.merge(&end),
        ))
    }

    fn parse_type_field(&mut self) -> Option<TypeField> {
        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => return None,
        };

        self.expect(&Token::Colon)?;
        let type_ann = self
            .parse_type_ann()
            .unwrap_or(Type::Named("unknown".into()));

        let default = if matches!(self.peek(), Token::Eq) {
            self.advance();
            self.parse_expr()
        } else {
            None
        };

        self.consume_newline_or_eof();

        Some(TypeField {
            name,
            type_ann,
            default,
        })
    }

    fn parse_enum_def(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'enum'

        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected enum name".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        self.skip_newlines();

        let mut variants = Vec::new();
        while let Token::Ident(_) = self.peek() {
            let vname = match self.peek().clone() {
                Token::Ident(n) => {
                    self.advance();
                    n
                }
                _ => break,
            };
            let value = if matches!(self.peek(), Token::Eq) {
                self.advance();
                self.parse_expr()
            } else {
                None
            };
            variants.push(EnumVariant { name: vname, value });
            self.skip_newlines();
        }

        let end = self.peek_span();

        Some(Spanned::new(
            Stmt::EnumDef(EnumDef {
                export: false,
                name,
                variants,
            }),
            start.merge(&end),
        ))
    }

    fn parse_method_def(&mut self) -> Option<Spanned<Stmt>> {
        let start = self.peek_span();
        self.advance(); // consume 'method'

        let name = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected method name".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        self.expect(&Token::LParen)?;

        // First param is the receiver type
        let receiver_type = match self.peek().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => {
                self.errors.push(ParseError {
                    message: "Expected receiver type".into(),
                    span: self.peek_span(),
                });
                return None;
            }
        };

        // Skip receiver name if present
        if let Token::Ident(_) = self.peek() {
            self.advance();
        }

        let mut params = Vec::new();
        while matches!(self.peek(), Token::Comma) {
            self.advance(); // consume ','
            if let Token::Ident(pname) = self.peek().clone() {
                self.advance();
                let type_ann = if matches!(self.peek(), Token::Colon) {
                    self.advance();
                    self.parse_type_ann()
                } else {
                    None
                };
                params.push(Param {
                    name: pname,
                    type_ann,
                    default: None,
                });
            }
        }

        self.expect(&Token::RParen)?;
        self.expect(&Token::FatArrow)?;

        let body = self.parse_block();

        let end = self.peek_span();

        Some(Spanned::new(
            Stmt::MethodDef(MethodDef {
                receiver_type,
                name,
                params,
                ret_type: None,
                body,
            }),
            start.merge(&end),
        ))
    }

    fn parse_expr_stmt(&mut self) -> Option<Spanned<Stmt>> {
        let expr = self.parse_expr()?;

        // Check for reassignment: expr := value
        if matches!(self.peek(), Token::ColonEq) {
            self.advance();
            let value = self.parse_expr()?;
            let span = expr.span.merge(&value.span);
            self.consume_newline_or_eof();
            return Some(Spanned::new(
                Stmt::Reassign {
                    target: expr,
                    value,
                },
                span,
            ));
        }

        // Check for compound assignment: expr += value, expr -= value, etc.
        if matches!(
            self.peek(),
            Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq | Token::PercentEq
        ) {
            let op_tok = self.advance();
            let op = match op_tok.token {
                Token::PlusEq => BinOp::Add,
                Token::MinusEq => BinOp::Sub,
                Token::StarEq => BinOp::Mul,
                Token::SlashEq => BinOp::Div,
                Token::PercentEq => BinOp::Mod,
                _ => unreachable!(),
            };
            let rhs = self.parse_expr()?;
            let span = expr.span.merge(&rhs.span);
            let value = Spanned::new(
                Expr::BinOp {
                    op,
                    lhs: Box::new(expr.clone()),
                    rhs: Box::new(rhs),
                },
                span.clone(),
            );
            self.consume_newline_or_eof();
            return Some(Spanned::new(
                Stmt::Reassign {
                    target: expr,
                    value,
                },
                span,
            ));
        }

        let span = expr.span.clone();
        self.consume_newline_or_eof();
        Some(Spanned::new(Stmt::Expr(expr), span))
    }

    fn parse_block(&mut self) -> Vec<Spanned<Stmt>> {
        self.skip_newlines();
        // The block must be indented further than the parent context.
        // Determine the indent of the first token in the block.
        let block_col = self.current_column();
        self.parse_block_at_indent(block_col)
    }

    /// Parse a sequence of statements that all start at `block_col` or deeper.
    /// Stops when a line starts at a shallower indent (or EOF).
    fn parse_block_at_indent(&mut self, block_col: usize) -> Vec<Spanned<Stmt>> {
        let mut stmts = Vec::new();

        loop {
            self.skip_newlines();

            if self.at_eof() {
                break;
            }

            let col = self.current_column();

            // If the next token is at a shallower column, the block is over.
            if col < block_col {
                break;
            }

            match self.parse_stmt() {
                Some(stmt) => stmts.push(stmt),
                None => {
                    // Error recovery: skip the token and keep trying
                    if !self.at_eof() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        stmts
    }

    // ── Expression parsing (Pratt / precedence climbing) ──────────────────

    fn parse_expr(&mut self) -> Option<Spanned<Expr>> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Option<Spanned<Expr>> {
        let cond = self.parse_or()?;

        // Allow newline before '?' for multi-line ternary
        self.skip_newlines_if_continuation();

        if matches!(self.peek(), Token::Question) {
            self.advance(); // consume '?'
            self.skip_newlines(); // allow newline after '?'
            let then_expr = self.parse_or()?;
            self.skip_newlines(); // allow newline before ':'
            if self.expect(&Token::Colon).is_none() {
                return Some(cond);
            }
            self.skip_newlines(); // allow newline after ':'
            let else_expr = self.parse_ternary()?; // right-recursive for nested ternaries
            let span = cond.span.merge(&else_expr.span);
            Some(Spanned::new(
                Expr::Ternary {
                    cond: Box::new(cond),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                },
                span,
            ))
        } else {
            Some(cond)
        }
    }

    fn parse_or(&mut self) -> Option<Spanned<Expr>> {
        let mut lhs = self.parse_and()?;
        loop {
            self.skip_newlines_if_continuation();
            if !matches!(self.peek(), Token::Or) {
                break;
            }
            self.advance();
            self.skip_newlines(); // allow newline after 'or'
            let rhs = self.parse_and()?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned::new(
                Expr::BinOp {
                    op: BinOp::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        Some(lhs)
    }

    fn parse_and(&mut self) -> Option<Spanned<Expr>> {
        let mut lhs = self.parse_comparison()?;
        loop {
            self.skip_newlines_if_continuation();
            if !matches!(self.peek(), Token::And) {
                break;
            }
            self.advance();
            self.skip_newlines(); // allow newline after 'and'
            let rhs = self.parse_comparison()?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned::new(
                Expr::BinOp {
                    op: BinOp::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        Some(lhs)
    }

    fn parse_comparison(&mut self) -> Option<Spanned<Expr>> {
        let mut lhs = self.parse_additive()?;
        loop {
            self.skip_newlines_if_continuation();
            let op = match self.peek() {
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::Ne,
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            self.skip_newlines(); // allow newline after comparison operator
            let rhs = self.parse_additive()?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned::new(
                Expr::BinOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        Some(lhs)
    }

    fn parse_additive(&mut self) -> Option<Spanned<Expr>> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            self.skip_newlines_if_continuation();
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            self.skip_newlines(); // allow newline after +/-
            let rhs = self.parse_multiplicative()?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned::new(
                Expr::BinOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        Some(lhs)
    }

    fn parse_multiplicative(&mut self) -> Option<Spanned<Expr>> {
        let mut lhs = self.parse_unary()?;
        loop {
            self.skip_newlines_if_continuation();
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            self.skip_newlines(); // allow newline after * / %
            let rhs = self.parse_unary()?;
            let span = lhs.span.merge(&rhs.span);
            lhs = Spanned::new(
                Expr::BinOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }
        Some(lhs)
    }

    fn parse_unary(&mut self) -> Option<Spanned<Expr>> {
        let start = self.peek_span();
        match self.peek() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(&operand.span);
                Some(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::Neg,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(&operand.span);
                Some(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::Not,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Option<Spanned<Expr>> {
        let mut expr = self.parse_atom()?;

        loop {
            match self.peek() {
                Token::Dot => {
                    self.advance();
                    // Allow keywords as field names (e.g. syminfo.type, line.style_solid)
                    let field_name = match self.peek().clone() {
                        Token::Ident(f) => Some(f),
                        Token::Type => Some("type".to_string()),
                        Token::And => Some("and".to_string()),
                        Token::Or => Some("or".to_string()),
                        Token::Not => Some("not".to_string()),
                        Token::If => Some("if".to_string()),
                        Token::Else => Some("else".to_string()),
                        Token::For => Some("for".to_string()),
                        Token::While => Some("while".to_string()),
                        Token::Switch => Some("switch".to_string()),
                        Token::Return => Some("return".to_string()),
                        Token::Break => Some("break".to_string()),
                        Token::Continue => Some("continue".to_string()),
                        Token::Var => Some("var".to_string()),
                        Token::Varip => Some("varip".to_string()),
                        Token::Import => Some("import".to_string()),
                        Token::Export => Some("export".to_string()),
                        Token::Method => Some("method".to_string()),
                        Token::Enum => Some("enum".to_string()),
                        Token::In => Some("in".to_string()),
                        Token::To => Some("to".to_string()),
                        Token::By => Some("by".to_string()),
                        Token::Na => Some("na".to_string()),
                        _ => None,
                    };
                    match field_name {
                        Some(field) => {
                            let ftok = self.advance();
                            let span = expr.span.merge(&ftok.span);
                            expr = Spanned::new(
                                Expr::Field {
                                    object: Box::new(expr),
                                    field,
                                },
                                span,
                            );
                        }
                        None => break,
                    }
                }
                Token::LBracket => {
                    self.advance();
                    self.skip_newlines(); // allow newline after '['
                    let index = self.parse_expr()?;
                    self.skip_newlines(); // allow newline before ']'
                    let end = self.peek_span();
                    self.expect(&Token::RBracket);
                    let span = expr.span.merge(&end);
                    expr = Spanned::new(
                        Expr::Index {
                            object: Box::new(expr),
                            index: Box::new(index),
                        },
                        span,
                    );
                }
                Token::Lt => {
                    // Try to parse generic call: expr<type>(args)
                    // e.g. array.new<float>(200), array.new<label>()
                    let save = self.pos;
                    self.advance(); // skip '<'
                                    // Try to read type names separated by commas until '>'
                    let mut valid_generic = true;
                    let mut depth = 1;
                    while depth > 0 && !self.at_eof() {
                        match self.peek() {
                            Token::Gt => {
                                depth -= 1;
                                if depth > 0 {
                                    self.advance();
                                }
                            }
                            Token::Lt => {
                                depth += 1;
                                self.advance();
                            }
                            Token::Ident(_) | Token::Comma => {
                                self.advance();
                            }
                            _ => {
                                valid_generic = false;
                                break;
                            }
                        }
                    }
                    if valid_generic && depth == 0 {
                        self.advance(); // skip '>'
                        if matches!(self.peek(), Token::LParen) {
                            // It's a generic call — now parse the call args
                            self.advance(); // skip '('
                            self.skip_newlines();
                            let (args, named) = self.parse_call_args();
                            self.skip_newlines();
                            let end = self.peek_span();
                            self.expect(&Token::RParen);
                            let span = expr.span.merge(&end);
                            expr = Spanned::new(
                                Expr::Call {
                                    func: Box::new(expr),
                                    args,
                                    named,
                                },
                                span,
                            );
                            continue;
                        }
                    }
                    // Not a generic call — backtrack and stop postfix chain
                    self.pos = save;
                    break;
                }
                Token::LParen => {
                    self.advance();
                    self.skip_newlines(); // allow newline after '('
                    let (args, named) = self.parse_call_args();
                    self.skip_newlines(); // allow newline before ')'
                    let end = self.peek_span();
                    self.expect(&Token::RParen);
                    let span = expr.span.merge(&end);
                    expr = Spanned::new(
                        Expr::Call {
                            func: Box::new(expr),
                            args,
                            named,
                        },
                        span,
                    );
                }
                _ => break,
            }
        }

        Some(expr)
    }

    fn parse_call_args(&mut self) -> (Vec<Spanned<Expr>>, Vec<(String, Spanned<Expr>)>) {
        let mut positional = Vec::new();
        let mut named = Vec::new();

        self.skip_newlines();

        if matches!(self.peek(), Token::RParen) {
            return (positional, named);
        }

        loop {
            self.skip_newlines();

            if matches!(self.peek(), Token::RParen | Token::Eof) {
                break;
            }

            // Try to parse named arg: ident = expr (but not ident == expr)
            if let Token::Ident(name) = self.peek().clone() {
                let save = self.pos;
                self.advance();
                if matches!(self.peek(), Token::Eq) {
                    self.advance();
                    if let Some(val) = self.parse_expr() {
                        named.push((name, val));
                    }
                } else {
                    // Not a named arg, backtrack and parse as positional
                    self.pos = save;
                    if let Some(val) = self.parse_expr() {
                        positional.push(val);
                    }
                }
            } else if let Some(val) = self.parse_expr() {
                positional.push(val);
            } else {
                break;
            }

            self.skip_newlines();

            if matches!(self.peek(), Token::Comma) {
                self.advance();
                self.skip_newlines();
                // Allow trailing comma before RParen
                if matches!(self.peek(), Token::RParen) {
                    break;
                }
            } else {
                break;
            }
        }

        (positional, named)
    }

    fn parse_atom(&mut self) -> Option<Spanned<Expr>> {
        let tok = self.peek().clone();
        let span = self.peek_span();

        match tok {
            Token::IntLit(v) => {
                self.advance();
                Some(Spanned::new(Expr::IntLit(v), span))
            }
            Token::FloatLit(v) => {
                self.advance();
                Some(Spanned::new(Expr::FloatLit(v), span))
            }
            Token::BoolLit(v) => {
                self.advance();
                Some(Spanned::new(Expr::BoolLit(v), span))
            }
            Token::Na => {
                self.advance();
                Some(Spanned::new(Expr::Na, span))
            }
            Token::StringLit(s) => {
                self.advance();
                // Handle string concatenation with '+' across newlines
                let mut result = s;
                while matches!(self.peek(), Token::Plus) {
                    let save = self.pos;
                    self.advance(); // consume '+'
                    self.skip_newlines();
                    if let Token::StringLit(s2) = self.peek().clone() {
                        self.advance();
                        result.push_str(&s2);
                    } else {
                        self.pos = save;
                        break;
                    }
                }
                if result.starts_with('#') {
                    Some(Spanned::new(Expr::ColorLit(result), span))
                } else {
                    Some(Spanned::new(Expr::StringLit(result), span))
                }
            }
            Token::Ident(name) => {
                self.advance();
                // Check if this is a type cast: float(expr), int(expr), etc.
                // Only for known primitive type names immediately followed by '('
                if matches!(name.as_str(), "int" | "float" | "bool" | "string" | "color")
                    && matches!(self.peek(), Token::LParen)
                {
                    // Treat as a regular function call — PineScript uses this for casts
                    // The postfix handler will pick up the LParen
                }
                Some(Spanned::new(Expr::Ident(name), span))
            }
            Token::LParen => {
                self.advance();
                self.skip_newlines(); // allow newline after '('
                let inner = self.parse_expr()?;
                self.skip_newlines(); // allow newline before ')'
                self.expect(&Token::RParen);
                Some(inner)
            }
            Token::LBracket => {
                // Tuple: [a, b, c]
                self.advance();
                self.skip_newlines();
                let mut elems = Vec::new();
                while !matches!(self.peek(), Token::RBracket | Token::Eof) {
                    self.skip_newlines();
                    if let Some(e) = self.parse_expr() {
                        elems.push(e);
                    }
                    self.skip_newlines();
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.skip_newlines();
                let end = self.peek_span();
                self.expect(&Token::RBracket);
                Some(Spanned::new(Expr::Tuple(elems), span.merge(&end)))
            }
            _ => {
                // Don't emit an error for newlines/eof — those are often just end-of-statement
                if !matches!(
                    tok,
                    Token::Newline
                        | Token::Eof
                        | Token::Else
                        | Token::RBrace
                        | Token::RParen
                        | Token::RBracket
                ) {
                    self.errors.push(ParseError {
                        message: format!("Unexpected token: {:?}", tok),
                        span: span.clone(),
                    });
                }
                None
            }
        }
    }
}

// ── Revised top-level entry point ─────────────────────────────────────────────

// We override the public entry point to properly collect results.

pub fn parse_script_impl(src: &str) -> ParseResult {
    let mut parser = Parser::new(src);
    let version = parser.extract_version();
    let mut stmts: Vec<Spanned<Stmt>> = Vec::new();

    parser.skip_newlines();
    while !parser.at_eof() {
        match parser.parse_stmt() {
            Some(stmt) => stmts.push(stmt),
            None => {
                // Error recovery: skip token
                if !parser.at_eof() {
                    parser.advance();
                }
            }
        }
        parser.skip_newlines();
    }

    // Detect script kind from top-level calls
    let kind = stmts.iter().find_map(|s| {
        if let Stmt::Expr(ref e) = s.node {
            if let Expr::Call { ref func, .. } = e.node {
                if let Expr::Ident(ref name) = func.node {
                    match name.as_str() {
                        "indicator" => return Some((ScriptKind::Indicator, e.clone())),
                        "strategy" => return Some((ScriptKind::Strategy, e.clone())),
                        "library" => return Some((ScriptKind::Library, e.clone())),
                        _ => {}
                    }
                }
            }
        }
        None
    });

    let script = Script {
        version,
        kind,
        stmts,
    };

    ParseResult {
        script: Some(script),
        errors: parser.errors,
    }
}
