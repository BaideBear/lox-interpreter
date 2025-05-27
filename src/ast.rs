use crate::token::Token;

#[derive(Debug,Clone)]
pub enum Stmt {
    // 表达式语句
    Expr(Expr),
    // 打印语句
    Print(Expr),
    // 变量声明
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    // 块语句
    Block(Vec<Stmt>),
    // 条件语句
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    // while循环
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    // for循环(后续会被desugar成while)
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Stmt>,
    },
    // 函数声明
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    // 返回语句
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    // 类声明
    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    // 基础字面量
    Literal(Literal),
    // 变量引用
    Variable(Token),
    // 赋值表达式
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    // 逻辑表达式
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    // 二元运算
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    // 一元运算
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    // 调用表达式
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    // 属性访问
    Get {
        object: Box<Expr>,
        name: Token,
    },
    // 属性设置
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    // this表达式
    This(Token),
    // super表达式
    Super {
        keyword: Token,
        method: Token,
    },
    // 分组表达式
    Grouping(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Null,
}
