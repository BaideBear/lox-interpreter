use lox_interpreter::{lexer::Lexer, parser::Parser, Expr, Stmt,Literal};
use lox_interpreter::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use rand::{distributions::Alphanumeric, Rng};
use std::rc::Rc;
fn gen_string(length: usize) -> String {//生成随机字符串
    rand::thread_rng()
        .sample_iter(&Alphanumeric) // 包括 A-Z, a-z, 0-9
        .take(length)
        .map(char::from)
        .collect()
}

#[derive(Debug, Clone)]
pub enum Value {//值类型
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Null,
    Function {//函数
        name: Framelist,
        frame: HashMap<(String,String),Option<Rc<RefCell<Value>>>>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        obj_bind: Option<Rc<RefCell<Value>>>,
        class_def: Option<String>,
    },
    Classdef {//类定义
        name: String,
        superclass: String,
        methods: Vec<Stmt>,
    },
    Instance {//实例
        name: String,
        fields: HashMap<(String,String), Option<Rc<RefCell<Value>>>>,
    },
}

#[derive(Debug, Clone)]
pub struct Framelist{//作用域链
    pub next: Option<Box<Framelist>>,
    pub frame: String,
}

#[derive(Debug, Clone)]
pub struct Ret{//返回值
    pub exit:bool,//是否需要退出当前函数
    pub value: Option<Rc<RefCell<Value>>>,
}

pub fn traverse_statements(statements: &Vec<Stmt>,depth: usize,map: &mut HashMap<(String,String), Option<Rc<RefCell<Value>>>>,
    env: Framelist,obj :Option<Rc<RefCell<Value>>>,cur_class: Option<String>) ->Ret{//遍历多条语句
    for stmt in statements {
        let val: Ret = traverse_stmt(stmt,depth,map,env.clone(),obj.clone(),cur_class.clone());
        if val.exit {
            return val;
        }
    }
    Ret {
        exit: false,
        value: Some(Rc::new(RefCell::new(Value::Nil))),
    }
}

pub fn traverse_stmt(stmt: &Stmt,depth: usize,map: &mut HashMap<(String,String), Option<Rc<RefCell<Value>>>>,
    env: Framelist,obj :Option<Rc<RefCell<Value>>>,cur_class: Option<String>) -> Ret{ //遍历单条语句
    match stmt {
        Stmt::Expr(expr) => {//表达式语句
            traverse_expr(expr,depth+1,map,env,obj.clone(),cur_class.clone());
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::Print(expr) => {//打印语句
            let value: Option<Rc<RefCell<Value>>> = traverse_expr(expr,depth+1,map,env,obj.clone(),cur_class.clone());
            match value {
                Some(ref rc_value) => {
                    let value = rc_value.borrow();
                },
                None => println!("Error: PrintStmt requires a value"),
            }
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::Var { name, initializer } => {//变量声明语句
            if let Some(expr) = initializer {
                let value: Option<Rc<RefCell<Value>>> = traverse_expr(expr,depth+1,map,env.clone(),obj.clone(),cur_class.clone());
                map.insert((name.lexeme().to_string(), env.clone().frame), value);
            }
            else{
                map.insert((name.lexeme().to_string(), env.clone().frame), Some(Rc::new(RefCell::new(Value::Nil))));
            }
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::Block(stmts) => {//块语句
            let mut new_map: HashMap<(String,String), Option<Rc<RefCell<Value>>>> = map.clone();
            let new_env = gen_string(15);
            let new_frame = Framelist {
                next: Some(Box::new(env)),
                frame: new_env.clone(),
            };
            let mut flag: bool=false;
            for s in stmts {
                let Ret: Ret=traverse_stmt(s,depth+1,&mut new_map,new_frame.clone(),obj.clone(),cur_class.clone());
                if Ret.exit {
                    flag=true;
                    break;
                }
            }
            for (key, value) in new_map {
                map.insert(key, value);
            }
            Ret {
                exit: flag,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::If { condition, then_branch, else_branch } => {//条件语句
            let cond: Option<Rc<RefCell<Value>>> = traverse_expr(condition, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            if let Some(ref rc_cond) = cond {
                let cond_value = rc_cond.borrow();
                if let Value::Bool(true) = &*cond_value {
                    let ret: Ret = traverse_stmt(then_branch, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
                    if ret.exit {
                        return ret;
                    }
                } else {
                    if let Some(else_branch) = else_branch {
                        let ret: Ret = traverse_stmt(else_branch, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
                        if ret.exit {
                            return ret;
                        }
                    }
                }
            }
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::While { condition, body } => {//while循环语句
            loop {
                let cond: Option<Rc<RefCell<Value>>> = traverse_expr(condition, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
                if let Some(ref rc_cond) = cond {
                    let cond_value = rc_cond.borrow();
                    if let Value::Bool(false) = &*cond_value {
                        break;
                    }
                }
                let ret: Ret = traverse_stmt(body, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
                if ret.exit {
                    return ret;
                }
            }
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::For { initializer, condition, increment, body } => {//for循环语句
            let mut new_map: HashMap<(String,String), Option<Rc<RefCell<Value>>>> = map.clone();
            if let Some(init) = initializer {
                traverse_stmt(init,depth + 1,&mut new_map,env.clone(), obj.clone(), cur_class.clone());
            }
            loop{
                if let Some(cond) = condition {
                    let cond = traverse_expr(cond,depth + 1,&mut new_map,env.clone(), obj.clone(), cur_class.clone());
                    if let Some(ref rc_cond) = cond {
                        let cond_value = rc_cond.borrow();
                        if let Value::Bool(false) = &*cond_value {
                            break;
                        }
                    }
                }
                let ret: Ret = traverse_stmt(body, depth + 1, &mut new_map, env.clone(), obj.clone(), cur_class.clone());
                if ret.exit {
                    return ret;
                }
                if let Some(inc) = increment {
                    traverse_expr(inc, depth + 1, &mut new_map, env.clone(), obj.clone(), cur_class.clone());
                }
            }
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::Function { name, params, body } => {//函数声明语句
            let func: Value = Value::Function {
                frame: map.clone(),
                params: params.clone(),
                body: body.clone(),
                name: env.clone(),
                obj_bind: obj.clone(),
                class_def: cur_class.clone(),
            };
            map.insert((name.lexeme().to_string(), env.frame.clone()), Some(Rc::new(RefCell::new(func.clone()))));
            if let Some(obj_ref) = obj {
                if let Value::Instance { name: instance_name, fields } = &mut *obj_ref.borrow_mut() {
                    if let Some(cur_class_name) = cur_class {
                        fields.insert((name.lexeme().to_string(), cur_class_name.clone()), Some(Rc::new(RefCell::new(func))));
                    } else {
                        println!("Error: Current class name is None");
                    }
                }
            }
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
        Stmt::Return { keyword, value } => {//返回语句
            if let Some(expr) = value {
                let val: Option<Rc<RefCell<Value>>> = traverse_expr(expr,depth + 1,map,env.clone(),obj.clone(),cur_class.clone());
                return Ret {
                    exit: true,
                    value: val,
                };
            }
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Null))),
            }
        }
        Stmt::Class { name, superclass, methods } => {//类声明语句
            let newclass: Value = Value::Classdef {
                name: name.lexeme().to_string(),
                superclass: if let Some(superclass_expr) = superclass {
                    if let Expr::Variable(token) = superclass_expr {
                        token.lexeme().to_string()
                    } else {
                        println!("Error: Superclass must be a variable");
                        String::new()
                    }
                } else {
                    String::new()
                },
                methods: methods.clone(),
            };
            map.insert((name.lexeme().to_string(), env.frame.clone()), Some(Rc::new(RefCell::new(newclass))));
            Ret {
                exit: false,
                value: Some(Rc::new(RefCell::new(Value::Nil))),
            }
        }
    }
}

pub fn traverse_expr(expr: &Expr,depth: usize,map: &mut HashMap<(String,String), Option<Rc<RefCell<Value>>>>,env: Framelist,
    obj :Option<Rc<RefCell<Value>>>,cur_class: Option<String>) -> Option<Rc<RefCell<Value>>> {
    match expr {
        Expr::Literal(literal) => {//字面量表达式
            let val: Value = traverse_literal(literal, depth + 1);
            Some(Rc::new(RefCell::new(val)))
        }
        Expr::Variable(token) => {//变量表达式
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
            Some(Rc::new(RefCell::new(Value::Nil))) // Return Nil if variable not found
        }
        Expr::Assign { name, value } => {//赋值表达式
            let value: Option<Rc<RefCell<Value>>> = traverse_expr(value,depth+1,map,env.clone(),obj.clone(),cur_class.clone());
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
            Some(Rc::new(RefCell::new(Value::Number(0.0)))) // Return Nil after assignment
        }
        Expr::Logical { left, operator, right } => {//逻辑表达式
            let left_value: Option<Rc<RefCell<Value>>> = traverse_expr(left, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            let right_value: Option<Rc<RefCell<Value>>> = traverse_expr(right, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            let mut result: Option<Rc<RefCell<Value>>> = None;
            match operator.lexeme() {
                "or" => {
                    if let Some(ref rc_left) = left_value {
                        let left_value = rc_left.borrow();
                        if let Value::Bool(true) = &*left_value {
                            result = Some(Rc::new(RefCell::new(left_value.clone())));
                        }
                    }
                    if result.is_none() {
                        result = right_value;
                    }
                }
                "and" => {
                    if let Some(ref rc_left) = left_value {
                        let left_value = rc_left.borrow();
                        if let Value::Bool(false) = &*left_value {
                            result = Some(Rc::new(RefCell::new(left_value.clone())));
                        } else {
                            result = right_value;
                        }
                    }
                }
                _ => println!("  Result: Unknown logical operation"),
            }
            result
        }
        Expr::Binary { left, operator, right } => {//二元运算表达式
            let left_value: Option<Rc<RefCell<Value>>> = traverse_expr(left,depth+1,map,env.clone(),obj.clone(),cur_class.clone());
            let right_value: Option<Rc<RefCell<Value>>> = traverse_expr(right,depth+1,map,env.clone(),obj.clone(),cur_class.clone());
            let mut result: Option<Rc<RefCell<Value>>> = Some(Rc::new(RefCell::new(Value::Number(0.0))));
            let mut isnumber: bool = false;
            let left_num = match left_value {
                Some(ref rc_left) => {
                    let left_value = rc_left.borrow();
                    match &*left_value {
                        Value::Number(num) => {
                            isnumber = true;
                            *num
                        },
                        _ => 0.0,
                    }
                },
                None => 0.0,
            };
            let right_num = match right_value {
                Some(ref rc_right) => {
                    let right_value = rc_right.borrow();
                    match &*right_value {
                        Value::Number(num) => {
                            isnumber = true;
                            *num
                        },
                        _ => 0.0,
                    }
                },
                None => 0.0,
            };
            let left_string = match left_value {
                Some(ref rc_left) => {
                    let left_value = rc_left.borrow();
                    match &*left_value {
                        Value::String(s) => s.clone(),
                        _ => String::new(),
                    }
                },
                None => String::new(),
            };
            let right_string = match right_value {
                Some(ref rc_right) => {
                    let right_value = rc_right.borrow();
                    match &*right_value {
                        Value::String(s) => s.clone(),
                        _ => String::new(),
                    }
                },
                None => String::new(),
            };
            match operator.lexeme() {
                "+" => {
                    if isnumber==true{
                        result = Some(Rc::new(RefCell::new(Value::Number(left_num + right_num))));
                    } else {
                        result = Some(Rc::new(RefCell::new(Value::String(format!("{}{}", left_string, right_string)))));
                    }
                }
                "-" => result = Some(Rc::new(RefCell::new(Value::Number(left_num - right_num)))),
                "*" => result = Some(Rc::new(RefCell::new(Value::Number(left_num * right_num)))),
                "/" => {
                    if right_num != 0.0 {
                        result = Some(Rc::new(RefCell::new(Value::Number(left_num / right_num))));
                    } else {
                        println!("  Error: Division by zero");
                    }
                }
                ">" => {
                    result = Some(Rc::new(RefCell::new(Value::Bool(left_num > right_num))));
                }
                "<" => {
                    result = Some(Rc::new(RefCell::new(Value::Bool(left_num < right_num))));
                }
                ">=" => {
                    result = Some(Rc::new(RefCell::new(Value::Bool(left_num >= right_num))));
                }
                "<=" => {
                    result = Some(Rc::new(RefCell::new(Value::Bool(left_num <= right_num))));
                }
                "==" => {
                    result=match (left_value, right_value) {
                        (Some(ref rc_left), Some(ref rc_right)) => {
                            let left_value = rc_left.borrow();
                            let right_value = rc_right.borrow();
                            match (&*left_value, &*right_value) {
                                (Value::Number(l), Value::Number(r)) => Some(Rc::new(RefCell::new(Value::Bool(l == r)))),
                                (Value::String(l), Value::String(r)) => Some(Rc::new(RefCell::new(Value::Bool(l == r)))),
                                (Value::Bool(l), Value::Bool(r)) => Some(Rc::new(RefCell::new(Value::Bool(l == r)))),
                                (Value::Nil, Value::Nil) => Some(Rc::new(RefCell::new(Value::Bool(true)))),
                                _ => Some(Rc::new(RefCell::new(Value::Bool(false)))),
                            }
                        },
                        _ => Some(Rc::new(RefCell::new(Value::Bool(false)))),
                    };
                }
                "!=" => {
                    result=match (left_value, right_value) {
                        (Some(ref rc_left), Some(ref rc_right)) => {
                            let left_value = rc_left.borrow();
                            let right_value = rc_right.borrow();
                            match (&*left_value, &*right_value) {
                                (Value::Number(l), Value::Number(r)) => Some(Rc::new(RefCell::new(Value::Bool(l != r)))),
                                (Value::String(l), Value::String(r)) => Some(Rc::new(RefCell::new(Value::Bool(l != r)))),
                                (Value::Bool(l), Value::Bool(r)) => Some(Rc::new(RefCell::new(Value::Bool(l != r)))),
                                (Value::Nil, Value::Nil) => Some(Rc::new(RefCell::new(Value::Bool(false)))),
                                _ => Some(Rc::new(RefCell::new(Value::Bool(true)))),
                            }
                        },
                        _ => Some(Rc::new(RefCell::new(Value::Bool(true)))),
                    };
                }
                _ => println!("  Result: Unknown operation"),
            };
            result
        }
        Expr::Unary { operator, right } => {//一元运算表达式
            let value: Option<Rc<RefCell<Value>>> = traverse_expr(right, depth + 1,map,env.clone(),obj.clone(),cur_class.clone());
            let mut result: Option<Rc<RefCell<Value>>> = Some(Rc::new(RefCell::new(Value::Number(0.0))));
            let num = match value {
                Some(ref rc_value) => {
                    let value = rc_value.borrow();
                    match &*value {
                        Value::Number(n) => *n,
                        Value::Bool(b) => if *b { 1.0 } else { 0.0 },
                        _ => 0.0, // Default to 0.0 for other types
                    }
                },
                None => 0.0,
            };
            match operator.lexeme() {
                "-" => result = Some(Rc::new(RefCell::new(Value::Number(-num)))),
                "!" => {
                    if num==1.0 {
                        result = Some(Rc::new(RefCell::new(Value::Bool(false))));
                    } else {
                        result = Some(Rc::new(RefCell::new(Value::Bool(true))));
                    }
                },
                _ => println!("  Result: Unknown unary operation"),
            }
            result
        }
        Expr::Call { callee, paren, arguments } => {//调用表达式
            let func: Option<Rc<RefCell<Value>>> = traverse_expr(callee, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            let mut args: Vec<Value> = Vec::new();
            for arg in arguments {
                let value = traverse_expr(arg, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
                if let Some(ref rc_value) = value {
                    args.push(rc_value.borrow().clone());
                } else {
                    args.push(Value::Nil);
                }
            }
            match func {
                Some(ref rc_func) => {
                    let func = rc_func.borrow();
                    match &*func {
                        Value::Function { frame, params, body, name ,obj_bind,class_def} => {//函数调用
                            // Create a new environment for the function call
                            let mut call_frame = frame.clone();
                            for(k,v) in map.iter() {
                                call_frame.insert((k.0.clone(), k.1.clone()), v.clone());
                            }
                            let new_env = gen_string(15);
                            for (param, arg) in params.iter().zip(args) {
                                call_frame.insert((param.lexeme().to_string(), new_env.clone()), Some(Rc::new(RefCell::new(arg))));
                            }
                            let new_frame = Framelist {
                                next: Some(Box::new(name.clone())),
                                frame: new_env.clone(),
                            };
                            let retval: Ret= traverse_statements(&body, depth + 1, &mut call_frame, new_frame,obj_bind.clone(),class_def.clone());
                            for(k,v) in call_frame.iter() {
                                map.insert((k.0.clone(), k.1.clone()), v.clone());
                            }
                            return retval.value;
                        }
                        Value::Classdef { name, superclass, methods } => {//类调用
                            let new_field: HashMap<(String,String), Option<Rc<RefCell<Value>>>>= HashMap::new();
                            let instance_name = name.clone();
                            let instance = Value::Instance {
                                name: instance_name.clone(),
                                fields: new_field,
                            };
                            let new_instance: Option<Rc<RefCell<Value>>> = Some(Rc::new(RefCell::new(instance.clone())));
                            let mut cur_name=name.clone();
                            loop {//load all methods from class and superclass
                                let mut methods_to_traverse=Vec::new();
                                let mut next_class=String::new();
                                {
                                    let mut cur_class = map.get(&(cur_name.clone(), env.frame.clone()));
                                    let mut cur_env: Framelist = env.clone();
                                    loop {
                                        if let Some(value) = map.get(&(cur_name.clone(), cur_env.frame.clone())) {
                                            cur_class = map.get(&(cur_name.clone(), cur_env.frame.clone()));
                                        }
                                        match &cur_env.next {
                                            Some(next) => cur_env = (**next).clone(),
                                            None => break, // No more environments to check
                                        }
                                    }
                                    if let Some(valueref)=cur_class{
                                        if let Some(ref methods) = valueref{
                                            let class_= methods.borrow();
                                            match &*class_ {
                                                Value::Classdef { name: _, superclass: next, methods: m } => {
                                                    next_class = next.clone();
                                                    methods_to_traverse = m.clone();
                                                }
                                                _ => {
                                                    println!("Error: Expected a class definition");
                                                    next_class = String::new();
                                                    methods_to_traverse = Vec::new();
                                                }
                                            }
                                        }
                                    }
                                }
                                traverse_statements(&methods_to_traverse, depth + 1, map, env.clone(), new_instance.clone(), Some(cur_name.clone()));
                                if next_class.is_empty() {
                                    break;
                                } else {
                                    cur_name = next_class;
                                }
                            }
                            //call init method if exists
                            cur_name=name.clone();
                            loop{
                                let mut flag:bool = false;
                                let mut frame_func:HashMap<(String,String), Option<Rc<RefCell<Value>>>> = HashMap::new();
                                let mut func_env_tmp:Framelist = Framelist { next: None, frame: env.frame.clone() };
                                let mut body_func:Vec<Stmt> = Vec::new();
                                let mut param_func: Vec<Token> = Vec::new();
                                if let Some(ref rc_inst)=new_instance{
                                    let inst = rc_inst.borrow();
                                    match &*inst {
                                        Value::Instance { name, fields } => {
                                            // Call the init method if it exists
                                            let init: String="init".to_string();
                                            if let Some(init_method) = fields.get(&(init, cur_name.clone())) {//if init method exists
                                                if let Some(ref rc_func)=init_method{
                                                    let final_func= rc_func.borrow();
                                                    if let Value::Function { frame, params, body, name: func_env, obj_bind: _ , class_def: _ } = &*final_func {
                                                        // Create a new environment for the init call
                                                        frame_func = frame.clone();
                                                        func_env_tmp=func_env.clone();
                                                        body_func = body.clone();
                                                        param_func = params.clone();
                                                        flag = true;
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                if flag {
                                    let mut call_frame = frame_func.clone();
                                    for(k,v) in map.iter() {
                                        call_frame.insert((k.0.clone(), k.1.clone()), v.clone());
                                    }
                                    let new_env = gen_string(15);
                                    for (param, arg) in param_func.iter().zip(args) {
                                        call_frame.insert((param.lexeme().to_string(), new_env.clone()), Some(Rc::new(RefCell::new(arg))));
                                    }
                                    let new_frame = Framelist {
                                        next: Some(Box::new(func_env_tmp.clone())),
                                        frame: new_env,
                                    };
                                    traverse_statements(&body_func, depth + 1, &mut call_frame, new_frame, new_instance.clone(), Some(cur_name.clone()));
                                    for(k,v) in call_frame.iter() {
                                        map.insert((k.0.clone(), k.1.clone()), v.clone());
                                    }
                                    return new_instance;
                                }
                                let mut next_class = String::new();
                                {
                                    let mut cur_class = map.get(&(cur_name.clone(), env.frame.clone()));
                                    let mut cur_env: Framelist = env.clone();
                                    loop {
                                        if let Some(value) = map.get(&(cur_name.clone(), cur_env.frame.clone())) {
                                            cur_class = map.get(&(cur_name.clone(), cur_env.frame.clone()));
                                        }
                                        match &cur_env.next {
                                            Some(next) => cur_env = (**next).clone(),
                                            None => break, // No more environments to check
                                        }
                                    }
                                    if let Some(valueref)=cur_class{
                                        if let Some(ref methods) = valueref{
                                            let class_= methods.borrow();
                                            match &*class_ {
                                                Value::Classdef { name: _, superclass: next, methods: m } => {
                                                    next_class = next.clone();
                                                }
                                                _ => {
                                                    println!("Error: Expected a class definition");
                                                    next_class = String::new();
                                                }
                                            }
                                        }
                                    } else {
                                        next_class = String::new();
                                    }
                                }
                                if next_class.is_empty() {
                                    break;
                                } else {
                                    cur_name = next_class;
                                }
                            }
                            return new_instance;
                        }
                        _ => {}
                    }
                    Option::Some(Rc::new(RefCell::new(Value::Nil))) // Return Nil if not a function or class
                }
                _ => Option::Some(Rc::new(RefCell::new(Value::Nil)))
            }
        }
        Expr::This(token) => {//this表达式
            obj
        }
        Expr::Get { object, name } => {//属性访问表达式
            let obj_value: Option<Rc<RefCell<Value>>> = traverse_expr(&object, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            match obj_value {
                Some(rc_value) => {
                    let value = rc_value.borrow();
                    if let Value::Instance { name: instance_name, fields } = &*value {
                        let mut cur_name=instance_name.clone();
                        loop {
                            if let Some(field_value) = fields.get(&(name.lexeme().to_string(), cur_name.clone())) {
                                return field_value.clone();
                            }
                            let mut next_class = String::new();
                            {
                                let cur_class = map.get(&(cur_name.clone(), env.frame.clone()));
                                if let Some(valueref)=cur_class{
                                    if let Some(ref methods) = valueref{
                                        let class_= methods.borrow();
                                        match &*class_ {
                                            Value::Classdef { name: _, superclass: next, methods: _ } => {
                                                next_class = next.clone();
                                            }
                                            _ => {
                                                println!("Error: Expected a class definition");
                                                next_class = String::new();
                                            }
                                        }
                                    }
                                } else {
                                    next_class = String::new();
                                }
                            }
                            if next_class.is_empty() {
                                break;
                            } else {
                                cur_name = next_class;
                            }
                        }
                    }
                }
                None => println!("Error: GetExpr can only be used on instances"),
            }
            Some(Rc::new(RefCell::new(Value::Nil))) // Return Nil if property not found
        }
        Expr::Set { object, name, value } => {//属性设置表达式
            let obj_value: Option<Rc<RefCell<Value>>> = traverse_expr(&object, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            let new_value: Option<Rc<RefCell<Value>>> = traverse_expr(&value, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            //let obj_value_clone= obj_value.clone();
            if let Some(rc_obj) = obj_value {
                let mut obj_borrow = rc_obj.borrow_mut();
                if let Value::Instance { name: instance_name, ref mut fields } = &mut *obj_borrow {
                    fields.insert((name.lexeme().to_string(), instance_name.clone()), new_value.clone());
                } else {
                    println!("Error: SetExpr can only be used on instances");
                }
            }
            Some(Rc::new(RefCell::new(Value::Nil))) // Return Nil after setting the property
        }
        Expr::Super { keyword, method } => {//super表达式
            let mut cur_env: Framelist = env.clone();
            let mut current_class=map.get(&(cur_class.clone().unwrap_or_default(), env.frame.clone()));
            loop {
                if let Some(value) = map.get(&(cur_class.clone().unwrap_or_default(), cur_env.frame.clone())) {
                    current_class=map.get(&(cur_class.clone().unwrap_or_default(), cur_env.frame.clone()));
                }
                match &cur_env.next {
                    Some(next) => cur_env = (**next).clone(),
                    None => break, // No more environments to check
                }
            }
            if let Some(current_class_ref) = current_class {
                if let Some(ref rc_current_class) = current_class_ref {
                    let current_class_value = rc_current_class.borrow();
                    if let Value::Classdef { name: class_name, superclass: super_name, methods } = &*current_class_value {
                        let mut cur_name=super_name.clone();
                        loop{
                            if cur_name.is_empty() {
                                break;
                            }
                            match obj{
                                Some(ref rc_obj) => {
                                    let obj_borrow = rc_obj.borrow();
                                    if let Value::Instance { name: instance_name, fields } = &*obj_borrow {
                                        if let Some(value) = fields.get(&(method.lexeme().to_string(), cur_name.clone())) {
                                            return value.clone();
                                        }
                                    }
                                }
                                None => {}
                            }
                            let mut next_class = String::new();
                            {
                                let cur_class = map.get(&(cur_name.clone(), env.frame.clone()));
                                if let Some(valueref)=cur_class{
                                    if let Some(ref methods) = valueref{
                                        let class_= methods.borrow();
                                        match &*class_ {
                                            Value::Classdef { name: _, superclass: next, methods: m } => {
                                                next_class = next.clone();
                                            }
                                            _ => {
                                                println!("Error: Expected a class definition");
                                                next_class = String::new();
                                            }
                                        }
                                    }
                                } else {
                                    next_class = String::new();
                                }
                            }
                            cur_name = next_class;
                        }
                    }
                }
            }
            Some(Rc::new(RefCell::new(Value::Nil)))
        }
        Expr::Grouping(expr) => {//分组表达式（括号内表达式）
            let val: Option<Rc<RefCell<Value>>> = traverse_expr(&expr, depth + 1, map, env.clone(), obj.clone(), cur_class.clone());
            val
        }
    }
}

pub fn traverse_literal(literal: &Literal, depth: usize) -> Value{//获取字面量的值
    match literal {
        lox_interpreter::Literal::Number(value) => {
            Value::Number(value.clone())
        }
        lox_interpreter::Literal::String(value) => {
            Value::String(value.clone())
        }
        lox_interpreter::Literal::Bool(value) => {
            Value::Bool(*value)
        }
        lox_interpreter::Literal::Nil => {
            Value::Nil
        }
    }
}