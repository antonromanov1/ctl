use ctl::parser::lexing;
use ctl::parser::parse;
use ctl::parser::Node;
use ctl::parser::Token;

#[test]
fn lexical1() {
    let input = "
    fn main() -> i64 {
        // incremented number
        let mut num: i64 = 0;
        while (num < 4) {
            num = num + 1;
        }
        return 0;
    }
    "
    .to_string();

    let tokens = lexing(input).unwrap();

    let expected = vec![
        Token::Func,
        Token::Id("main".to_string()),
        Token::LParent,
        Token::RParent,
        Token::Arrow,
        Token::I64,
        Token::LBrace,
        Token::Let,
        Token::Mut,
        Token::Id("num".to_string()),
        Token::Colon,
        Token::I64,
        Token::Assign,
        Token::IntLiteral(0),
        Token::Semi,
        Token::While,
        Token::LParent,
        Token::Id("num".to_string()),
        Token::Lt,
        Token::IntLiteral(4),
        Token::RParent,
        Token::LBrace,
        Token::Id("num".to_string()),
        Token::Assign,
        Token::Id("num".to_string()),
        Token::Plus,
        Token::IntLiteral(1),
        Token::Semi,
        Token::RBrace,
        Token::Return,
        Token::IntLiteral(0),
        Token::Semi,
        Token::RBrace,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn lexical2() {
    let input = "
    fn main() {
        if (true) {}
        if (false) {}
        print();
    }
    "
    .to_string();

    let tokens = lexing(input).unwrap();

    let expected = vec![
        Token::Func,
        Token::Id("main".to_string()),
        Token::LParent,
        Token::RParent,
        Token::LBrace,
        Token::If,
        Token::LParent,
        Token::True,
        Token::RParent,
        Token::LBrace,
        Token::RBrace,
        Token::If,
        Token::LParent,
        Token::False,
        Token::RParent,
        Token::LBrace,
        Token::RBrace,
        Token::Id("print".to_string()),
        Token::LParent,
        Token::RParent,
        Token::Semi,
        Token::RBrace,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn parsing_empty_function() {
    let source = "
    fn main() {}
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![]);
}

#[test]
fn parsing_arithmetic_sum_literals() {
    let source = "
    fn main() {
        let num: i64 = -1 + 2;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let minus = Node::Minus(Box::new(Node::Integer(1)));
    let lit2 = Node::Integer(2);

    let add = Node::Add(Box::new(minus), Box::new(lit2));
    let let_ = Node::Let("num".to_string(), Some(Box::new(add)));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![let_]);
}

#[test]
fn parsing_arithmetic_sum_with_id() {
    let source = "
    fn main() {
        let num1 = 1;
        let num2: i64 = -1 + num1;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let minus = Node::Minus(Box::new(Node::Integer(1)));
    let let1 = Node::Let("num1".to_string(), Some(Box::new(Node::Integer(1))));

    let add = Node::Add(Box::new(minus), Box::new(Node::Id("num1".to_string())));
    let let2 = Node::Let("num2".to_string(), Some(Box::new(add)));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![let1, let2]);
}

#[test]
fn parsing_arithmetic_mul_sum_literals() {
    let source = "
    fn main() {
        let num: i64 = (1 + 2) * 3;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let lit1 = Node::Integer(1);
    let lit2 = Node::Integer(2);
    let lit3 = Node::Integer(3);

    let add = Node::Add(Box::new(lit1), Box::new(lit2));
    let mul = Node::Mul(Box::new(add), Box::new(lit3));
    let let_ = Node::Let("num".to_string(), Some(Box::new(mul)));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![let_]);
}

#[test]
fn parsing_arithmetic_div() {
    let source = "
    fn main(p: i64) {
        let num: i64 = p / 2;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let lit = Node::Integer(2);
    let id = Node::Id("p".to_string());
    let div = Node::Div(Box::new(id), Box::new(lit));
    let let_ = Node::Let("num".to_string(), Some(Box::new(div)));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![let_]);
}

#[test]
fn parsing_assign() {
    let source = "
    fn main() {
        let mut num;
        num = 0;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let int = Node::Integer(0);
    let let_ = Node::Let("num".to_string(), None);
    let assign = Node::Assign("num".to_string(), Box::new(int));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![let_, assign]);
}

#[test]
fn parsing_shifts() {
    let source = "
    fn main() {
        let num1 = 1 << 2;
        let num2 = 2 >> 1;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let lshift = Node::Shl(Box::new(Node::Integer(1)), Box::new(Node::Integer(2)));
    let rshift = Node::Shr(Box::new(Node::Integer(2)), Box::new(Node::Integer(1)));
    let let1 = Node::Let("num1".to_string(), Some(Box::new(lshift)));
    let let2 = Node::Let("num2".to_string(), Some(Box::new(rshift)));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![let1, let2]);
}

#[test]
fn parsing_if_one_block() {
    let source = "
    fn main() {
        if (0 == 0) {}
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let eq = Node::Eq(Box::new(Node::Integer(0)), Box::new(Node::Integer(0)));
    let if_stmt = Node::If(Box::new(eq), Box::new(Node::Block(Box::new(vec![]))), None);

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![if_stmt]);
}

#[test]
fn parsing_if_two_blocks() {
    let source = "
    fn main() {
        if (0 == 0) {} else {}
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let eq = Node::Eq(Box::new(Node::Integer(0)), Box::new(Node::Integer(0)));
    let empty_block = Node::Block(Box::new(vec![]));
    let if_stmt = Node::If(
        Box::new(eq),
        Box::new(empty_block.clone()),
        Some(Box::new(empty_block)),
    );

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![if_stmt]);
}

#[test]
fn parsing_while() {
    let source = "
    fn main() {
        while (true) {
            break;
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let block = Node::Block(Box::new(vec![Node::Break]));
    let while_ = Node::While(Box::new(Node::True), Box::new(block));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![while_]);
}

#[test]
fn parsing_call_separately() {
    let source = "
    fn main() {
        print(num, other, 1 + 1);
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let num1 = Node::Integer(1);
    let num2 = Node::Integer(1);
    let add = Node::Add(Box::new(num1), Box::new(num2));
    let id1 = Node::Id("num".to_string());
    let id2 = Node::Id("other".to_string());
    let call = Node::Call("print".to_string(), Box::new(vec![id1, id2, add]));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![call]);
}

#[test]
fn parsing_call_as_expression() {
    let source = "
    fn main() {
        let num = calc() + 1;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let num = Node::Integer(1);
    let call = Node::Call("calc".to_string(), Box::new(vec![]));
    let add = Node::Add(Box::new(call), Box::new(num));
    let let_ = Node::Let("num".to_string(), Some(Box::new(add)));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![let_]);
}

#[test]
fn parsing_return() {
    let source = "
    fn main() -> i64 {
        return 0;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let ret = Node::Return(Box::new(Node::Integer(0)));

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 1);
    assert_eq!(*funcs[0].get_stmts(), vec![ret]);
}

#[test]
fn parsing_two_functions() {
    let source = "
    fn print(num: i64) {}
    fn main() {}
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();

    // Create expected nodes
    let param = Node::Param("num".to_string());

    // Compare the parsed nodes with the expected ones
    assert_eq!(funcs.len(), 2);
    assert_eq!(*funcs[0].get_stmts(), vec![]);
    assert_eq!(*funcs[1].get_stmts(), vec![]);
    assert_eq!(*funcs[0].get_params(), vec![param]);
}
