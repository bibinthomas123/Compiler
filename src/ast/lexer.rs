use std::fmt::{Display, Formatter};
use crate::text::span::TextSpan;



#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Literals
    Number(i64),
    Decimal(f64),
    String(String),
    StringEnd,
    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Equals,
    Ampersand,
    Dot,
    Pipe,
    Caret,
    DoubleAsterisk,
    Tilde,
    GreaterThan,
    LessThan,
    GreaterThanEquals,
    LessThanEquals,
    EqualsEquals,
    BangEquals,
    // Keywords
    Let,
    If,
    Else,
    True,
    False,
    While,
    Func,
    Return,
    // Separators
    LeftParen,
    RightParen,
    OpenBrace,
    CloseBrace,
    Comma,
    Colon,
    SemiColon,
    Arrow,
    // Other
    Bad,
    Whitespace,
    Identifier,
    Eof,
    // Null
  
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringFragment {
    Literal { len: usize },
    Interpolation { tokens: Vec<Token> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringKind {
    // Delimited by one double-quote: "
    Normal,
    // Delimited by two single-quotes: ''
    Indented,
}


impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Number(_) => write!(f, "Number"),
            TokenKind::Decimal(_) => write!(f, "Decimal"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Asterisk => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Bad => write!(f, "Bad"),
            TokenKind::Whitespace => write!(f, "Whitespace"),
            TokenKind::Eof => write!(f, "Eof"),
            TokenKind::Let => write!(f, "Let"),
            TokenKind::Identifier => write!(f, "Identifier"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::DoubleAsterisk => write!(f, "**"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::If => write!(f, "If"),
            TokenKind::Else => write!(f, "Else"),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::GreaterThanEquals => write!(f, ">="),
            TokenKind::LessThanEquals => write!(f, "<="),
            TokenKind::EqualsEquals => write!(f, "=="),
            TokenKind::BangEquals => write!(f, "!="),
            TokenKind::OpenBrace => write!(f, "{{"),
            TokenKind::CloseBrace => write!(f, "}}"),
            TokenKind::True => write!(f, "True"),
            TokenKind::False => write!(f, "False"),
            TokenKind::While => write!(f, "While"),
            TokenKind::Func => write!(f, "Func"),
            TokenKind::Return => write!(f, "Return"),
            TokenKind::Comma => write!(f, "Comma"),
            TokenKind::Colon => write!(f, "Colon"),
            TokenKind::Arrow => write!(f, "Arrow"),
            TokenKind::SemiColon => write!(f, ";"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::String { .. } => write!(f, "String"),
            TokenKind::StringEnd => write!(f, "StringEnd"),

        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: TextSpan,
}

impl Token {
    pub fn new(kind: TokenKind, span: TextSpan) -> Self {
        Self { kind, span }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    current_pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, current_pos: 0 }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if self.current_pos == self.input.len() {
            let eof_char: char = '\0';
            self.current_pos += 1;
            return Some(Token::new(
                TokenKind::Eof,
                TextSpan::new(0, 0, eof_char.to_string()),
            ));
        }
        let c = self.current_char();
        return c.map(|c| {
            let start = self.current_pos;
            let mut kind = TokenKind::Bad;
            if Self::is_number_start(&c) {
                let number: i64 = self.consume_number();
                kind = TokenKind::Number(number);
            } 
            else if Self::is_decimal(&c){
                let decimal: f64 = self.consume_decimal();
                println!("{} from is decimal",decimal);
                kind = TokenKind::Decimal(decimal); 
            }
            else if Self::is_whitespace(&c){
                self.consume();
                kind = TokenKind::Whitespace;
            }
            else if Self::is_string_start(&c) {
                let string_literal = self.consume_string();
                // kind = TokenKind::String {
                //     fragments: StringFragment::Literal { len: string_literal.len() },
                //     kind: StringKind::Normal,
                //     is_terminated: true,
                // };
                println!("{} from is string",string_literal);
                kind = TokenKind::String(string_literal);
            } 
            
            else if Self::is_identifier_start(&c){
                let identifier = self.consume_identifier();
                kind = match identifier.as_str() {
                    "let" => TokenKind::Let,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "while" => TokenKind::While,
                    "func" => TokenKind::Func,
                    "return" => TokenKind::Return,
                    _ => TokenKind::Identifier,
                }

            } else  {
                kind = self.consume_punctuation();
            }

            let end = self.current_pos;
            let literal = self.input[start..end].to_string();
            let span = TextSpan::new(start, end, literal);
            Token::new(kind, span)
        });
    }

    fn consume_punctuation(&mut self) -> TokenKind {
        let c = self.consume().unwrap();
        match c {
            '+' => TokenKind::Plus,
            '-' => self.lex_potential_double_char_operator('>', TokenKind::Minus, TokenKind::Arrow),
            '*' => {
                self.lex_potential_double_char_operator('*', TokenKind::Asterisk, TokenKind::DoubleAsterisk)
            },
            '/' => TokenKind::Slash,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '=' => {
                self.lex_potential_double_char_operator('=', TokenKind::Equals, TokenKind::EqualsEquals)
            },
            '&' => TokenKind::Ampersand,
            '|' => TokenKind::Pipe,
            '^' => TokenKind::Caret,
            '~' => TokenKind::Tilde,
            '>' => {
                self.lex_potential_double_char_operator('=', TokenKind::GreaterThan, TokenKind::GreaterThanEquals)
            },
            '<' => {
                self.lex_potential_double_char_operator('=', TokenKind::LessThan, TokenKind::LessThanEquals)
            },
            '!' => {
                self.lex_potential_double_char_operator('=', TokenKind::Bad, TokenKind::BangEquals)
            },
            '{' => {
                TokenKind::OpenBrace
            },
            '}' => {
                TokenKind::CloseBrace
            },
            ',' => {
                TokenKind::Comma
            },
            ':' => {
                TokenKind::Colon
            },
            ';' => {
                TokenKind::SemiColon
            }

            _ => TokenKind::Bad,
        }
    }

    fn lex_potential_double_char_operator(&mut self, expected: char, one_char_kind: TokenKind, double_char_kind: TokenKind) -> TokenKind {
        if let Some(next) = self.current_char() {
            if next == expected {
                self.consume();
                double_char_kind
            } else {
                one_char_kind
            }
        } else {
            one_char_kind
        }
    }

    fn is_number_start(c: &char) -> bool {
        c.is_digit(10)
    }

    fn is_string_start(c: &char) -> bool {
        *c == '"' || *c == '\''
    }

    fn is_identifier_start(c: &char) -> bool {
        c.is_alphabetic()
    }

    fn is_decimal(c: &char) -> bool {
       *c == 'd'  
    }

    fn is_whitespace(c: &char) -> bool {
        c.is_whitespace()
    }

    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.current_pos)
    }

    fn consume(&mut self) -> Option<char> {
        if self.current_pos >= self.input.len() {
            return None;
        }
        let c = self.current_char();
        self.current_pos += 1;

        c
    }

    fn consume_identifier(&mut self) -> String {
        let mut identifier = String::new();
        while let Some(c) = self.current_char() {
            if Self::is_identifier_start(&c) {
                self.consume().unwrap();
                identifier.push(c);
            } else {
                break;
            }
        }
        identifier
    }

    fn consume_number(&mut self) -> i64 {
        let mut number: i64 = 0;
        while let Some(c) = self.current_char() {
            if c.is_digit(10) {
                self.consume().unwrap();
                number = number * 10 + c.to_digit(10).unwrap() as i64;
            } else {
                break;
            }
        }
        number
    }

    fn consume_string(&mut self) -> String {
        let mut string = String::new();
        let quote_char = self.consume().unwrap(); // Consume the opening quote character

        while let Some(c) = self.current_char() {
            if c == quote_char {
                self.consume().unwrap(); // Consume the closing quote character
                break;
            }
            string.push(c);
            self.consume().unwrap();
        }
        string
    }

    
    fn consume_decimal(&mut self) -> f64 {
        // Check for the 'f' prefix
        let is_float = if let Some('d') = self.current_char() {
            self.consume().unwrap(); // Consume the 'f'
            true
        } else {
            false
        };
    
        // Parse the float literal without the 'f' prefix
        let float_literal = self.parse_float_literal();
        // Adjust the result based on whether it's a float or not
        if is_float {
            float_literal
        } else {
            float_literal as f64
        }
    }
    
    fn parse_float_literal(&mut self) -> f64 {
        let start = self.current_pos;

    
        // Consume the digits before the decimal point
        while let Some(c) = self.current_char() {
            if c.is_digit(10) {
                self.consume().unwrap();
            } else {
                break;
            }
        }
    
        // Check for the decimal point
        if let Some('.') = self.current_char() {
            self.consume().unwrap(); // Consume the decimal point
    
            // Consume the digits after the decimal point
            while let Some(c) = self.current_char() {
                if c.is_digit(10) {
                    self.consume().unwrap();
                } else {
                    break;
                }
            }
        }
    
        // Parse the consumed characters as a float
        let literal = &self.input[start..self.current_pos];
        literal.parse().unwrap_or(0.0)
    }
    
    
    
}