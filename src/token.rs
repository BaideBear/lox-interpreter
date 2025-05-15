#[derive(Debug, PartialEq, Clone)]

pub enum Token {
    // 单字符 token
    LeftParen,    // "("
    RightParen,   // ")"
    LeftBrace,    // "{"
    RightBrace,   // "}"
    Comma,        // ","
    Dot,          // "."
    Minus,        // "-"
    Plus,         // "+"
    Semicolon,    // ";"
    Slash,        // "/"
    Star,         // "*"
    
    // 可能是两个字符的 token
    Bang,         // "!"
    BangEqual,    // "!="
    Equal,        // "="
    EqualEqual,   // "=="
    Greater,      // ">"
    GreaterEqual, // ">="
    Less,         // "<"
    LessEqual,    // "<="
    
    // 字面量
    Identifier(String),  // 变量名/函数名等
    String(String),      // 字符串字面量
    Number(f64),         // 数字字面量
    
    // 关键字
    And,         // "and"
    Class,       // "class"
    Else,        // "else"
    False,       // "false"
    Fun,         // "fun"
    For,         // "for"
    If,          // "if"
    Nil,         // "nil"
    Or,          // "or"
    Print,       // "print"
    Return,      // "return"
    Super,       // "super"
    This,        // "this"
    True,        // "true"
    Var,         // "var"
    While,       // "while"
    
    // 特殊 token
    Eof,         // 文件结束
}

impl Token {
    pub fn line(&self) -> usize {
        // 实际实现需要跟踪行号
        0 // 暂时返回默认值
    }
    
    pub fn lexeme(&self) -> &str {
        match self {
            Token::Identifier(s) => s,
            Token::String(s) => s,
            Token::Number(n) => Box::leak(n.to_string().into_boxed_str()),
            Token::LeftParen => "(",
            Token::RightParen => ")",
            Token::LeftBrace => "{",
            Token::RightBrace => "}",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::Minus => "-",
            Token::Plus => "+",
            Token::Semicolon => ";",
            Token::Slash => "/",
            Token::Star => "*",
            Token::Bang => "!",
            Token::BangEqual => "!=",
            Token::Equal => "=",
            Token::EqualEqual => "==",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::And => "and",
            Token::Class => "class",
            Token::Else => "else",
            Token::False => "false",
            Token::Fun => "fun",
            Token::For => "for",
            Token::If => "if",
            Token::Nil => "nil",
            Token::Or => "or",
            Token::Print => "print",
            Token::Return => "return",
            Token::Super => "super",
            Token::This => "this",
            Token::True => "true",
            Token::Var => "var",
            Token::While => "while",
            Token::Eof => "EOF",
        }
    }
}
