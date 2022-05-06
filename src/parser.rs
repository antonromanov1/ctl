use std::collections::BTreeMap;
use std::collections::HashMap;

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

impl Token {
    fn to_string(&self) -> String {
        match self {
            Token::IntLiteral(int) => format!("IntLiteral<{}>", int),
            Token::Plus => "Plus".to_string(),
            Token::Minus => "Minus".to_string(),
            Token::Star => "Star".to_string(),
            Token::Slash => "Slash".to_string(),
            Token::Percent => "Percent".to_string(),
            Token::Assign => "Assign".to_string(),
            Token::LParent => "LParent".to_string(),
            Token::RParent => "RParent".to_string(),
            Token::LBrace => "LBrace".to_string(),
            Token::RBrace => "RBrace".to_string(),
            Token::Shl => "Shl".to_string(),
            Token::Shr => "Shr".to_string(),
            Token::Lt => "LessThan".to_string(),
            Token::Gt => "GreaterThan".to_string(),
            Token::Le => "LessThanOrEqual".to_string(),
            Token::Ge => "GreaterThanOrEqual".to_string(),
            Token::Eq => "Equal".to_string(),
            Token::Ne => "NotEqual".to_string(),
            Token::Semi => "Semi".to_string(),
            Token::Colon => "Colon".to_string(),
            Token::Arrow => "Arrow".to_string(),
            Token::Comma => "Comma".to_string(),
            Token::Return => "Return".to_string(),
            Token::Eof => "Eof".to_string(),
            Token::Func => "Function".to_string(),
            Token::Id(name) => format!("ID<{}>", name),

            Token::True => "true".to_string(),
            Token::False => "false".to_string(),
            Token::If => "If".to_string(),
            Token::Else => "Else".to_string(),
            Token::While => "While".to_string(),
            Token::Break => "Break".to_string(),
            Token::Let => "Let".to_string(),
            Token::Mut => "Mutable".to_string(),
            Token::I64 => "i64".to_string(),
            Token::LineFeed => "LineFeed".to_string(),
            _ => "".to_string(),
        }
    }

    fn should_ignore(&self) -> bool {
        match self {
            Token::Blank | Token::LineFeed | Token::COMMENT => true,
            _ => false,
        }
    }
}

// Create the keywords map
// Key: keywords string
// Value: pair of the respective Token and length of keywords string
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

    keywords
}

type TokenLen = usize;

fn tokenize_symbols(input: &String) -> Result<Option<(Token, TokenLen)>, String> {
    if input.len() >= 2 {
        // Check the symbol has multilength at read-offset
        let multilength: String = std::str::from_utf8(&input.as_bytes()[0..2]).unwrap().into();
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
    input: &String,
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
    '0' <= ch && ch <= '9'
}

fn count_len(input: &String, f: fn(ch: &char) -> bool) -> TokenLen {
    input.chars().take_while(f).collect::<String>().len()
}

fn tokenize_multisymbols(input: &String) -> Option<Token> {
    match input.as_str() {
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
    if input.len() == 0 {
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
    // Functions parameter
    Param(Name),

    // Unary-operation
    Minus(Child),

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
    Let(Name, Option<Expr>),
    Assign(Name, Expr),
    If(Condition, BlockNode, Alter),
    While(Condition, BlockNode),
    Break,
    Block(Elements),
    Return(Expr),

    Call(Name, Elements),

    Invalid,
}

macro_rules! elements_to_string {
    ($box:expr) => {{
        let mut elements = String::new();
        for node in &**$box {
            elements.push_str(&format!("{}, ", node.string()));
        }
        elements
    }};
}

impl Node {
    pub fn string(&self) -> String {
        match self {
            Node::Add(lch, rch) => format!("Add<{}, {}>", lch.string(), rch.string()),
            Node::Sub(lch, rch) => format!("Sub<{}, {}>", lch.string(), rch.string()),
            Node::Mul(lch, rch) => format!("Mul<{}, {}>", lch.string(), rch.string()),
            Node::Div(lch, rch) => format!("Div<{}, {}>", lch.string(), rch.string()),
            Node::Mod(lch, rch) => format!("Mod<{}, {}>", lch.string(), rch.string()),

            Node::Ne(lch, rch) => format!("Ne<{},{}>", lch.string(), rch.string()),
            Node::Eq(lch, rch) => format!("Eq<{},{}>", lch.string(), rch.string()),
            Node::Lt(lch, rch) => format!("Lt<{},{}>", lch.string(), rch.string()),
            Node::Gt(lch, rch) => format!("Gt<{},{}>", lch.string(), rch.string()),
            Node::Le(lch, rch) => format!("Le<{},{}>", lch.string(), rch.string()),
            Node::Ge(lch, rch) => format!("Ge<{},{}>", lch.string(), rch.string()),

            Node::Shl(lch, rch) => format!("Shl<{},{}>", lch.string(), rch.string()),
            Node::Shr(lch, rch) => format!("Shr<{},{}>", lch.string(), rch.string()),

            Node::Minus(ch) => format!("Minus<{}>", ch.string()),

            Node::True => "True".to_string(),
            Node::False => "False".to_string(),
            Node::Integer(val) => format!("Int<{}> ", val),

            Node::Id(name) => format!("Id<{}>", name),
            Node::Return(expr) => format!("Return({})", expr.string()),

            Node::Let(id, option) => match option {
                Some(expr) => format!("Let <{}> = ({})", id, expr.string()),
                None => format!("Let <{}>", id),
            },
            Node::Assign(id, expr) => format!("Assign<{}>({})", id, expr.string()),

            Node::Block(stmts) => {
                let elements = elements_to_string!(stmts);
                format!("Block with {} elements: {}", stmts.len(), elements)
            }
            Node::Call(id, args) => {
                let arguments = elements_to_string!(args);
                format!("Call {}, args: {}", id, arguments)
            }

            Node::Param(name) => format!("Param<{}>", name),

            Node::While(cond, stmts) => {
                format!("While {}:\n\t\t{}", cond.string(), (*stmts).string())
            }
            Node::Break => "Break".to_string(),

            Node::If(cond, stmts, alter) => match alter {
                Some(alt) => format!(
                    "IF<{},{}> ELSE<{}>",
                    cond.string(),
                    stmts.string(),
                    alt.string()
                ),
                None => format!("IF<{},{}>", cond.string(), stmts.string()),
            },

            _ => "INVALID".to_string(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct Symbol {
    stack_offset: usize,
    is_mutable: bool,
}

impl Symbol {
    fn new(offset: usize, flg: bool) -> Self {
        Self {
            stack_offset: offset,
            is_mutable: flg,
        }
    }
}

#[derive(Clone)]
struct Env {
    sym_table: BTreeMap<String, Symbol>,
    prev: Option<Box<Env>>,
}

impl Env {
    fn new() -> Env {
        Env {
            sym_table: BTreeMap::new(),
            prev: None,
        }
    }
}

#[derive(Clone)]
pub struct Func {
    name: String,
    stmts: Vec<Node>,
    params: Vec<Node>,
    env: Env,
}

impl Func {
    pub fn new() -> Self {
        Func {
            name: String::new(),
            stmts: Vec::new(),
            params: Vec::new(),
            env: Env::new(),
        }
    }

    pub fn get_params(&self) -> &Vec<Node> {
        &self.params
    }

    pub fn get_stmts(&self) -> &Vec<Node> {
        &self.stmts
    }
}

pub fn dump_ast(funcs: &Vec<Func>) {
    println!("--------Dump AST--------");
    for f in funcs.iter() {
        println!("Function {}", f.name);
        for st in f.stmts.iter() {
            println!("\t{}", st.string());
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    funcs: Vec<Func>,
    cur_env: Env,

    // Index of current token in vector of tokens
    cur: usize,
    // Index of the next one
    next: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            funcs: Vec::with_capacity(100),
            cur_env: Env::new(),
            cur: 0,
            next: 1,
        }
    }

    // Parse whole parsing source code
    fn top_level(&mut self) -> ParseResult<()> {
        let global = Env::new();
        let mut funcs = Vec::with_capacity(100);

        loop {
            let t: &Token = &self.get_token();

            match t {
                &Token::Func => {
                    funcs.push(self.parse_func(global.clone())?);
                }
                _ => break,
            }
        }

        self.funcs = funcs;
        Ok(())
    }

    fn stmt(&mut self) -> ParseResult<Node> {
        let t: Token = self.get_token();
        match t {
            Token::Return => self.parse_return(),
            Token::Let => self.parse_let(),

            Token::Id(name) => {
                if *self.next_token() == Token::Assign {
                    return self.parse_assign();
                }
                if *self.next_token() != Token::LParent {
                    return Err(format!("Undefined token after id"));
                }
                return self.parse_call(name.clone());
            }

            Token::LBrace => self.parse_block(),

            Token::While => self.parse_while(),
            Token::Break => {
                self.go_next_token();
                self.expect(&Token::Semi)?;
                Ok(Node::Break)
            }

            Token::If => self.parse_if(),
            _ => Err(format!("statement can't start with '{}'", t.to_string())),
        }
    }

    fn parse_func(&mut self, global: Env) -> ParseResult<Func> {
        self.cur_env = Env::new();
        self.cur_env.prev = Some(Box::new(global));
        self.go_next_token();
        let func_name: String = self.consume_id()?;
        self.expect(&Token::LParent)?;
        let mut func_params: Vec<Node> = Vec::new();

        while !self.consume(&Token::RParent) {
            func_params.push(self.define_param()?);
            if !self.consume(&Token::Comma) {
                self.expect(&Token::RParent)?;
                break;
            }
        }

        if self.consume(&Token::Arrow) {
            self.consume_typename()?;
        }

        let func_stmts: Vec<Node> = self.compound_stmt()?;
        Ok(Func {
            name: func_name,
            params: func_params,
            stmts: func_stmts,
            env: self.cur_env.clone(),
        })
    }

    fn define_param(&mut self) -> ParseResult<Node> {
        let mutable: bool = self.consume(&Token::Mut);
        let param_name: String = self.consume_id()?;
        self.consume(&Token::Colon);
        self.consume_typename()?;
        self.cur_env
            .sym_table
            .insert(param_name.clone(), Symbol::new(0, mutable));

        Ok(Node::Param(param_name))
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
        let mutable_flag: bool = self.consume(&Token::Mut);
        let id_name: String = self.consume_id()?;
        if self.cur_token() == &Token::Colon {
            self.go_next_token();
            self.consume_typename()?;
        }
        if self.consume(&Token::Assign) {
            let expr: Node = self.expr()?;
            self.cur_env
                .sym_table
                .insert(id_name.clone(), Symbol::new(0, mutable_flag));
            self.expect(&Token::Semi)?;

            Ok(Node::Let(id_name, Some(Box::new(expr))))
        } else {
            if self.cur_token() != &Token::Semi {
                return Err(format!(
                    "Expected a semicolon, got {}",
                    self.cur_token().to_string()
                ));
            }
            self.go_next_token();
            Ok(Node::Let(id_name, None))
        }
    }

    fn parse_return(&mut self) -> ParseResult<Node> {
        self.expect(&Token::Return)?;
        let expr: Node = self.expr()?;
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

        self.expect(&Token::Semi)?;
        Ok(Node::Call(name, Box::new(args)))
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
        self.ensure_valid(&lhs)?;

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
        self.ensure_valid(&lhs)?;

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
        self.ensure_valid(&lhs)?;

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
        self.ensure_valid(&lhs)?;

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
        self.ensure_valid(&lhs)?;

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
                Ok(Node::Minus(Box::new(self.unary()?)))
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
                    Token::LParent => {
                        self.expect(&Token::LParent)?;
                        let mut args: Vec<Node> = Vec::new();

                        while !self.consume(&Token::RParent) {
                            args.push(self.expr()?);
                            if !self.consume(&Token::Comma) {
                                self.expect(&Token::RParent)?;
                                break;
                            }
                        }

                        Ok(Node::Call(name, Box::new(args)))
                    }

                    _ => Ok(Node::Id(name)),
                }
            }

            _ => Err(format!("term can't start with '{}'", t.to_string())),
        }
    }

    // Parse block of statements (begining with opening curly brace and ending with the closing one)
    // and return vector of the nodes
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
        Err(format!(
            "expected {} but got '{}'",
            t.to_string(),
            cur.to_string()
        ))
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

            Token::Id(name) => {
                self.go_next_token();
                Ok(Token::Id(name.to_string()))
            }

            _ => Err(format!("got {} it's not typename ", t.to_string())),
        }
    }

    fn consume_id(&mut self) -> ParseResult<String> {
        let t: Token = self.get_token();
        if let Token::Id(name) = t {
            self.go_next_token();
            Ok(name.to_string())
        } else {
            Err(format!("expected identifier but got '{}'", t.to_string()))
        }
    }

    fn ensure_valid(&mut self, n: &Node) -> ParseResult<()> {
        if let &Node::Invalid = n {
            return Err("got INVALID Node".to_string());
        }
        Ok(())
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
}

pub fn parse(source: String) -> ParseResult<Vec<Func>> {
    let tokens = lexing(source)?;
    let mut parser: Parser = Parser::new(tokens);
    parser.top_level()?;
    Ok(parser.funcs)
}