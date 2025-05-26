use lox_interpreter::{lexer::Lexer, parser::Parser, Expr, Stmt,Literal};
use lox_interpreter::token::Token;
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref GLOBAL_MAP: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
}

pub fn traverse_statements(statements: &Vec<Stmt>,depth: usize) {
    for stmt in statements {
        traverse_stmt(stmt,depth);
    }
}

pub fn traverse_stmt(stmt: &Stmt,depth: usize) {
    for _ in 0..depth {
        print!("  ");
    }
    match stmt {
        Stmt::Expr(expr) => {
            println!("ExpressionStmt");
            traverse_expr(expr,depth+1);
        }
        Stmt::Print(expr) => {
            println!("PrintStmt");
            let value: Literal = traverse_expr(expr,depth+1);
            match value {
                Literal::Number(num) => println!("{}", num),
                Literal::String(s) => println!("{}", s),
                Literal::Bool(b) => println!("{}", b),
                Literal::Nil => println!("nil"),
            }
        }
        Stmt::Var { name, initializer } => {
            println!("VarStmt: {:?}", name);
            if let Some(expr) = initializer {
                traverse_expr(expr,depth+1);
            }
        }
        Stmt::Block(stmts) => {
            println!("BlockStmt");
            for s in stmts {
                traverse_stmt(s,depth+1);
            }
        }
        Stmt::If { condition, then_branch, else_branch } => {
            println!("IfStmt");
            traverse_expr(condition,depth+1);
            traverse_stmt(then_branch,depth+1);
            if let Some(else_branch) = else_branch {
                traverse_stmt(else_branch,depth + 1);
            }
        }
        Stmt::While { condition, body } => {
            println!("WhileStmt");
            traverse_expr(condition,depth+1);
            traverse_stmt(body,depth+1);
        }
        Stmt::For { initializer, condition, increment, body } => {
            println!("ForStmt");
            if let Some(init) = initializer {
                traverse_stmt(init,depth + 1);
            }
            if let Some(cond) = condition {
                traverse_expr(cond,depth + 1);
            }
            if let Some(inc) = increment {
                traverse_expr(inc,depth + 1);
            }
            traverse_stmt(body,depth + 1);
        }
        Stmt::Function { name, params, body } => {
            println!("FunctionStmt: {:?}", name);
            for param in params {
                println!("  Param: {:?}", param);
            }
            traverse_statements(body,depth + 1);
        }
        Stmt::Return { keyword, value } => {
            println!("ReturnStmt: {:?}", keyword);
            if let Some(expr) = value {
                traverse_expr(expr,depth + 1);  
            }
        }
        Stmt::Class { name, superclass, methods } => {
            println!("ClassStmt: {:?}", name);
            if let Some(superclass_expr) = superclass {
                traverse_expr(superclass_expr,depth + 1);
            }
            for method in methods {
                traverse_stmt(method,depth + 1);
            }
        }
    }
}

pub fn traverse_expr(expr: &Expr,depth: usize) -> Literal {
    for _ in 0..depth {
        print!("  ");
    }
    match expr {
        Expr::Literal(literal) => {
            println!("LiteralExpr: {:?}", literal);
            let val: Literal = traverse_literal(literal, depth + 1);
            val
        }
        Expr::Variable(token) => {
            println!("VariableExpr: {:?}", token);
            Literal::Number(0.0)
        }
        Expr::Assign { name, value } => {
            println!("AssignExpr: {:?}", name);
            traverse_expr(value,depth+1);
            Literal::Number(0.0)
        }
        Expr::Logical { left, operator, right } => {
            println!("LogicalExpr: {:?}", operator);
            traverse_expr(left,depth+1);
            traverse_expr(right,depth+1);
            Literal::Number(0.0)
        }
        Expr::Binary { left, operator, right } => {
            println!("BinaryExpr: {:?}", operator);
            let left_value: Literal = traverse_expr(left,depth+1);
            let right_value: Literal = traverse_expr(right,depth+1);
            let mut result: Literal = Literal::Number(0.0);
            let left_num = match left_value {
                Literal::Number(num) => num,
                _ => 0.0,
            };
            let right_num = match right_value {
                Literal::Number(num) => num,
                _ => 0.0,
            };
            match operator.lexeme() {
                "+" => result = Literal::Number(left_num + right_num),
                "-" => result = Literal::Number(left_num - right_num),
                "*" => result = Literal::Number(left_num * right_num),
                "/" => {
                    if right_num != 0.0 {
                        result = Literal::Number(left_num / right_num);
                    } else {
                        println!("  Error: Division by zero");
                    }
                }
                _ => println!("  Result: Unknown operation"),
            }
            result
        }
        Expr::Unary { operator, right } => {
            println!("UnaryExpr: {:?}", operator);
            let value: Literal = traverse_expr(right, depth + 1);
            let mut result: Literal = Literal::Number(0.0);
            let num = match value {
                Literal::Number(n) => n,
                _ => 0.0,
            };
            match operator.lexeme() {
                "-" => result = Literal::Number(-num),
                "!" => result = if num == 0.0 { Literal::Number(1.0) } else { Literal::Number(0.0) },
                _ => println!("  Result: Unknown unary operation"),
            }
            result
        }
        Expr::Call { callee, paren, arguments } => {
            println!("CallExpr: {:?}", paren);
            traverse_expr(callee,depth+1);
            for arg in arguments {
                traverse_expr(arg,depth+1);
            }
            Literal::Number(0.0)
        }
        Expr::Get { object, name } => {
            println!("GetExpr: {:?}", name);
            traverse_expr(object,depth+1);
            Literal::Number(0.0)
        }
        Expr::Set { object, name, value } => {
            println!("SetExpr: {:?}", name);
            traverse_expr(object,depth+1);
            traverse_expr(value,depth+1);
            Literal::Number(0.0)
        }
        Expr::This(token) => {
            println!("ThisExpr: {:?}", token);
            Literal::Number(0.0)
        }
        Expr::Super { keyword, method } => {
            println!("SuperExpr: {:?} {:?}", keyword, method);
            Literal::Number(0.0)
        }
        Expr::Grouping(expr) => {
            println!("GroupingExpr");
            let val: Literal = traverse_expr(expr, depth + 1);
            val
        }
    }
}

pub fn traverse_literal(literal: &Literal, depth: usize) -> Literal{
    for _ in 0..depth {
        print!("  ");
    }
    match literal {
        lox_interpreter::Literal::Number(value) => {
            println!("NumberLiteral: {}", value);
            Literal::Number(value.clone())
        }
        lox_interpreter::Literal::String(value) => {
            println!("StringLiteral: {}", value);
            Literal::String(value.clone())
        }
        lox_interpreter::Literal::Bool(value) => {
            println!("BooleanLiteral: {}", value);
            Literal::Bool(*value)
        }
        lox_interpreter::Literal::Nil => {
            println!("NilLiteral");
            Literal::Nil
        }
    }
}