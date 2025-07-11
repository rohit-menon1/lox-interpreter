use std::char;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  {} tokenize <file.lox>", args[0]);
        eprintln!("  {} evaluate <file.lox>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file: {}", filename);
        String::new()
    });

    match command.as_str() {
        "tokenize" => run_tokenizer(&file_contents),
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.token_type {
            TokenType::Error(ch, _) => {
                if ch.eq_ignore_ascii_case(&'"') {
                    write!(f, "[line {}] Error: Unterminated String", self.line)
                } else {
                    write!(
                        f,
                        "[line {}] Error: Unexpected character: {}",
                        self.line, ch
                    )
                }
            }
            TokenType::LeftParen => write!(f, "LEFT_PAREN ( null"),
            TokenType::RightParen => write!(f, "RIGHT_PAREN ) null"),
            TokenType::LeftBrace => write!(f, "LEFT_BRACE {{ null"),
            TokenType::RightBrace => write!(f, "RIGHT_BRACE }} null"),
            TokenType::Comma => write!(f, "COMMA , null"),
            TokenType::Dot => write!(f, "DOT . null"),
            TokenType::Minus => write!(f, "MINUS - null"),
            TokenType::Plus => write!(f, "PLUS + null"),
            TokenType::Semicolon => write!(f, "SEMICOLON ; null"),
            TokenType::Star => write!(f, "STAR * null"),
            TokenType::String(s) => write!(f, "STRING {} \" null", s),
            TokenType::Eof => write!(f, "EOF  null"),
            TokenType::DoubleEquals => write!(f, "EQUAL_EQUAL == null"),
            TokenType::Equals => write!(f, "EQUAL = null"),
            TokenType::Greater => write!(f, "GREATER > null"),
            TokenType::GreaterEquals => write!(f, "GREATER_EQUALS >= null"),
            TokenType::LessThanEquals => write!(f, "LESSTHAN_EQUALS <= null"),
            TokenType::LessThan => write!(f, "LESSTHAN < null"),
            TokenType::Bang => write!(f, "NOT ! null"),
            TokenType::BangEquals => write!(f, "NOT_EQUALS != null"),
            TokenType::Slash => write!(f, "SLASH / null"),
            TokenType::Number(val) => match val.parse::<f64>() {
                Ok(num) => write!(f, "NUMBER {} {}", val, num),
                Err(_) => write!(
                    f,
                    "[line {}] Error: Invalid number literal: {}",
                    self.line, val
                ),
            },
            TokenType::Identifier(ident) => write!(f, "IDENTIFIER {} null", ident),
            TokenType::Reserved(reserved_word) => {
                write!(
                    f,
                    "{} {}",
                    format!("{:?}", reserved_word),
                    format!("{:?}", reserved_word).to_lowercase()
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,
    String(String),
    Eof,
    Error(char, usize), // unexpected character with line
    Equals,
    DoubleEquals,
    Greater,
    GreaterEquals,
    LessThan,
    LessThanEquals,
    Bang,
    BangEquals,
    Slash,
    Number(String),
    Identifier(String),
    Reserved(ReservedWords),
}

#[derive(Clone, Copy, Debug)]
pub enum ReservedWords {
    AND,
    CLASS,
    FOR,
    FALSE,
    ELSE,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
}

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    current: usize,
    line: usize,
    in_string: bool,
    string_buffer: String,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            current: 0,
            line: 1,
            in_string: false,
            string_buffer: String::new(),
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn advance(&mut self) -> Option<char> {
        let mut chars = self.source[self.current..].chars();
        let ch = chars.next()?;
        self.current += ch.len_utf8();
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn identifier(&mut self, curr: char) -> TokenType {
        let mut map: HashMap<&'static str, ReservedWords> = HashMap::new();
        map.insert("and", ReservedWords::AND);
        map.insert("class", ReservedWords::CLASS);
        map.insert("else", ReservedWords::ELSE);
        map.insert("false", ReservedWords::FALSE);
        map.insert("for", ReservedWords::FOR);
        map.insert("fun", ReservedWords::FUN);
        map.insert("if", ReservedWords::IF);
        map.insert("nil", ReservedWords::NIL);
        map.insert("or", ReservedWords::OR);
        map.insert("print", ReservedWords::PRINT);
        map.insert("return", ReservedWords::RETURN);
        map.insert("super", ReservedWords::SUPER);
        map.insert("this", ReservedWords::THIS);
        map.insert("true", ReservedWords::TRUE);
        map.insert("var", ReservedWords::VAR);
        map.insert("while", ReservedWords::WHILE);

        let mut identifier = String::new();
        identifier.push(curr);
        while let Some(val) = self.peek() {
            if val.is_ascii_alphanumeric() || val == '_' {
                identifier.push(val);
                self.advance();
            } else {
                break;
            }
        }

        match map.get(identifier.as_str()) {
            Some(reserved) => TokenType::Reserved(*reserved),
            None => TokenType::Identifier(identifier),
        }
    }

    fn scan_number(&mut self, curr: char) -> String {
        let mut number = String::new();
        number.push(curr);

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if let (Some('.'), Some(next_digit)) = (self.peek(), self.peek_next()) {
            if next_digit.is_ascii_digit() {
                number.push('.'); // consume '.'
                self.advance();

                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        number.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        return number;
    }

    fn peek_next(&self) -> Option<char> {
        self.source[self.current..].chars().nth(1)
    }

    fn match_next(&mut self, expected: char) -> bool {
        if let Some(next) = self.peek() {
            if next == expected {
                self.advance(); // consume it
                return true;
            }
        }
        false
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.at_end() {
            let ch = self.advance().unwrap();
            match ch {
                '\n' => {
                    self.line += 1;
                }
                '(' => self.push(TokenType::LeftParen),
                ')' => self.push(TokenType::RightParen),
                '{' => self.push(TokenType::LeftBrace),
                '}' => self.push(TokenType::RightBrace),
                ',' => self.push(TokenType::Comma),
                '.' => self.push(TokenType::Dot),
                '-' => self.push(TokenType::Minus),
                '+' => self.push(TokenType::Plus),
                '*' => self.push(TokenType::Star),
                ';' => self.push(TokenType::Semicolon),
                '=' => {
                    if self.match_next('=') {
                        self.push(TokenType::DoubleEquals);
                    } else {
                        self.push(TokenType::Equals);
                    }
                }
                '>' => {
                    if self.match_next('=') {
                        self.push(TokenType::GreaterEquals);
                    } else {
                        self.push(TokenType::Greater);
                    }
                }
                '<' => {
                    if self.match_next('=') {
                        self.push(TokenType::LessThanEquals);
                    } else {
                        self.push(TokenType::LessThan);
                    }
                }
                '!' => {
                    if self.match_next('=') {
                        self.push(TokenType::BangEquals);
                    } else {
                        self.push(TokenType::Bang);
                    }
                }
                '"' => {
                    self.in_string = !self.in_string;
                    if !self.in_string {
                        self.tokens.push(Token {
                            token_type: TokenType::String(self.string_buffer.clone()),
                            line: self.line,
                        });
                        self.string_buffer.clear();
                    }
                }
                '/' => {
                    if self.match_next('/') {
                        self.skip_line_comment();
                    } else {
                        self.push(TokenType::Slash);
                    }
                }
                _ if ch.is_ascii_digit() => {
                    let val = self.scan_number(ch);
                    self.tokens.push(Token {
                        token_type: TokenType::Number(val),
                        line: self.line,
                    });
                }

                _ if self.in_string => {
                    self.string_buffer.push(ch);
                }
                _ if ch.is_whitespace() => {}
                _ if ch.is_ascii_alphabetic() => {
                    let ident = self.identifier(ch);
                    self.tokens.push(Token {
                        token_type: ident,
                        line: self.line,
                    });
                }
                _ => self.tokens.push(Token {
                    token_type: TokenType::Error(ch, self.line),
                    line: self.line,
                }),
            }
        }
        if self.in_string {
            self.tokens.push(Token {
                token_type: TokenType::Error('"', self.line),
                line: self.line,
            });
        }
        self.push(TokenType::Eof);
        self.tokens
    }

    fn push(&mut self, kind: TokenType) {
        self.tokens.push(Token {
            token_type: kind,
            line: self.line,
        });
    }
}

fn run_tokenizer(source: &str) {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{}", token);
    }
}
