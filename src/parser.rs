use crate::{
    ast::{Expr, Literal, Stmt},
    token::Token,
};

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    function_depth: usize, // 跟踪函数嵌套深度
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0, function_depth: 0, }
    }

    // --------------------------------------------
    // 核心方法框架
    // --------------------------------------------

    /// 消费当前token并前进
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }


    /// 检查当前token是否匹配给定类型
    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(token_type)
        }
    }

    /// 如果匹配则消费token
    fn consume(&mut self, expected: &Token, message: &str) -> Result<&Token, String> {
        if self.check(expected) {
            Ok(self.advance())
        } else {
            let show_details_messages = &[
                "Expect ';' after value.",
                "Expect ')' after parameters.",
                "Expect variable name.",
                "Unexpected character.",
                "Expect '}' after block.",
                "Can't return from top-level code.",
                "Can't use 'break' outside of a loop.",
                "Can't use 'continue' outside of a loop.",
                "Already a variable with this name in this scope.",
                "Superclass must be a class.",
                "Can't use 'super' in a class with no superclass.",
                "Can't use 'this' outside of a class."
            ];
            let should_show_details = show_details_messages.iter()
            .any(|&m| message.starts_with(m));
        
            if should_show_details {
                Err(self.error(self.peek(), message))
            } else {
                // 对于不需要详细信息的错误，只返回错误类型部分
                // let error_type = message.split(':').next().unwrap_or(message).trim();
                // Err(error_type.to_string())
                Err("".to_string())
            }
            //Err(self.error(self.peek(), message))
        }
    }

    /// 查看当前token
    fn peek(&self) -> &Token {
        if self.is_at_end() {
            &Token::Eof
        } else {
            &self.tokens[self.current]
        }
    }

    /// 查看前一个token
    fn previous(&self) -> &Token {
        if self.current == 0 {
            &Token::Eof
        } else {
            &self.tokens[self.current - 1]
        }
    }

    /// 是否到达末尾
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }


    // --------------------------------------------
    // 解析入口点
    // --------------------------------------------

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    eprintln!("{}", e);
                    self.synchronize();
                }
            }
        }
        statements
    }

    // --------------------------------------------
    // 声明解析框架
    // --------------------------------------------

    fn declaration(&mut self) -> Result<Stmt, String> {
        let result = if self.check(&Token::Class) {
            self.class_declaration()
        } else if self.check(&Token::Fun) {
            self.function_declaration()
        } else if self.check(&Token::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };
        
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn class_declaration(&mut self) -> Result<Stmt, String> {
        let _class_token = self.advance().clone();
        let name = self.consume_identifier("Expect class name")?;
        
        let superclass = if self.check(&Token::Less) {
            self.advance();
            let super_name = self.consume_identifier("Expect superclass name")?;
            Some(Expr::Variable(super_name))
        } else {
            None
        };
        
        self.consume(&Token::LeftBrace, "Expect '{' before class body")?;
        
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            methods.push(self.method()?);
        }
        
        self.consume(&Token::RightBrace, "Expect '}' after class body")?;
        Ok(Stmt::Class {
            name,
            superclass,
            methods,
        })
    }
    
    // 新增方法：专门解析类方法
    fn method(&mut self) -> Result<Stmt, String> {
        let name = self.consume_identifier("Expect method name")?;
        
        self.consume(&Token::LeftParen, "Expect '(' after method name")?;
        
        let mut params = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(self.error(self.peek(), "Can't have more than 255 parameters"));
                }
                
                params.push(self.consume_identifier("Expect parameter name")?);
                
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        self.consume(&Token::RightParen, "Expect ')' after parameters.")?;
        
        let body = if self.check(&Token::LeftBrace) {
            self.function_depth += 1;
            let block_body = self.block()?;
            self.function_depth -= 1;
            block_body
        } else {
            // 如果方法体不是块语句，则创建只包含一个语句的块
            let stmt = self.statement()?;
            vec![stmt]
        };
        
        Ok(Stmt::Function {
            name,
            params,
            body,
        })
    }
    

    fn function_declaration(&mut self) -> Result<Stmt, String> {
        let _fun_token = self.advance().clone();  // 消费'fun'
        let name = self.consume_identifier("Expect function name")?;
        
        self.consume(&Token::LeftParen, "Expect '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(self.error(self.peek(), "Can't have more than 255 parameters"));
                }
                
                let param = self.consume_identifier("Expect parameter name")?;
                params.push(param);
                
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();  // 消费逗号
            }
        }
        
        self.consume(&Token::RightParen, "Expect ')' after parameters.")?;
        
        let body = if self.check(&Token::LeftBrace) {
            self.function_depth += 1;
            let block_body = self.block()?;
            self.function_depth -= 1;
            block_body
        } else {
            return Err(self.error(self.peek(), "Expect '{' before function body"));
        };
        
        Ok(Stmt::Function {
            name,
            params,
            body,
        })
    }
    
    // 添加辅助方法用于消费标识符
    fn consume_identifier(&mut self, message: &str) -> Result<Token, String> {
        if let Token::Identifier(_) = self.peek() {
            Ok(self.advance().clone())
        } else {
            let show_details_messages = &[
                "Expect ';' after value.",
                "Expect ')' after parameters.",
                "Expect variable name.",
                "Unexpected character.",
                "Expect '}' after block.",
                "Can't return from top-level code.",
                "Can't use 'break' outside of a loop.",
                "Can't use 'continue' outside of a loop.",
                "Already a variable with this name in this scope.",
                "Superclass must be a class.",
                "Can't use 'super' in a class with no superclass.",
                "Can't use 'this' outside of a class."
            ];
            let should_show_details = show_details_messages.iter()
            .any(|&m| message.starts_with(m));
        
            if should_show_details {
                Err(self.error(self.peek(), message))
            }
            else {
                // let error_type = message.split(':').next().unwrap_or(message).trim();
                // Err(error_type.to_string())
                Err("".to_string())
            }
            //Err(self.error(self.peek(), message))
        }
    }
    


    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let _var_token = self.advance().clone();
        let name = if let Token::Identifier(name) = self.advance().clone() {
            Token::Identifier(name)
        } else {
            return Err(self.error(self.previous(), "Expect variable name."));
        };
        
        let initializer = if self.check(&Token::Equal) {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };
        
        self.consume(&Token::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Var { name, initializer })
    }

    // --------------------------------------------
    // 语句解析框架
    // --------------------------------------------

    fn statement(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Token::Print => self.print_statement(),
            Token::LeftBrace => Ok(Stmt::Block(self.block()?)),
            Token::If => self.if_statement(),
            Token::While => self.while_statement(),
            Token::For => self.for_statement(),
            Token::Return => self.return_statement(),
            _ => self.expr_statement(),
        }
    }

    fn expr_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(&Token::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expr(expr))
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        let for_token = self.advance().clone();
        self.consume(&Token::LeftParen, "Expect '(' after 'for'")?;
        
        // 初始化部分
        let initializer = if self.check(&Token::Semicolon) {
            self.advance();
            None
        } else if self.check(&Token::Var) {
            Some(Box::new(self.var_declaration()?))
        } else {
            Some(Box::new(self.expr_statement()?))
        };
        
        // 条件部分
        let condition = if !self.check(&Token::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&Token::Semicolon, "Expect ';' after value.")?;
        
        // 增量部分
        let increment = if !self.check(&Token::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&Token::RightParen, "Expect ')' after for clauses")?;
        
        let mut body = self.statement()?;
        
        // 将for循环转换为while循环
        if let Some(inc) = increment {
            body = Stmt::Block(vec![
                body,
                Stmt::Expr(inc),
            ]);
        }
        
        body = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal(Literal::Bool(true))),
            body: Box::new(body),
        };
        
        if let Some(init) = initializer {
            body = Stmt::Block(vec![
                *init,
                body,
            ]);
        }
        
        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let if_token = self.advance().clone();
        self.consume(&Token::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&Token::RightParen, "Expect ')' after if condition")?;
        
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.check(&Token::Else) {
            self.advance();
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let print_token = self.advance().clone();
        let expr = self.expression()?;
        self.consume(&Token::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let keyword = self.advance().clone();
        // 检查是否在函数内
        if self.function_depth == 0 {
            return Err("Error: Can't return from top-level code.".to_string());
        }
        let value = if !self.check(&Token::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&Token::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        let while_token = self.advance().clone();
        self.consume(&Token::LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&Token::RightParen, "Expect ')' after condition")?;
        let body = Box::new(self.statement()?);
        
        Ok(Stmt::While { condition, body })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.consume(&Token::LeftBrace, "Expect '{' before block")?;
        
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        
        self.consume(&Token::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    // --------------------------------------------
    // 表达式解析框架 (按优先级从低到高)
    // --------------------------------------------

    /// expression → assignment
    pub fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    /// assignment → ( call "." )? IDENTIFIER "=" assignment
    ///            | logic_or
    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.logic_or()?;
        
        if self.check(&Token::Equal) {
            let equals = self.advance().clone();
            let value = self.assignment()?; // 递归解析右值
            
            // 处理普通变量赋值（a = 3）
            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign { name, value: Box::new(value) });
            }
            // 处理属性赋值（a.b = 3）
            else if let Expr::Get { object, name } = expr {
                return Ok(Expr::Set { object, name, value: Box::new(value) });
            }
            
            return Err(self.error(&equals, "Invalid assignment target"));
        }
        
        Ok(expr)
    }
    

    /// logic_or → logic_and ( "or" logic_and )*
    fn logic_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.logic_and()?;
        
        while self.check(&Token::Or) {
            let operator = self.advance().clone();
            let right = self.logic_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    /// logic_and → equality ( "and" equality )*
    fn logic_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;
        
        while self.check(&Token::And) {
            let operator = self.advance().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    /// equality → comparison ( ("!=" | "==") comparison )*
    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        
        while matches!(self.peek(), Token::BangEqual | Token::EqualEqual) {
            let operator = self.advance().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    /// comparison → term ( (">" | ">=" | "<" | "<=" ) term )*
    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        
        while matches!(
            self.peek(),
            Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual
        ) {
            let operator = self.advance().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    /// term → factor ( ("+" | "-") factor )*
    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let operator = self.advance().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    /// factor → unary ( ("/" | "*") unary )*
    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        
        while matches!(self.peek(), Token::Slash | Token::Star) {
            let operator = self.advance().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    /// unary → ("!" | "-") unary | call
    fn unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Token::Bang | Token::Minus) {
            let operator = self.advance().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.call()
        }
    }

    /// call → primary ( "(" arguments? ")" | "." IDENTIFIER )*
    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        
        loop {
            if self.check(&Token::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.check(&Token::Dot) {
                let _dot = self.advance().clone();
                let name = self.consume(
                    &Token::Identifier("".to_string()),
                    "Expect property name after '.'"
                )?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name: name.clone(),
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    /// 辅助方法：处理函数调用参数
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let _paren = self.advance();
        let mut arguments = Vec::new();
        
        if !self.check(&Token::RightParen) {
            loop {
                arguments.push(self.expression()?);
                
                if !self.check(&Token::Comma) {
                    break;
                }
                
                self.advance();
                
                if arguments.len() >= 255 {
                    return Err(self.error(self.peek(), "Can't have more than 255 arguments"));
                }
            }
        }
        
        let paren = self.consume(&Token::RightParen, "Expect ')' after arguments")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren: paren.clone(),
            arguments,
        })
    }

    /// primary → "true" | "false" | "nil" | "this"
    ///         | NUMBER | STRING | IDENTIFIER 
    ///         | "(" expression ")"
    ///         | "super" "." IDENTIFIER
    fn primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Token::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Token::Nil => {
                self.advance();
                Ok(Expr::Literal(Literal::Nil))
            }
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Number(value)))
            }
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(value)))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&Token::RightParen, "Expect ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            Token::This => {
                let token = self.advance().clone();
                Ok(Expr::This(token))
            }
            Token::Super => {
                let keyword = self.advance().clone();
                self.consume(&Token::Dot, "Expect '.' after 'super'")?;
                let method = self.consume_identifier("Expect superclass method name")?;
                Ok(Expr::Super { keyword, method })
            }
            Token::Identifier(_) => {
                let token = self.advance().clone();
                Ok(Expr::Variable(token))
            }
            //_ => Err(self.error(self.peek(), "Expect expression")),
            _ => Err("".to_string()),
        }
    }
    

    // --------------------------------------------
    // 辅助方法框架
    // --------------------------------------------

    fn error(&self, token: &Token, message: &str) -> String {
        format!("Error at '{}': {}", token.lexeme(), message)
    }

    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            if let Token::Semicolon = self.previous() {
                return;
            }
            
            match self.peek() {
                Token::Class | Token::Fun | Token::Var | 
                Token::For | Token::If | Token::While |
                Token::Print | Token::Return => return,
                _ => {self.advance();}
            }
        }
    }
}
