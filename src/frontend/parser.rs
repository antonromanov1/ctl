use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Token {
    // Symbols
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,

    // Round brackets ()
    LParent,
    RParent,

    // Curly brackets {}
    LBrace,
    RBrace,

    // Comparison signs
    // Less than <, Greater than >
    Lt,
    Gt,

    // multisymbols
    // Left shift <<, Right shift >>, Less than or equal <=, Greater than or equal >=,
    // Equal ==, Not equal !=, Arrow ->
    Shl,
    Shr,
    Le,
    Ge,
    Eq,
    Ne,
    Arrow,

    Semi,
    Colon,
    Comma,

    // Keywords
    Func,
    Return,
    True,
    False,
    If,
    Else,
    While,
    Break,
    Continue,
    Let,
    Mut,
    I64,

    // etc
    IntLiteral(i64),
    Id(String),
    Eof,

    // A whitespace or a tab character
    Blank,

    // A new line
    LineFeed,

    COMMENT,
}

use std::fmt;

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::IntLiteral(int) => write!(f, "IntLiteral<{}>", int),
            Token::Plus => write!(f, "Plus"),
            Token::Minus => write!(f, "Minus"),
            Token::Star => write!(f, "Star"),
            Token::Slash => write!(f, "Slash"),
            Token::Percent => write!(f, "Percent"),
            Token::Assign => write!(f, "Assign"),
            Token::LParent => write!(f, "LParent"),
            Token::RParent => write!(f, "RParent"),
            Token::LBrace => write!(f, "LBrace"),
            Token::RBrace => write!(f, "RBrace"),
            Token::Shl => write!(f, "Shl"),
            Token::Shr => write!(f, "Shr"),
            Token::Lt => write!(f, "LessThan"),
            Token::Gt => write!(f, "GreaterThan"),
            Token::Le => write!(f, "LessThanOrEqual"),
            Token::Ge => write!(f, "GreaterThanOrEqual"),
            Token::Eq => write!(f, "Equal"),
            Token::Ne => write!(f, "NotEqual"),
            Token::Semi => write!(f, "Semi"),
            Token::Colon => write!(f, "Colon"),
            Token::Arrow => write!(f, "Arrow"),
            Token::Comma => write!(f, "Comma"),
            Token::Return => write!(f, "Return"),
            Token::Eof => write!(f, "Eof"),
            Token::Func => write!(f, "Function"),
            Token::Id(name) => write!(f, "ID<{}>", name),

            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "If"),
            Token::Else => write!(f, "Else"),
            Token::While => write!(f, "While"),
            Token::Break => write!(f, "Break"),
            Token::Continue => write!(f, "Continue"),
            Token::Let => write!(f, "Let"),
            Token::Mut => write!(f, "Mutable"),
            Token::I64 => write!(f, "i64"),
            Token::LineFeed => write!(f, "LineFeed"),
            _ => std::unreachable!("Got blank or comment token"),
        }
    }
}

impl Token {
    fn should_ignore(&self) -> bool {
        matches!(self, Token::Blank | Token::LineFeed | Token::COMMENT)
    }
}

/// Create the keywords map
/// Key: keywords string
/// Value: pair of the respective Token and length of keywords string
fn build_keywords() -> HashMap<&'static str, (Token, usize)> {
    const RETURN: &str = "return";
    const TRUE: &str = "true";
    const FALSE: &str = "false";
    const IF: &str = "if";
    const ELSE: &str = "else";
    const FN: &str = "fn";
    const LET: &str = "let";
    const I64: &str = "i64";
    const MUT: &str = "mut";
    const WHILE: &str = "while";
    const BREAK: &str = "break";
    const CONTINUE: &str = "continue";

    let mut keywords: HashMap<&str, (Token, usize)> = HashMap::with_capacity(12);
    keywords.insert(RETURN, (Token::Return, RETURN.len()));
    keywords.insert(TRUE, (Token::True, TRUE.len()));
    keywords.insert(FALSE, (Token::False, FALSE.len()));
    keywords.insert(IF, (Token::If, IF.len()));
    keywords.insert(ELSE, (Token::Else, ELSE.len()));
    keywords.insert(FN, (Token::Func, FN.len()));
    keywords.insert(LET, (Token::Let, LET.len()));
    keywords.insert(I64, (Token::I64, I64.len()));
    keywords.insert(MUT, (Token::Mut, MUT.len()));
    keywords.insert(WHILE, (Token::While, WHILE.len()));
    keywords.insert(BREAK, (Token::Break, BREAK.len()));
    keywords.insert(CONTINUE, (Token::Continue, CONTINUE.len()));

    keywords
}

type TokenLen = usize;

fn tokenize_symbols(input: &String) -> Result<Option<(Token, TokenLen)>, String> {
    if input.len() >= 2 {
        // Check the symbol has multilength at read-offset
        let multilength: String = Some(&input[0..2]).unwrap().into();
        if let Some(t) = tokenize_multisymbols(&multilength) {
            return Ok(Some((t, 2)));
        }
    }

    match input.as_bytes()[0] as char {
        '+' => Ok(Some((Token::Plus, 1))),
        '-' => Ok(Some((Token::Minus, 1))),
        '*' => Ok(Some((Token::Star, 1))),
        '/' => Ok(Some((Token::Slash, 1))),
        '%' => Ok(Some((Token::Percent, 1))),
        '(' => Ok(Some((Token::LParent, 1))),
        ')' => Ok(Some((Token::RParent, 1))),
        '{' => Ok(Some((Token::LBrace, 1))),
        '}' => Ok(Some((Token::RBrace, 1))),
        '<' => Ok(Some((Token::Lt, 1))),
        '>' => Ok(Some((Token::Gt, 1))),
        ':' => Ok(Some((Token::Colon, 1))),
        ';' => Ok(Some((Token::Semi, 1))),
        ',' => Ok(Some((Token::Comma, 1))),
        '=' => Ok(Some((Token::Assign, 1))),
        ' ' => Ok(Some((Token::Blank, count_len(input, |c| c == &' ')))),
        '\n' => Ok(Some((Token::LineFeed, 1))),
        '\t' => Ok(Some((Token::Blank, 1))),
        '\0' => Ok(Some((Token::Eof, 1))),
        c => Err(format!("unexpected mark '{}'", c)),
    }
}

fn tokenize_keywords(
    input: &str,
    keywords: &HashMap<&str, (Token, usize)>,
) -> Result<Option<(Token, TokenLen)>, String> {
    let length: TokenLen = count_len(input, |c| c.is_digit(10) || c == &'_' || c.is_alphabetic());

    if let Some(t) = keywords.get(&input[0..length]) {
        return Ok(Some((t.0.clone(), t.1)));
    }

    Ok(Some((
        Token::Id(input.chars().take(length).collect::<String>()),
        length,
    )))
}

fn is_decimal(ch: char) -> bool {
    ('0'..='9').contains(&ch)
}

fn count_len(input: &str, f: fn(ch: &char) -> bool) -> TokenLen {
    input.chars().take_while(f).collect::<String>().len()
}

fn tokenize_multisymbols(input: &str) -> Option<Token> {
    match input {
        "<<" => Some(Token::Shl),
        ">>" => Some(Token::Shr),
        "<=" => Some(Token::Le),
        ">=" => Some(Token::Ge),
        "==" => Some(Token::Eq),
        "!=" => Some(Token::Ne),
        "->" => Some(Token::Arrow),
        _ => None,
    }
}

fn tokenize(
    input: &mut String,
    keywords: &HashMap<&str, (Token, usize)>,
) -> Result<Option<(Token, TokenLen)>, String> {
    // return None if can not tokenize
    if input.is_empty() {
        return Ok(None);
    }

    match input.as_bytes()[0] as char {
        // keyword and identifier
        c if c.is_alphabetic() => tokenize_keywords(input, keywords),

        // integer-literal
        c if is_decimal(c) => {
            let length: TokenLen = count_len(input, |c| c.is_ascii_digit());
            Ok(Some((
                Token::IntLiteral(input[..length].parse::<i64>().unwrap()),
                length,
            )))
        }

        // ignore comment or Token::Slash
        '/' => {
            if input.as_bytes()[1] as char == '/' {
                let length: TokenLen = count_len(input, |c| c != &'\n') + 1;
                return Ok(Some((Token::COMMENT, length)));
            }
            tokenize_symbols(input)
        }

        // ignore white-space
        ' ' => Ok(Some((Token::Blank, count_len(input, |c| c == &' ')))),
        // symbol
        _ => tokenize_symbols(input),
    }
}

type ParseResult<T> = Result<T, String>;

pub fn lexing(mut input: String) -> ParseResult<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::with_capacity(2048);

    // build all keywords they used in izber
    let keywords: HashMap<&str, (Token, usize)> = build_keywords();

    // append this_token to tokens while given tokens are valid
    while let Some((t, idx)) = tokenize(&mut input, &keywords)? {
        // next point
        input.drain(..idx);

        if t.should_ignore() {
            continue;
        }

        tokens.push(t.clone());
        // if this_token is End-Of-File then we should exit from tokenize
        if let &Token::Eof = &t {
            break;
        }
    }

    Ok(tokens)
}

type Name = String;
type Child = Box<Node>;
type Expr = Box<Node>;
type Condition = Box<Node>;
type BlockNode = Box<Node>;
type Alter = Option<Box<Node>>;
type Elements = Box<Vec<Node>>;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    // Unary-operation
    Neg(Child),

    // Numeric literal
    Integer(i64),

    // Identifier
    Id(Name),

    // Binary arithmetic operations
    Add(Child, Child),
    Sub(Child, Child),
    Mul(Child, Child),
    Div(Child, Child),
    Mod(Child, Child),

    Shl(Child, Child),
    Shr(Child, Child),

    // Boolean literals
    True,
    False,

    // Equality
    Eq(Child, Child),
    Ne(Child, Child),

    // Relation
    Lt(Child, Child),
    Gt(Child, Child),
    Le(Child, Child),
    Ge(Child, Child),

    // Statements
    Let(Name, Expr),
    Assign(Name, Expr),
    If(Condition, BlockNode, Alter),
    While(Condition, BlockNode),
    Break,
    Continue,
    Block(Elements),
    ReturnVoid,
    Return(Expr),

    // Name of calling function, passing arguments and is call separate or it is a
    // subexpression.
    Call(Name, Elements, bool),
}

macro_rules! elements_to_string {
    ($box:expr) => {{
        let mut elements = String::new();
        for node in &**$box {
            elements.push_str(&format!("{}, ", node.to_string()));
        }
        elements
    }};
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Add(lch, rch) => write!(f, "Add<{}, {}>", lch, rch),
            Node::Sub(lch, rch) => write!(f, "Sub<{}, {}>", lch, rch),
            Node::Mul(lch, rch) => write!(f, "Mul<{}, {}>", lch, rch),
            Node::Div(lch, rch) => write!(f, "Div<{}, {}>", lch, rch),
            Node::Mod(lch, rch) => write!(f, "Mod<{}, {}>", lch, rch),

            Node::Ne(lch, rch) => write!(f, "Ne<{},{}>", lch, rch),
            Node::Eq(lch, rch) => write!(f, "Eq<{},{}>", lch, rch),
            Node::Lt(lch, rch) => write!(f, "Lt<{},{}>", lch, rch),
            Node::Gt(lch, rch) => write!(f, "Gt<{},{}>", lch, rch),
            Node::Le(lch, rch) => write!(f, "Le<{},{}>", lch, rch),
            Node::Ge(lch, rch) => write!(f, "Ge<{},{}>", lch, rch),

            Node::Shl(lch, rch) => write!(f, "Shl<{},{}>", lch, rch),
            Node::Shr(lch, rch) => write!(f, "Shr<{},{}>", lch, rch),

            Node::Neg(child) => write!(f, "Neg<{}>", child),

            Node::True => write!(f, "True"),
            Node::False => write!(f, "False"),
            Node::Integer(val) => write!(f, "Int<{}> ", val),

            Node::Id(name) => write!(f, "Id<{}>", name),
            Node::ReturnVoid => write!(f, "ReturnVoid"),
            Node::Return(expr) => write!(f, "Return({})", expr),

            Node::Let(name, expr) => write!(f, "Let {} = {}", name, expr),
            Node::Assign(id, expr) => write!(f, "Assign<{}>({})", id, expr),

            Node::Block(stmts) => {
                let elements = elements_to_string!(stmts);
                write!(f, "Block with {} elements: {}", stmts.len(), elements)
            }
            Node::Call(id, args, _) => {
                let arguments = elements_to_string!(args);
                write!(f, "Call {}, args: {}", id, arguments)
            }

            Node::While(cond, stmts) => {
                write!(f, "While {}:\n\t\t{}", cond, (*stmts))
            }
            Node::Break => write!(f, "Break"),
            Node::Continue => write!(f, "Continue"),

            Node::If(cond, stmts, alter) => match alter {
                Some(alt) => write!(f, "IF<{},{}> ELSE<{}>", cond, stmts, alt),
                None => write!(f, "IF<{},{}>", cond, stmts),
            },
        }
    }
}

#[derive(Clone)]
pub struct Func {
    name: String,
    stmts: Vec<Node>,
    params: Vec<String>,
}

impl Func {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn params(&self) -> &[String] {
        &self.params
    }

    pub fn stmts(&self) -> &[Node] {
        &self.stmts
    }
}

pub fn dump_ast(funcs: &[Func]) {
    println!("--------Dump AST--------");
    for f in funcs.iter() {
        println!("Function {}", f.name);
        for st in f.stmts.iter() {
            println!("\t{}", st);
        }
    }
}

/// Field tokens is written after lexing one time and is never rewritten, only read.
/// The source language is only able to have top level variable declarations and no ones in
/// the inner scopes. Therefore we use cur_variables set for all of the local variables and the
/// parameters of a function.
struct Parser {
    tokens: Vec<Token>,
    // Index of current token in vector of tokens
    cur: usize,
    // Index of the next one
    next: usize,

    funcs: Vec<Func>,
    cur_variables: HashSet<String>,
    // Does current function have a return type
    return_type: bool,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            funcs: Vec::with_capacity(100),
            cur_variables: HashSet::new(),
            return_type: false,
            cur: 0,
            next: 1,
        }
    }

    // Parse whole parsing source code
    fn top_level(&mut self) -> ParseResult<()> {
        debug_assert!(self.funcs.is_empty());

        loop {
            let t: &Token = self.cur_token();

            match t {
                &Token::Func => {
                    let cur = self.parse_func()?;
                    self.funcs.push(cur);
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn stmt(&mut self) -> ParseResult<Node> {
        let t: Token = self.get_token();
        match t {
            Token::Return => self.parse_return(),

            Token::Id(name) => {
                // After the ID there should be either the equal sign (which means this is an assign)
                // or opening parenthesis.

                if *self.next_token() == Token::Assign {
                    if !self.cur_variables.contains(&name) {
                        return Err(format!("Assign to undeclared variable {}", name));
                    }
                    return self.parse_assign();
                }

                if *self.next_token() != Token::LParent {
                    return Err("Undefined token after id".to_string());
                }

                self.parse_call(name)
            }

            Token::LBrace => self.parse_block(),

            Token::While => self.parse_while(),
            Token::Break => {
                self.go_next_token();
                self.expect(&Token::Semi)?;
                Ok(Node::Break)
            }
            Token::Continue => {
                self.go_next_token();
                self.expect(&Token::Semi)?;
                Ok(Node::Continue)
            }

            Token::If => self.parse_if(),
            _ => Err(format!("statement can't start with '{}'", t)),
        }
    }

    fn parse_func(&mut self) -> ParseResult<Func> {
        self.expect(&Token::Func)?;
        let func_name: String = self.consume_id()?;
        self.expect(&Token::LParent)?;

        // Parse function parameter declarations, add parameter names to cur_variables.
        let mut func_params = Vec::new();
        self.cur_variables = HashSet::new();
        while !self.consume(&Token::RParent) {
            let param_name = self.define_param()?;
            func_params.push(param_name.clone());
            self.cur_variables.insert(param_name.clone());

            if !self.consume(&Token::Comma) {
                self.expect(&Token::RParent)?;
                break;
            }
        }

        self.return_type = false;
        if self.consume(&Token::Arrow) {
            self.consume_typename()?;
            self.return_type = true;
        }

        self.expect(&Token::LBrace)?;

        let mut func_stmts = Vec::new();
        // Parse local variable declarations.
        while self.cur_token() == &Token::Let {
            let let_ = self.parse_let()?;
            if let Node::Let(name, _expr) = let_.clone() {
                self.cur_variables.insert(name.clone());
            } else {
                std::unreachable!();
            }
            func_stmts.push(let_);
        }

        // Parse function statements including blocks.
        while !self.consume(&Token::RBrace) {
            let st: Node = self.stmt()?;
            func_stmts.push(st);
        }

        Ok(Func {
            name: func_name,
            params: func_params,
            stmts: func_stmts,
        })
    }

    fn define_param(&mut self) -> ParseResult<String> {
        let param_name: String = self.consume_id()?;
        self.consume(&Token::Colon);
        self.consume_typename()?;

        Ok(param_name)
    }

    fn parse_while(&mut self) -> ParseResult<Node> {
        self.expect(&Token::While)?;
        self.expect(&Token::LParent)?;
        let cond: Node = self.expr()?;
        self.expect(&Token::RParent)?;
        let stmt: Node = self.stmt()?;
        Ok(Node::While(Box::new(cond), Box::new(stmt)))
    }

    fn parse_block(&mut self) -> ParseResult<Node> {
        let stmts: Vec<Node> = self.compound_stmt()?;
        Ok(Node::Block(Box::new(stmts)))
    }

    fn parse_if(&mut self) -> ParseResult<Node> {
        self.expect(&Token::If)?;
        self.expect(&Token::LParent)?;
        let cond: Node = self.expr()?;
        self.expect(&Token::RParent)?;

        let stmt: Node = self.stmt()?;
        if !self.consume(&Token::Else) {
            return Ok(Node::If(Box::new(cond), Box::new(stmt), None));
        }
        Ok(Node::If(
            Box::new(cond),
            Box::new(stmt),
            Some(Box::new(self.stmt()?)),
        ))
    }

    fn parse_let(&mut self) -> ParseResult<Node> {
        self.expect(&Token::Let)?;
        self.expect(&Token::Mut)?;
        let id_name: String = self.consume_id()?;
        self.expect(&Token::Colon)?;
        self.consume_typename()?;
        self.expect(&Token::Assign)?;
        let expr = self.expr()?;
        self.expect(&Token::Semi)?;

        Ok(Node::Let(id_name, Box::new(expr)))
    }

    fn parse_return(&mut self) -> ParseResult<Node> {
        self.expect(&Token::Return)?;
        if let Token::Semi = self.cur_token() {
            // `let` expressions in this position are experimental
            // cargo 1.54.0 (5ae8d74b3 2021-06-22)
            if self.return_type {
                return Err("Function with a returning type returns void".to_string());
            }

            self.go_next_token();
            return Ok(Node::ReturnVoid);
        }

        let expr: Node = self.expr()?;
        if !self.return_type {
            return Err(format!(
                "Function with no return type returns value: {}",
                expr
            ));
        }

        self.expect(&Token::Semi)?;
        Ok(Node::Return(Box::new(expr)))
    }

    fn parse_call(&mut self, name: String) -> ParseResult<Node> {
        let id_name: String = self.consume_id()?;
        debug_assert!(id_name == name);
        self.expect(&Token::LParent)?;

        let mut args: Vec<Node> = Vec::new();
        loop {
            match self.cur_token() {
                Token::Comma => self.go_next_token(),
                Token::RParent => {
                    self.go_next_token();
                    break;
                }
                _ => {
                    let expr = self.expr()?;
                    args.push(expr);
                }
            }
        }

        self.check_call(&name, args.len())?;

        self.expect(&Token::Semi)?;
        Ok(Node::Call(name, Box::new(args), false))
    }

    fn parse_assign(&mut self) -> ParseResult<Node> {
        let id_name: String = self.consume_id()?;
        self.expect(&Token::Assign)?;
        let expr: Node = self.expr()?;
        self.expect(&Token::Semi)?;

        Ok(Node::Assign(id_name, Box::new(expr)))
    }

    fn expr(&mut self) -> ParseResult<Node> {
        self.equal()
    }

    fn equal(&mut self) -> ParseResult<Node> {
        let mut lhs: Node = self.relation()?;

        while self.check_vec(vec![Token::Eq, Token::Ne]) {
            let op: Token = self.get_token();
            self.go_next_token();

            if let &Token::Eq = &op {
                lhs = Node::Eq(Box::new(lhs), Box::new(self.relation()?));
            } else if let &Token::Ne = &op {
                lhs = Node::Ne(Box::new(lhs), Box::new(self.relation()?));
            }
        }

        Ok(lhs)
    }

    fn relation(&mut self) -> ParseResult<Node> {
        let mut lhs: Node = self.shift()?;

        while self.check_vec(vec![Token::Lt, Token::Gt, Token::Le, Token::Ge]) {
            let op: Token = self.get_token();
            self.go_next_token();

            if let &Token::Lt = &op {
                lhs = Node::Lt(Box::new(lhs), Box::new(self.relation()?));
            } else if let &Token::Gt = &op {
                lhs = Node::Gt(Box::new(lhs), Box::new(self.relation()?));
            } else if let &Token::Le = &op {
                lhs = Node::Le(Box::new(lhs), Box::new(self.relation()?));
            } else if let &Token::Ge = &op {
                lhs = Node::Ge(Box::new(lhs), Box::new(self.relation()?));
            }
        }

        Ok(lhs)
    }

    fn shift(&mut self) -> ParseResult<Node> {
        let mut lhs: Node = self.add_sub()?;

        loop {
            if self.check(&Token::Shl) {
                self.go_next_token();
                lhs = Node::Shl(Box::new(lhs), Box::new(self.add_sub()?));
            } else if self.check(&Token::Shr) {
                self.go_next_token();
                lhs = Node::Shr(Box::new(lhs), Box::new(self.add_sub()?));
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    fn add_sub(&mut self) -> ParseResult<Node> {
        let mut lhs: Node = self.mul_div()?;

        while self.check_vec(vec![Token::Plus, Token::Minus]) {
            let op: Token = self.get_token();
            self.go_next_token();
            if let Token::Plus = op {
                lhs = Node::Add(Box::new(lhs), Box::new(self.mul_div()?));
            } else if let Token::Minus = op {
                lhs = Node::Sub(Box::new(lhs), Box::new(self.mul_div()?));
            }
        }

        Ok(lhs)
    }

    fn mul_div(&mut self) -> ParseResult<Node> {
        let mut lhs: Node = self.unary()?;

        while self.check_vec(vec![Token::Star, Token::Slash, Token::Percent]) {
            let op: Token = self.get_token();
            self.go_next_token();
            if let Token::Star = op {
                lhs = Node::Mul(Box::new(lhs), Box::new(self.unary()?));
            } else if let Token::Slash = op {
                lhs = Node::Div(Box::new(lhs), Box::new(self.unary()?));
            } else if let Token::Percent = op {
                lhs = Node::Mod(Box::new(lhs), Box::new(self.unary()?));
            }
        }

        Ok(lhs)
    }

    fn unary(&mut self) -> ParseResult<Node> {
        let t: Token = self.get_token();
        match t {
            Token::Minus => {
                self.go_next_token();
                Ok(Node::Neg(Box::new(self.unary()?)))
            }
            _ => self.term(),
        }
    }

    fn term(&mut self) -> ParseResult<Node> {
        let t: Token = self.get_token();

        match t {
            Token::LParent => {
                self.expect(&Token::LParent)?;
                let expr: Node = self.expr()?;
                self.expect(&Token::RParent)?;
                Ok(expr)
            }

            Token::IntLiteral(val) => {
                self.go_next_token();
                Ok(Node::Integer(val))
            }

            Token::True => {
                self.go_next_token();
                Ok(Node::True)
            }
            Token::False => {
                self.go_next_token();
                Ok(Node::False)
            }

            Token::Id(name) => {
                self.go_next_token();
                let t: Token = self.get_token();
                match t {
                    // Call case
                    Token::LParent => {
                        // TODO: refactor using call of `parse_call`
                        self.expect(&Token::LParent)?;
                        let mut args: Vec<Node> = Vec::new();

                        while !self.consume(&Token::RParent) {
                            args.push(self.expr()?);
                            if !self.consume(&Token::Comma) {
                                self.expect(&Token::RParent)?;
                                break;
                            }
                        }

                        self.check_call(&name, args.len())?;

                        Ok(Node::Call(name, Box::new(args), true))
                    }

                    _ => {
                        if self.cur_variables.contains(&name) {
                            Ok(Node::Id(name))
                        } else {
                            return Err(format!("Use of undeclared variable {}", name));
                        }
                    }
                }
            }

            _ => Err(format!("term can't start with '{}'", t)),
        }
    }

    /// Parse block of statements (begining with opening curly brace and ending with the closing one)
    /// and return vector of the nodes
    fn compound_stmt(&mut self) -> ParseResult<Vec<Node>> {
        let mut stmts: Vec<Node> = Vec::new();
        self.expect(&Token::LBrace)?;
        while !self.consume(&Token::RBrace) {
            let st: Node = self.stmt()?;
            stmts.push(st);
        }
        Ok(stmts)
    }

    fn expect(&mut self, t: &Token) -> ParseResult<()> {
        let cur: &Token = self.cur_token();
        if t == cur {
            self.go_next_token();
            return Ok(());
        }
        Err(format!("expected {} but got '{}'", t, cur))
    }

    fn consume(&mut self, t: &Token) -> bool {
        let cur: &Token = self.cur_token();
        if t == cur {
            self.go_next_token();
            true
        } else {
            false
        }
    }

    fn consume_typename(&mut self) -> ParseResult<Token> {
        let t: Token = self.get_token();
        match t {
            Token::I64 => {
                self.go_next_token();
                Ok(Token::I64)
            }

            _ => Err(format!("got {}, it's not a type name ", t)),
        }
    }

    fn consume_id(&mut self) -> ParseResult<String> {
        let t: Token = self.get_token();
        if let Token::Id(name) = t {
            self.go_next_token();
            Ok(name)
        } else {
            Err(format!("expected identifier but got '{}'", t))
        }
    }

    fn check_vec(&self, tks: Vec<Token>) -> bool {
        tks.iter().any(|t| t == self.cur_token())
    }

    fn get_token(&self) -> Token {
        if self.cur >= self.tokens.len() {
            return Token::Eof;
        }
        self.tokens[self.cur].clone()
    }

    fn cur_token(&self) -> &Token {
        if self.cur >= self.tokens.len() {
            return &Token::Eof;
        }
        &self.tokens[self.cur]
    }

    fn next_token(&self) -> &Token {
        if self.cur >= self.tokens.len() {
            return &Token::Eof;
        }
        &self.tokens[self.next]
    }

    fn go_next_token(&mut self) {
        self.cur += 1;
        self.next += 1;
    }

    fn check(&self, t: &Token) -> bool {
        if self.cur_token() == t {
            return true;
        }
        false
    }

    fn check_call(&self, name: &str, args_len: usize) -> Result<(), String> {
        for func in &self.funcs {
            if func.name.as_str() != name {
                continue;
            }

            if func.params.len() != args_len {
                return Err(format!(
                    "Function {} takes {} arguments but {} was given",
                    name,
                    func.params.len(),
                    args_len
                ));
            } else {
                return Ok(());
            }
        }

        const PRINT_ARGS_LEN: usize = 1;
        // Should be a built-in function
        if name == "print" {
            if args_len == PRINT_ARGS_LEN {
                return Ok(());
            } else {
                return Err(format!(
                    "Function print takes 1 argument but {} was given",
                    args_len
                ));
            }
        }

        Err(format!("No function named {} defined", name))
    }
}

pub fn parse(source: String) -> ParseResult<Vec<Func>> {
    let tokens = lexing(source)?;
    let mut parser: Parser = Parser::new(tokens);
    parser.top_level()?;
    Ok(parser.funcs)
}
