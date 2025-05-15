# lox语言解释器----rust实现

```bash
├── Cargo.lock
├── Cargo.toml
├── README
├── src
│   ├── ast.rs #ast树定义
│   ├── lexer.rs #词法分析实现
│   ├── lib.rs 
│   ├── main.rs #入口
│   ├── parser.rs #语法分析实现
│   └── token.rs #token目录
└── testcase #测试样例
    ├── 1.lox
    ├── 10.lox
    ├── 11.lox
    ├── 12.lox
```

目前的进度和问题：

1. 词法分析：实现完成
2. 语法分析：ast树解析完成，存在问题如下：
    - break， continue没解析，会被解析成变量名（实际上lox的语法要求中没有break，continue，但是实验要求中有提及，解析的时候要注意判断变量名是不是break或者continue，要特殊处理一下，我的ast和lox的官方定义一致，没有特殊处理这两个token）
    - 错误处理的一些部分没完成，有一些错误不适合在语法分析中实现,实现情况如下：

| 错误类型                       | 示例代码                        | 错误信息                                                         |
|-------------------------------|--------------------------------|------------------------------------------------------------------|
| 缺失分号（done）                       | `print 1`                      | `Error at 'EOF': Expect ';' after value.`                         |
| 函数声明缺少右括号 （done）            | `fun f(a, b {}`                | `Error at '{': Expect ')' after parameters.`                     |
| 缺失变量名（done）                     | `var = 5;`                     | `Error at '=': Expect variable name.`                            |
| 非法 token（如多余字符）（done）      | `@abc`                         | `Error at '@': Unexpected character.`                            |
| 块语句缺失右大括号（done）             | `{ var x = 1;`                 | `Error at 'EOF': Expect '}' after block.`                        |
| return 在顶层出现             | `return 5;`                    | `Error: Can't return from top-level code.`                       |
| break 或 continue 在非循环中使用| `break;`                       | `Error: Can't use 'break' outside of a loop.`                    |
| 重复变量定义（同一作用域）    | `{ var a = 1; var a = 2; }`    | `Error: Already a variable with this name in this scope.`        |
| 类继承非法父类                 | `class A < 123 {}`             | `Error: Superclass must be a class.`                             |
| super 用在非子类中             | `super.method();`              | `Error: Can't use 'super' in a class with no superclass.`        |
| this 用在类外部                | `print this;`                  | `Error: Can't use 'this' outside of a class.`                    |

> return、break、this等标志的内容判断在语法分析中实现会相当麻烦