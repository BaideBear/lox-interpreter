use std::str::Chars;
use std::iter::Peekable;
use crate::token::Token;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            input: source.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        match self.input.peek() {
            Some(&ch) => match ch {
                // 单字符 token
                '(' => { self.input.next(); Token::LeftParen },
                ')' => { self.input.next(); Token::RightParen },
                '{' => { self.input.next(); Token::LeftBrace },
                '}' => { self.input.next(); Token::RightBrace },
                ',' => { self.input.next(); Token::Comma },
                '.' => { self.input.next(); Token::Dot },
                '-' => { self.input.next(); Token::Minus },
                '+' => { self.input.next(); Token::Plus },
                ';' => { self.input.next(); Token::Semicolon },
                '*' => { self.input.next(); Token::Star },
                
                // 可能双字符的 token
                '!' => {
                    self.input.next();
                    if self.match_char('=') {
                        Token::BangEqual
                    } else {
                        Token::Bang
                    }
                },
                '=' => {
                    self.input.next();
                    if self.match_char('=') {
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                },
                '<' => {
                    self.input.next();
                    if self.match_char('=') {
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                },
                '>' => {
                    self.input.next();
                    if self.match_char('=') {
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                },
                '/' => {
                    self.input.next();
                    if self.match_char('/') {
                        // 注释，跳过直到行尾
                        while let Some(&ch) = self.input.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.input.next();
                        }
                        self.next_token()  // 递归调用处理注释后的内容
                    } else {
                        Token::Slash
                    }
                },
                
                // 字符串字面量
                '"' => self.string(),
                
                // 数字字面量
                '0'..='9' => self.number(),
                
                // 标识符或关键字
                'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
                
                _ => panic!("Error at '{}': Unexpected character.", ch),
            },
            None => Token::Eof,
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }
    }
    
    fn match_char(&mut self, expected: char) -> bool {
        if let Some(&ch) = self.input.peek() {
            if ch == expected {
                self.input.next();
                return true;
            }
        }
        false
    }
    
    fn string(&mut self) -> Token {
        self.input.next(); // 跳过开始的引号
        
        let mut s = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch == '"' {
                self.input.next(); // 跳过结束的引号
                return Token::String(s);
            }
            s.push(ch);
            self.input.next();
        }
        
        panic!("Unterminated string");
    }
    
    fn number(&mut self) -> Token {
        let mut num = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                num.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        
        Token::Number(num.parse().unwrap())
    }
    
    fn identifier(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        
        // 检查是否是关键字
        match ident.as_str() {
            "and" => Token::And,
            "class" => Token::Class,
            "else" => Token::Else,
            "false" => Token::False,
            "fun" => Token::Fun,
            "for" => Token::For,
            "if" => Token::If,
            "nil" => Token::Nil,
            "or" => Token::Or,
            "print" => Token::Print,
            "return" => Token::Return,
            "super" => Token::Super,
            "this" => Token::This,
            "true" => Token::True,
            "var" => Token::Var,
            "while" => Token::While,
            _ => Token::Identifier(ident),
        }
    }
}
