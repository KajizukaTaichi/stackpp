use clap::Parser;
use rustyline::DefaultEditor;
use std::{collections::HashMap, fs::read_to_string};

const VERSION: &str = "0.2.0";

#[derive(Parser, Debug)]
#[command(
    name = "Stack++",
    version = VERSION,
    author = "梶塚太智 <kajizukataichi@outlook.jp>",
    about = "A improved Stack machine programming language",
)]
struct Cli {
    /// Run the script file
    #[arg(index = 1)]
    file: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let mut rl = DefaultEditor::new().unwrap();
    let mut stackpp = Core {
        stack: vec![],
        memory: HashMap::new(),
    };

    if let Some(path) = cli.file {
        if let Ok(code) = read_to_string(path) {
            stackpp.eval(Core::parse(code));
        } else {
            eprintln!("Error! it fault to open the file");
        }
    } else {
        println!("Stack++");
        loop {
            let mut code = String::new();
            loop {
                let enter = rl.readline("> ").unwrap();
                code += &format!("{enter}\n");
                if enter.is_empty() {
                    break;
                }
            }

            let program = Core::parse(code.to_string());
            println!("AST    : {program:?}");
            stackpp.eval(program);
            println!("Result : {stackpp:?}");
        }
    }
}

#[derive(Clone, Debug)]
enum Type {
    Number(f64),
    String(String),
    Bool(bool),
    Variable(String),
    Instruction(Instruction),
    Block(Vec<Type>),
    Error(Error),
}

impl Type {
    fn get_number(&self) -> f64 {
        match self {
            Type::Number(n) => n.to_owned(),
            _ => 0.0,
        }
    }

    fn get_string(&self) -> String {
        match self {
            Type::String(s) | Type::Variable(s) => s.to_owned(),
            Type::Number(n) => n.to_string(),
            _ => String::new(),
        }
    }

    fn get_bool(&self) -> bool {
        match self {
            Type::Bool(n) => n.to_owned(),
            _ => false,
        }
    }

    fn get_block(&self) -> Vec<Type> {
        match self {
            Type::Block(b) => b.to_owned(),
            other => vec![other.to_owned()],
        }
    }
}

#[derive(Clone, Debug)]
enum Error {
    StackEmpty,
}

#[derive(Clone, Debug)]
enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    Print,
    Input,
    Equal,
    LessThan,
    GreaterThan,
    Eval,
    When,
    IfElse,
    While,
    Until,
    Let,
    Pop,
}

#[derive(Clone, Debug)]
struct Core {
    stack: Vec<Type>,
    memory: HashMap<String, Type>,
}

impl Core {
    fn parse(source: String) -> Vec<Type> {
        fn tokenize_expr(input: String) -> Vec<String> {
            let mut tokens = Vec::new();
            let mut current_token = String::new();
            let mut in_parentheses: usize = 0;
            let mut in_quote = false;

            for c in input.chars() {
                match c {
                    '{' if !in_quote => {
                        in_parentheses += 1;
                        current_token.push(c);
                    }
                    '}' if !in_quote => {
                        if in_parentheses != 0 {
                            current_token.push(c);
                            in_parentheses -= 1;
                            if in_parentheses == 0 {
                                tokens.push(current_token.clone());
                                current_token.clear();
                            }
                        }
                    }
                    '"' => {
                        if in_parentheses == 0 {
                            if in_quote {
                                current_token.push(c);
                                in_quote = false;
                                tokens.push(current_token.clone());
                                current_token.clear();
                            } else {
                                in_quote = true;
                                current_token.push(c);
                            }
                        } else {
                            current_token.push(c);
                        }
                    }
                    ' ' | '\n' | '\t' | '\r' | '　' => {
                        if in_parentheses != 0 || in_quote {
                            current_token.push(c);
                        } else if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                    }
                    _ => {
                        current_token.push(c);
                    }
                }
            }

            if !(in_parentheses != 0 || in_quote || current_token.is_empty()) {
                tokens.push(current_token);
            }
            tokens
        }
        let mut result = vec![];
        for token in tokenize_expr(source) {
            let mut token = token.trim().to_string();
            if let Ok(n) = token.parse::<f64>() {
                result.push(Type::Number(n));
            } else if token.starts_with('"') && token.ends_with('"') {
                token.remove(token.find('"').unwrap_or_default());
                token.remove(token.rfind('"').unwrap_or_default());
                result.push(Type::String(token));
            } else if token.starts_with("{") && token.ends_with("}") {
                token.remove(token.find('{').unwrap_or_default());
                token.remove(token.rfind('}').unwrap_or_default());
                result.push(Type::Block(Core::parse(token)));
            } else if token.starts_with("$") {
                token.remove(token.find('$').unwrap_or_default());
                result.push(Type::Variable(token));
            } else {
                match token.as_str() {
                    "add" => result.push(Type::Instruction(Instruction::Add)),
                    "sub" => result.push(Type::Instruction(Instruction::Sub)),
                    "mul" => result.push(Type::Instruction(Instruction::Mul)),
                    "div" => result.push(Type::Instruction(Instruction::Div)),
                    "mod" => result.push(Type::Instruction(Instruction::Mod)),
                    "pow" => result.push(Type::Instruction(Instruction::Pow)),
                    "concat" => result.push(Type::Instruction(Instruction::Concat)),
                    "print" => result.push(Type::Instruction(Instruction::Print)),
                    "input" => result.push(Type::Instruction(Instruction::Input)),
                    "equal" => result.push(Type::Instruction(Instruction::Equal)),
                    "less-than" => result.push(Type::Instruction(Instruction::LessThan)),
                    "greater-than" => result.push(Type::Instruction(Instruction::GreaterThan)),
                    "eval" => result.push(Type::Instruction(Instruction::Eval)),
                    "when" => result.push(Type::Instruction(Instruction::When)),
                    "if-else" => result.push(Type::Instruction(Instruction::IfElse)),
                    "while" => result.push(Type::Instruction(Instruction::While)),
                    "until" => result.push(Type::Instruction(Instruction::Until)),
                    "let" => result.push(Type::Instruction(Instruction::Let)),
                    "pop" => result.push(Type::Instruction(Instruction::Pop)),
                    _ => {}
                }
            }
        }
        result
    }

    fn eval(&mut self, program: Vec<Type>) {
        for order in program {
            match order {
                Type::Instruction(instruction) => match instruction {
                    Instruction::Add => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Number(a + b))
                    }
                    Instruction::Sub => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Number(a - b))
                    }
                    Instruction::Mul => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Number(a * b))
                    }
                    Instruction::Div => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Number(a / b))
                    }
                    Instruction::Mod => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Number(a % b))
                    }
                    Instruction::Pow => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Number(a.powf(b)))
                    }
                    Instruction::Concat => {
                        let b = self.pop().get_string();
                        let a = self.pop().get_string();
                        self.stack.push(Type::String(a + &b));
                    }
                    Instruction::Print => {
                        let a = self.pop().get_string();
                        print!("{}", a);
                    }
                    Instruction::Input => self.stack.push(Type::String(
                        DefaultEditor::new().unwrap().readline("").unwrap(),
                    )),
                    Instruction::Equal => {
                        let b = self.pop().get_string();
                        let a = self.pop().get_string();
                        self.stack.push(Type::Bool(a == b));
                    }
                    Instruction::LessThan => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Bool(a < b))
                    }
                    Instruction::GreaterThan => {
                        let b = self.pop().get_number();
                        let a = self.pop().get_number();
                        self.stack.push(Type::Bool(a > b))
                    }
                    Instruction::Eval => {
                        let code = self.pop().get_block();
                        self.eval(code);
                    }
                    Instruction::When => {
                        let code = self.pop().get_block();
                        let condition = self.pop().get_bool();
                        if condition {
                            self.eval(code);
                        };
                    }
                    Instruction::IfElse => {
                        let code_false = self.pop().get_block();
                        let code_true = self.pop().get_block();
                        let condition = self.pop().get_bool();
                        if condition {
                            self.eval(code_true);
                        } else {
                            self.eval(code_false);
                        };
                    }
                    Instruction::While => {
                        let code = self.pop().get_block();
                        let condition = self.pop().get_block();
                        while {
                            self.eval(condition.clone());
                            self.pop().get_bool()
                        } {
                            self.eval(code.clone());
                        }
                    }
                    Instruction::Until => {
                        let code = self.pop().get_block();
                        let condition = self.pop().get_block();
                        while {
                            self.eval(condition.clone());
                            !self.pop().get_bool()
                        } {
                            self.eval(code.clone());
                        }
                    }
                    Instruction::Let => {
                        let name = self.pop().get_string();
                        let value = self.pop();
                        self.memory.insert(name, value);
                    }
                    Instruction::Pop => {
                        self.stack.pop();
                    }
                },
                Type::Variable(name) => {
                    if let Some(value) = self.memory.get(&name) {
                        self.stack.push(value.to_owned());
                    } else {
                        self.stack.push(Type::Variable(name));
                    }
                }
                other => self.stack.push(other),
            }
        }
    }

    fn pop(&mut self) -> Type {
        if let Some(value) = self.stack.pop() {
            value
        } else {
            Type::Error(Error::StackEmpty)
        }
    }
}
