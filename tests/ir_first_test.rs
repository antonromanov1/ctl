use ctl::ir_first::generate_insts;
use ctl::ir_first::Inst;
use ctl::parser::parse;

#[test]
fn generate_empty_function_no_param() {
    let source = "
    fn main() {}
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Try to generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(insts.is_empty());
}

fn dump(insts: Vec<Inst>) -> String {
    let mut res = String::new();
    for (i, inst) in insts.iter().enumerate() {
        res.push_str("\n        ");
        res.push_str(&format!("{}: {}", i, inst.to_string()));
    }

    res
}

#[test]
fn generate_empty_function_few_parameters() {
    let source = "
    fn main(p0: i64, p1: i64) {}
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        0: v0 = Parameter
        1: v1 = Parameter"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_function_returning_its_param() {
    let source = "
    fn main(p: i64) -> i64 {
        return p;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        0: v0 = Parameter
        1: Return v0"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_function_returning_param_plus_local() {
    let source = "
    fn main(p: i64) -> i64 {
        let mut local: i64 = 1;
        return p + local;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        0: v0 = Parameter
        1: MoveImm v2, 1
        2: Move v1, v2
        3: v3 = Add(v0, v1)
        4: Return v3"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_arithmetic_expression() {
    let source = "
    fn main(p: i64) -> i64 {
        return (p - 2) * 4 / 2 % 3;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        0: v0 = Parameter
        1: MoveImm v1, 2
        2: v2 = Sub(v0, v1)
        3: MoveImm v3, 4
        4: v4 = Mul(v2, v3)
        5: MoveImm v5, 2
        6: v6 = Div(v4, v5)
        7: MoveImm v7, 3
        8: v8 = Mod(v6, v7)
        9: Return v8"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_conditional_branch_with_assign() {
    let source = "
    fn main(p: i64) -> i64 {
        let mut a: i64 = 0;
        if (p == 0) {
            a = 1;
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        0: v0 = Parameter
        1: MoveImm v2, 0
        2: Move v1, v2
        3: MoveImm v3, 0
        4: IfFalse v0 == v3, goto 7
        5: MoveImm v4, 1
        6: Move v1, v4
        7: ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_conditional_branch_with_returns() {
    let source = "
    fn main(p: i64) -> i64 {
        if (p == 0) {
            return 0;
        } else {
            return 1;
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        0: v0 = Parameter
        1: MoveImm v1, 0
        2: IfFalse v0 == v1, goto 6
        3: MoveImm v2, 0
        4: Return v2
        5: Goto 8
        6: MoveImm v3, 1
        7: Return v3
        8: ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_conditional_loop() {
    let source = "
    fn main(p: i64) {
        let mut a: i64 = 0;
        while (p == 0) {
            a = a + 1;
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Parameter, initialize variable "a" (v1 here)
    let mut expected = "
        0: v0 = Parameter
        1: MoveImm v2, 0
        2: Move v1, v2
        "
    .to_string();

    // Compare parameter with immediate 0 and enter the cycle
    expected = expected.clone()
        + "3: MoveImm v3, 0
        4: IfFalse v0 == v3, goto 9
        5: MoveImm v4, 1
        6: v5 = Add(v1, v4)
        7: Move v1, v5
        8: Goto 4
        ";

    // Return void
    expected = expected.clone() + "9: ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_infinite_loop() {
    let source = "
    fn main() {
        let mut a: i64 = 0;
        while (true) {
            a = a + 1;
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Initialize variable "a" (v0 here)
    let mut expected = "
        0: MoveImm v1, 0
        1: Move v0, v1
        "
    .to_string();

    // Infinite loop
    expected = expected.clone()
        + "2: MoveImm v2, 1
        3: v3 = Add(v0, v2)
        4: Move v0, v3
        5: Goto 2
        ";

    // Return void
    expected = expected.clone() + "6: ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_conditional_loop_with_break() {
    let source = "
    fn main() {
        let mut a: i64 = 0;
        while (a == 1) {
            a = a + 1;
            if (a == 4) {
                break;
            }
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Initialize variable "a" (v0 here)
    let mut expected = "
        0: MoveImm v1, 0
        1: Move v0, v1
        "
    .to_string();

    // Compare "a" with immediate 1 and enter the cycle
    expected = expected.clone()
        + "2: MoveImm v2, 1
        3: IfFalse v0 == v2, goto 11
        4: MoveImm v3, 1
        5: v4 = Add(v0, v3)
        6: Move v0, v4
        ";

    // Compare "a" with immediate 4 and break or go back at the begining of the block
    expected = expected.clone()
        + "7: MoveImm v5, 4
        8: IfFalse v0 == v5, goto 10
        9: Goto 11
        10: Goto 3
        ";

    // Return void
    expected = expected.clone() + "11: ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_infinite_loop_with_break() {
    let source = "
    fn main() -> i64 {
        let mut a: i64 = 0;
        while (true) {
            a = a + 1;
            if (a == 4) {
                break;
            }
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Initialize variable "a" (v0 here)
    let mut expected = "
        0: MoveImm v1, 0
        1: Move v0, v1
        "
    .to_string();

    // Loop's block
    expected = expected.clone()
        + "2: MoveImm v2, 1
        3: v3 = Add(v0, v2)
        4: Move v0, v3
        5: MoveImm v4, 4
        6: IfFalse v0 == v4, goto 8
        7: Goto 9
        8: Goto 2
        ";

    // Return void
    expected = expected.clone() + "9: ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_nested_infinite_loops() {
    let source = "
    fn main(p0: i64, p1: i64) {
        while (true) {
            if (p0 == 1) {
                break;
            }

            while (true) {
                if (p1 == 2) {
                    break;
                }
            }
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // 2 parameters
    let mut expected = "
        0: v0 = Parameter
        1: v1 = Parameter"
        .to_string();

    // Outer loop
    expected = expected.clone()
        + "
        2: MoveImm v2, 1
        3: IfFalse v0 == v2, goto 5
        4: Goto 10
        ";

    // Inner loop
    expected = expected.clone()
        + "5: MoveImm v3, 2
        6: IfFalse v1 == v3, goto 8
        7: Goto 9
        8: Goto 5
        ";

    // End of outer loop, return (implicit) void (end of the function)
    expected = expected.clone()
        + "9: Goto 2
        10: ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_call_no_arguments() {
    let source = "
    fn main() {
        print();
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let insts = generate_insts(&funcs[0]);
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        0: v0 = Call print, args: 
        1: ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}
