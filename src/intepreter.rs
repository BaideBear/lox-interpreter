use lox_interpreter::{lexer::Lexer, parser::Parser, Expr, Stmt,Literal};
use lox_interpreter::token::Token;
use std::collections::HashMap;

pub fn traverse_statements(statements: &Vec<Stmt>,depth: usize,map: &mut HashMap<String, Literal>,
                           func: &mut HashMap<String,(Vec<Token>, Vec<Stmt>)>) ->Literal{
    for stmt in statements {
        let val: Literal=traverse_stmt(stmt,depth,map,func);
        match val {
            Literal::Number(_) => return val,
            Literal::String(_) => return val,
            Literal::Bool(_) => return val,
            Literal::Null => return val,
            _ => continue,
        }
    }
    return Literal::Nil
}

pub fn traverse_stmt(stmt: &Stmt,depth: usize,map: &mut HashMap<String, Literal>,
                     func: &mut HashMap<String,(Vec<Token>, Vec<Stmt>)>) -> Literal{
    for _ in 0..depth {
        print!("  ");
    }
    match stmt {
        Stmt::Expr(expr) => {
            println!("ExpressionStmt");
            traverse_expr(expr,depth+1,map,func);
            Literal::Nil
        }
        Stmt::Print(expr) => {
            println!("PrintStmt");
            let value: Literal = traverse_expr(expr,depth+1,map,func);
            match value {
                Literal::Number(num) => println!("{}", num),
                Literal::String(s) => println!("{}", s),
                Literal::Bool(b) => println!("{}", b),
                Literal::Nil => println!("nil"),
                _ => println!("Unknown value"),
            }
            Literal::Nil
        }
        Stmt::Var { name, initializer } => {
            println!("VarStmt: {:?}", name);
            if let Some(expr) = initializer {
                let value: Literal = traverse_expr(expr,depth+1,map,func);
                map.insert(name.lexeme().to_string(), value);
            }
            else{
                map.insert(name.lexeme().to_string(), Literal::Nil);
            }
            Literal::Nil
        }
        Stmt::Block(stmts) => {
            println!("BlockStmt");
            let mut new_map: HashMap<String, Literal> = map.clone();
            for s in stmts {
                traverse_stmt(s,depth+1,&mut new_map,func);
            }
            for (key, value) in new_map {
                map.insert(key, value);
            }
            Literal::Nil
        }
        Stmt::If { condition, then_branch, else_branch } => {
            println!("IfStmt");
            let cond: Literal = traverse_expr(condition, depth + 1, map, func);
            if let Literal::Bool(true) = cond {
                traverse_stmt(then_branch, depth + 1, map, func);
            } else {
                if let Some(else_branch) = else_branch {
                    traverse_stmt(else_branch, depth + 1, map, func);
                }
            }
            Literal::Nil
        }
        Stmt::While { condition, body } => {
            println!("WhileStmt");
            loop{
                let cond: Literal = traverse_expr(condition, depth + 1, map, func);
                if let Literal::Bool(false) = cond {
                    break;
                }
                traverse_stmt(body, depth + 1, map, func);
            }
            Literal::Nil
        }
        Stmt::For { initializer, condition, increment, body } => {
            println!("ForStmt");
            let mut new_map: HashMap<String, Literal> = map.clone();
            if let Some(init) = initializer {
                traverse_stmt(init,depth + 1,&mut new_map,func);
            }
            loop{
                if let Some(cond) = condition {
                    let cond = traverse_expr(cond,depth + 1,&mut new_map,func); 
                    if let Literal::Bool(false) = cond {
                        break;
                    }
                }
                traverse_stmt(body,depth + 1,&mut new_map,func);
                if let Some(inc) = increment {
                    traverse_expr(inc,depth + 1,&mut new_map,func);
                }
            }
            Literal::Nil
        }
        Stmt::Function { name, params, body } => {
            println!("FunctionStmt: {:?}", name);
            for param in params {
                println!("  Param: {:?}", param);
            }
            func.insert(name.lexeme().to_string(), (params.clone(), body.clone()));
            Literal::Nil
        }
        Stmt::Return { keyword, value } => {
            println!("ReturnStmt: {:?}", keyword);
            if let Some(expr) = value {
                let val: Literal=traverse_expr(expr,depth + 1,map,func);
                return val;
            }
            Literal::Null
        }
        Stmt::Class { name, superclass, methods } => {
            println!("ClassStmt: {:?}", name);
            if let Some(superclass_expr) = superclass {
                traverse_expr(superclass_expr,depth + 1,map,func);
            }
            for method in methods {
                traverse_stmt(method,depth + 1,map,func);
            }
            Literal::Nil
        }
    }
}

pub fn traverse_expr(expr: &Expr,depth: usize,map: &mut HashMap<String, Literal>,func: &mut HashMap<String,(Vec<Token>, Vec<Stmt>)>) -> Literal {
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
            let value: Literal = traverse_expr(value,depth+1,map,func);
            map.insert(name.lexeme().to_string(), value);
            Literal::Number(0.0)
        }
        Expr::Logical { left, operator, right } => {
            println!("LogicalExpr: {:?}", operator);
            traverse_expr(left,depth+1,map,func);
            traverse_expr(right,depth+1,map,func);
            Literal::Number(0.0)
        }
        Expr::Binary { left, operator, right } => {
            println!("BinaryExpr: {:?}", operator);
            let left_value: Literal = traverse_expr(left,depth+1,map,func);
            let right_value: Literal = traverse_expr(right,depth+1,map,func);
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
            let value: Literal = traverse_expr(right, depth + 1,map,func);
            let mut result: Literal = Literal::Number(0.0);
            let num = match value {
                Literal::Number(n) => n,
                _ => 0.0,
            };
            match operator.lexeme() {
                "-" => result = Literal::Number(-num),
                "!" => {result=Literal::Bool(!matches!(value, Literal::Bool(true)));},
                _ => println!("  Result: Unknown unary operation"),
            }
            result
        }
        Expr::Call { callee, paren, arguments } => {
            println!("CallExpr: {:?}", paren);
            if let Expr::Variable(token) = &**callee {
                // Move function params and body out of borrow scope
                let func_key = token.lexeme().to_string();
                if let Some((params, body)) = func.get(&func_key).map(|(p, b)| (p.clone(), b.clone())) {
                    println!("  Function: {}", token.lexeme());
                    if params.len() != arguments.len() {
                        println!("  Error: Expected {} arguments, got {}", params.len(), arguments.len());
                    } else {
                        let mut new_map: HashMap<String, Literal> = map.clone();
                        for (i, arg) in arguments.iter().enumerate() {
                            let param_name = &params[i].lexeme().to_string();
                            let arg_value: Literal = traverse_expr(arg, depth + 1, &mut new_map, func);
                            new_map.insert(param_name.clone(), arg_value);
                        }
                        let returnval: Literal=traverse_statements(&body, depth + 1, &mut new_map, func);
                        for (key, value) in new_map {
                            map.insert(key, value);
                        }
                        return returnval;
                    }
                } else {
                    println!("  Error: Function '{}' not found", token.lexeme());
                }
            } else {
                println!("  callee is not variable");
            }
            Literal::Number(0.0)
        }
        Expr::Get { object, name } => {
            println!("GetExpr: {:?}", name);
            traverse_expr(object,depth+1,map,func);
            Literal::Number(0.0)
        }
        Expr::Set { object, name, value } => {
            println!("SetExpr: {:?}", name);
            traverse_expr(object,depth+1,map,func);
            traverse_expr(value,depth+1,map,func);
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
            let val: Literal = traverse_expr(expr, depth + 1,map,func);
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
        lox_interpreter::Literal::Null => {
            println!("NullLiteral");
            Literal::Null
        }
    }
}