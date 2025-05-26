use lox_interpreter::{lexer::Lexer, parser::Parser, Expr, Stmt,Literal};
use lox_interpreter::token::Token;
use std::collections::HashMap;

pub fn traverse_statements(statements: &Vec<Stmt>,depth: usize,map: &mut HashMap<String, Literal>) {
    for stmt in statements {
        traverse_stmt(stmt,depth,map);
    }
}

pub fn traverse_stmt(stmt: &Stmt,depth: usize,map: &mut HashMap<String, Literal>) {
    for _ in 0..depth {
        print!("  ");
    }
    match stmt {
        Stmt::Expr(expr) => {
            println!("ExpressionStmt");
            traverse_expr(expr,depth+1,map);
        }
        Stmt::Print(expr) => {
            println!("PrintStmt");
            let value: Literal = traverse_expr(expr,depth+1,map);
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
                let value: Literal = traverse_expr(expr,depth+1,map);
                map.insert(name.lexeme().to_string(), value);
            }
            else{
                map.insert(name.lexeme().to_string(), Literal::Nil);
            }
        }
        Stmt::Block(stmts) => {
            println!("BlockStmt");
            let mut new_map: HashMap<String, Literal> = map.clone();
            for s in stmts {
                traverse_stmt(s,depth+1,&mut new_map);
            }
            for (key, value) in new_map {
                map.insert(key, value);
            }
        }
        Stmt::If { condition, then_branch, else_branch } => {
            println!("IfStmt");
            let cond: Literal = traverse_expr(condition, depth + 1, map);
            if let Literal::Bool(true) = cond {
                traverse_stmt(then_branch, depth + 1, map);
            } else {
                if let Some(else_branch) = else_branch {
                    traverse_stmt(else_branch, depth + 1, map);
                }
            }
        }
        Stmt::While { condition, body } => {
            println!("WhileStmt");
            loop{
                let cond: Literal = traverse_expr(condition, depth + 1, map);
                if let Literal::Bool(false) = cond {
                    break;
                }
                traverse_stmt(body, depth + 1, map);
            }
            //traverse_expr(condition, depth + 1, map);
            //traverse_stmt(body, depth + 1, map);
        }
        Stmt::For { initializer, condition, increment, body } => {
            println!("ForStmt");
            if let Some(init) = initializer {
                traverse_stmt(init,depth + 1,map);
            }
            if let Some(cond) = condition {
                traverse_expr(cond,depth + 1,map);
            }
            if let Some(inc) = increment {
                traverse_expr(inc,depth + 1,map);
            }
            traverse_stmt(body,depth + 1,map);
        }
        Stmt::Function { name, params, body } => {
            println!("FunctionStmt: {:?}", name);
            for param in params {
                println!("  Param: {:?}", param);
            }
            traverse_statements(body,depth + 1,map);
        }
        Stmt::Return { keyword, value } => {
            println!("ReturnStmt: {:?}", keyword);
            if let Some(expr) = value {
                traverse_expr(expr,depth + 1,map);  
            }
        }
        Stmt::Class { name, superclass, methods } => {
            println!("ClassStmt: {:?}", name);
            if let Some(superclass_expr) = superclass {
                traverse_expr(superclass_expr,depth + 1,map);
            }
            for method in methods {
                traverse_stmt(method,depth + 1,map);
            }
        }
    }
}

pub fn traverse_expr(expr: &Expr,depth: usize,map: &mut HashMap<String, Literal>) -> Literal {
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
            let value: Literal = match map.get(&token.lexeme().to_string()) {
                Some(val) => val.clone(),
                None => {
                    println!("  Error: Variable '{}' not found", token.lexeme());
                    Literal::Nil
                }
            };
            value
        }
        Expr::Assign { name, value } => {
            println!("AssignExpr: {:?}", name);
            let value: Literal = traverse_expr(value,depth+1,map);
            map.insert(name.lexeme().to_string(), value);
            Literal::Number(0.0)
        }
        Expr::Logical { left, operator, right } => {
            println!("LogicalExpr: {:?}", operator);
            traverse_expr(left,depth+1,map);
            traverse_expr(right,depth+1,map);
            Literal::Number(0.0)
        }
        Expr::Binary { left, operator, right } => {
            println!("BinaryExpr: {:?}", operator);
            let left_value: Literal = traverse_expr(left,depth+1,map);
            let right_value: Literal = traverse_expr(right,depth+1,map);
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
                ">" => {
                    result = Literal::Bool(left_num > right_num);
                }
                "<" => {
                    result = Literal::Bool(left_num < right_num);
                }
                ">=" => {
                    result = Literal::Bool(left_num >= right_num);
                }
                "<=" => {
                    result = Literal::Bool(left_num <= right_num);
                }
                "==" => {
                    result=match (left_value, right_value) {
                        (Literal::Number(l), Literal::Number(r)) => Literal::Bool(l == r),
                        (Literal::String(l), Literal::String(r)) => Literal::Bool(l == r),
                        (Literal::Bool(l), Literal::Bool(r)) => Literal::Bool(l == r),
                        (Literal::Nil, Literal::Nil) => Literal::Bool(true),
                        _ => Literal::Bool(false),
                    };
                }
                "!=" => {
                    result=match (left_value, right_value) {
                        (Literal::Number(l), Literal::Number(r)) => Literal::Bool(l != r),
                        (Literal::String(l), Literal::String(r)) => Literal::Bool(l != r),
                        (Literal::Bool(l), Literal::Bool(r)) => Literal::Bool(l != r),
                        (Literal::Nil, Literal::Nil) => Literal::Bool(false),
                        _ => Literal::Bool(true),
                    };
                }
                _ => println!("  Result: Unknown operation"),
            }
            result
        }
        Expr::Unary { operator, right } => {
            println!("UnaryExpr: {:?}", operator);
            let value: Literal = traverse_expr(right, depth + 1,map);
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
            traverse_expr(callee,depth+1,map);
            for arg in arguments {
                traverse_expr(arg,depth+1,map);
            }
            Literal::Number(0.0)
        }
        Expr::Get { object, name } => {
            println!("GetExpr: {:?}", name);
            traverse_expr(object,depth+1,map);
            Literal::Number(0.0)
        }
        Expr::Set { object, name, value } => {
            println!("SetExpr: {:?}", name);
            traverse_expr(object,depth+1,map);
            traverse_expr(value,depth+1,map);
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
            let val: Literal = traverse_expr(expr, depth + 1,map);
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