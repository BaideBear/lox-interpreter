use lox_interpreter::{lexer::Lexer, parser::Parser, Expr, Stmt,Literal};
use lox_interpreter::token::Token;
use std::collections::HashMap;
use rand::{distributions::Alphanumeric, Rng};
fn gen_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric) // 包括 A-Z, a-z, 0-9
        .take(length)
        .map(char::from)
        .collect()
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Null,
    Function {
        name: Framelist,
        frame: HashMap<(String,String),Value>,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Framelist{
    pub next: Option<Box<Framelist>>,
    pub frame: String,
}

pub fn traverse_statements(statements: &Vec<Stmt>,depth: usize,map: &mut HashMap<(String,String), Value>,env: Framelist) ->Value{
    for stmt in statements {
        let val: Value=traverse_stmt(stmt,depth,map,env.clone());
        match val {
            Value::Number(_) => return val,
            Value::String(_) => return val,
            Value::Bool(_) => return val,
            Value::Null => return val,
            Value::Function { frame, params, body, name } => {
                return Value::Function {
                    name,
                    frame,
                    params,
                    body,
                };
            }
            _ => continue,
        }
    }
    return Value::Nil
}

pub fn traverse_stmt(stmt: &Stmt,depth: usize,map: &mut HashMap<(String,String), Value>,env: Framelist) -> Value{
    for _ in 0..depth {
        print!("  ");
    }
    match stmt {
        Stmt::Expr(expr) => {
            println!("ExpressionStmt");
            traverse_expr(expr,depth+1,map,env);
            Value::Nil
        }
        Stmt::Print(expr) => {
            println!("PrintStmt");
            let value: Value = traverse_expr(expr,depth+1,map,env);
            match value {
                Value::Number(num) => println!("{}", num),
                Value::String(s) => println!("{}", s),
                Value::Bool(b) => println!("{}", b),
                Value::Nil => println!("nil"),
                _ => println!("Unknown value"),
            }
            Value::Nil
        }
        Stmt::Var { name, initializer } => {
            println!("VarStmt: {:?}", name);
            if let Some(expr) = initializer {
                let value: Value = traverse_expr(expr,depth+1,map,env.clone());
                map.insert((name.lexeme().to_string(), env.clone().frame), value);
            }
            else{
                map.insert((name.lexeme().to_string(), env.clone().frame), Value::Nil);
            }
            Value::Nil
        }
        Stmt::Block(stmts) => {
            println!("BlockStmt");
            let mut new_map: HashMap<(String,String), Value> = map.clone();
            let new_env = gen_string(15);
            let new_frame = Framelist {
                next: Some(Box::new(env)),
                frame: new_env.clone(),
            };
            for s in stmts {
                traverse_stmt(s,depth+1,&mut new_map,new_frame.clone());
            }
            for (key, value) in new_map {
                map.insert(key, value);
            }
            Value::Nil
        }
        Stmt::If { condition, then_branch, else_branch } => {
            println!("IfStmt");
            let cond: Value = traverse_expr(condition, depth + 1, map, env.clone());
            if let Value::Bool(true) = cond {
                traverse_stmt(then_branch, depth + 1, map, env.clone());
            } else {
                if let Some(else_branch) = else_branch {
                    traverse_stmt(else_branch, depth + 1, map, env.clone());
                }
            }
            Value::Nil
        }
        Stmt::While { condition, body } => {
            println!("WhileStmt");
            loop{
                let cond: Value = traverse_expr(condition, depth + 1, map, env.clone());
                if let Value::Bool(false) = cond {
                    break;
                }
                traverse_stmt(body, depth + 1, map, env.clone());
            }
            Value::Nil
        }
        Stmt::For { initializer, condition, increment, body } => {
            println!("ForStmt");
            let mut new_map: HashMap<(String,String), Value> = map.clone();
            if let Some(init) = initializer {
                traverse_stmt(init,depth + 1,&mut new_map,env.clone());
            }
            loop{
                if let Some(cond) = condition {
                    let cond = traverse_expr(cond,depth + 1,&mut new_map,env.clone());
                    if let Value::Bool(false) = cond {
                        break;
                    }
                }
                traverse_stmt(body,depth + 1,&mut new_map,env.clone());
                if let Some(inc) = increment {
                    traverse_expr(inc,depth + 1,&mut new_map,env.clone());
                }
            }
            Value::Nil
        }
        Stmt::Function { name, params, body } => {
            println!("FunctionStmt: {:?}", name);
            for param in params {
                println!("  Param: {:?}", param);
            }
            let func: Value = Value::Function {
                frame: map.clone(),
                params: params.clone(),
                body: body.clone(),
                name: env.clone(),
            };
            map.insert((name.lexeme().to_string(), env.frame.clone()), func);
            Value::Nil
        }
        Stmt::Return { keyword, value } => {
            println!("ReturnStmt: {:?}", keyword);
            if let Some(expr) = value {
                let val: Value = traverse_expr(expr,depth + 1,map,env.clone());
                if let Value::Function { .. } = val {
                    println!("val is a function, returning it directly");
                }
                return val;
            }
            Value::Null
        }
        Stmt::Class { name, superclass, methods } => {
            println!("ClassStmt: {:?}", name);
            if let Some(superclass_expr) = superclass {
                traverse_expr(superclass_expr,depth + 1,map,env.clone());
            }
            for method in methods {
                traverse_stmt(method,depth + 1,map,env.clone());
            }
            Value::Nil
        }
    }
}

pub fn traverse_expr(expr: &Expr,depth: usize,map: &mut HashMap<(String,String), Value>,env: Framelist) -> Value {
    for _ in 0..depth {
        print!("  ");
    }
    match expr {
        Expr::Literal(literal) => {
            println!("LiteralExpr: {:?}", literal);
            let val: Value = traverse_literal(literal, depth + 1);
            val
        }
        Expr::Variable(token) => {
            println!("VariableExpr: {:?}", token);
            let mut cur_env: Framelist = env.clone();
            loop {
                if let Some(value) = map.get(&(token.lexeme().to_string(), cur_env.frame.clone())) {
                    return value.clone();
                }
                match &cur_env.next {
                    Some(next) => cur_env = (**next).clone(),
                    None => break, // No more environments to check
                }
            }
            Value::Nil
        }
        Expr::Assign { name, value } => {
            println!("AssignExpr: {:?}", name);
            let value: Value = traverse_expr(value,depth+1,map,env.clone());
            let mut cur_env: Framelist = env.clone();
            let old_frame:Framelist;
            loop {
                if let Some(value_local) = map.get(&(name.lexeme().to_string(), cur_env.frame.clone())) {
                    old_frame = cur_env.clone();
                    map.insert((name.lexeme().to_string(), old_frame.frame), value);
                    break;
                }
                match &cur_env.next {
                    Some(next) => cur_env = (**next).clone(),
                    None => break,
                }
            }
            Value::Number(0.0)
        }
        Expr::Logical { left, operator, right } => {
            println!("LogicalExpr: {:?}", operator);
            traverse_expr(left,depth+1,map,env.clone());
            traverse_expr(right,depth+1,map,env.clone());
            Value::Number(0.0)
        }
        Expr::Binary { left, operator, right } => {
            println!("BinaryExpr: {:?}", operator);
            let left_value: Value = traverse_expr(left,depth+1,map,env.clone());
            let right_value: Value = traverse_expr(right,depth+1,map,env.clone());
            let mut result: Value = Value::Number(0.0);
            let left_num = match left_value {
                Value::Number(num) => num,
                _ => 0.0,
            };
            let right_num = match right_value {
                Value::Number(num) => num,
                _ => 0.0,
            };
            match operator.lexeme() {
                "+" => result = Value::Number(left_num + right_num),
                "-" => result = Value::Number(left_num - right_num),
                "*" => result = Value::Number(left_num * right_num),
                "/" => {
                    if right_num != 0.0 {
                        result = Value::Number(left_num / right_num);
                    } else {
                        println!("  Error: Division by zero");
                    }
                }
                ">" => {
                    result = Value::Bool(left_num > right_num);
                }
                "<" => {
                    result = Value::Bool(left_num < right_num);
                }
                ">=" => {
                    result = Value::Bool(left_num >= right_num);
                }
                "<=" => {
                    result = Value::Bool(left_num <= right_num);
                }
                "==" => {
                    result=match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Value::Bool(l == r),
                        (Value::String(l), Value::String(r)) => Value::Bool(l == r),
                        (Value::Bool(l), Value::Bool(r)) => Value::Bool(l == r),
                        (Value::Nil, Value::Nil) => Value::Bool(true),
                        _ => Value::Bool(false),
                    };
                }
                "!=" => {
                    result=match (left_value, right_value) {
                        (Value::Number(l), Value::Number(r)) => Value::Bool(l != r),
                        (Value::String(l), Value::String(r)) => Value::Bool(l != r),
                        (Value::Bool(l), Value::Bool(r)) => Value::Bool(l != r),
                        (Value::Nil, Value::Nil) => Value::Bool(false),
                        _ => Value::Bool(true),
                    };
                }
                _ => println!("  Result: Unknown operation"),
            }
            result
        }
        Expr::Unary { operator, right } => {
            println!("UnaryExpr: {:?}", operator);
            let value: Value = traverse_expr(right, depth + 1,map,env.clone());
            let mut result: Value = Value::Number(0.0);
            let num = match value {
                Value::Number(n) => n,
                _ => 0.0,
            };
            match operator.lexeme() {
                "-" => result = Value::Number(-num),
                "!" => {result=Value::Bool(!matches!(value, Value::Bool(true)));},
                _ => println!("  Result: Unknown unary operation"),
            }
            result
        }
        Expr::Call { callee, paren, arguments } => {
            println!("CallExpr: {:?}", paren);
            let func: Value= traverse_expr(callee, depth + 1, map, env.clone());
            let mut args: Vec<Value> = Vec::new();
            for arg in arguments {
                let value = traverse_expr(arg, depth + 1, map, env.clone());
                args.push(value);
            }
            match func {
                Value::Function { frame, params, body, name } => {
                    // Create a new environment for the function call
                    let mut call_frame = frame.clone();
                    for(k,v) in map.iter() {
                        call_frame.insert((k.0.clone(), k.1.clone()), v.clone());
                    }
                    let new_env = gen_string(15);
                    for (param, arg) in params.iter().zip(args) {
                        call_frame.insert((param.lexeme().to_string(), new_env.clone()), arg);
                    }
                    let new_frame = Framelist {
                        next: Some(Box::new(name)),
                        frame: new_env.clone(),
                    };
                    // Execute the function body in the new environment
                    //println!("  Calling function: {} with new environment: {} old env: {}", name, new_env, env);
                    let retval: Value= traverse_statements(&body, depth + 1, &mut call_frame, new_frame);
                    for(k,v) in call_frame.iter() {
                        map.insert((k.0.clone(), k.1.clone()), v.clone());
                    }
                    return retval;
                }
                _ => println!("  Error: Attempted to call a non-function"),
            }
            Value::Number(0.0)
        }
        Expr::Get { object, name } => {
            println!("GetExpr: {:?}", name);
            traverse_expr(object,depth+1,map,env.clone());
            Value::Number(0.0)
        }
        Expr::Set { object, name, value } => {
            println!("SetExpr: {:?}", name);
            traverse_expr(object,depth+1,map,env.clone());
            traverse_expr(value,depth+1,map,env.clone());
            Value::Number(0.0)
        }
        Expr::This(token) => {
            println!("ThisExpr: {:?}", token);
            Value::Number(0.0)
        }
        Expr::Super { keyword, method } => {
            println!("SuperExpr: {:?} {:?}", keyword, method);
            Value::Number(0.0)
        }
        Expr::Grouping(expr) => {
            println!("GroupingExpr");
            let val: Value = traverse_expr(expr, depth + 1,map,env.clone());
            val
        }
    }
}

pub fn traverse_literal(literal: &Literal, depth: usize) -> Value{
    for _ in 0..depth {
        print!("  ");
    }
    match literal {
        lox_interpreter::Literal::Number(value) => {
            println!("NumberLiteral: {}", value);
            Value::Number(value.clone())
        }
        lox_interpreter::Literal::String(value) => {
            println!("StringLiteral: {}", value);
            Value::String(value.clone())
        }
        lox_interpreter::Literal::Bool(value) => {
            println!("BooleanLiteral: {}", value);
            Value::Bool(*value)
        }
        lox_interpreter::Literal::Nil => {
            println!("NilLiteral");
            Value::Nil
        }
    }
}