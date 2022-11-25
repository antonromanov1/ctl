use ctl::ir::InstData;
use ctl::ir_builder::generate_ir;
use ctl::parser::parse;

fn dump(insts: &[InstData]) -> String {
    let mut res = String::new();
    for (i, inst) in insts.iter().enumerate() {
        res.push_str("\n        ");
        match inst {
            InstData::Store(_, _)
            | InstData::Goto(_)
            | InstData::IfFalse(_, _, _, _)
            | InstData::ReturnVoid
            | InstData::Return(_) => res.push_str(&format!(" {} {}", i, inst)),
            _ => res.push_str(&format!("%{} = {}", i, inst)),
        }
    }

    res
}

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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
         0 ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Parameter
        %1 = Parameter
         2 ReturnVoid"
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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Parameter
         1 Return %0"
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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Parameter
        %1 = Alloc
        %2 = Constant 1
         3 Store %2 at %1
        %4 = Load %1
        %5 = Add %0, %4
         6 Return %5"
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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Parameter
        %1 = Constant 2
        %2 = Sub %0, %1
        %3 = Constant 4
        %4 = Mul %2, %3
        %5 = Div %4, %1
        %6 = Constant 3
        %7 = Mod %5, %6
         8 Return %7"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_negate() {
    let source = "
    fn main() {
        let mut a: i64 = -1;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Alloc
        %1 = Constant 1
        %2 = Neg %1
         3 Store %2 at %0
         4 ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_shifts() {
    let source = "
    fn main() {
        let mut a: i64 = 1 << 2;
        let mut b: i64 = 1 >> 2;
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Alloc
        %1 = Constant 1
        %2 = Constant 2
        %3 = Shl %1, %2
         4 Store %3 at %0
        %5 = Alloc
        %6 = Shr %1, %2
         7 Store %6 at %5
         8 ReturnVoid"
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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Parameter
        %1 = Alloc
        %2 = Constant 0
         3 Store %2 at %1
         4 IfFalse %0 == %2, goto 7
        %5 = Constant 1
         6 Store %5 at %1
         7 ReturnVoid"
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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Parameter
        %1 = Constant 0
         2 IfFalse %0 == %1, goto 5
         3 Return %1
         4 Goto 7
        %5 = Constant 1
         6 Return %5
         7 ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

// TODO: eliminate calling clone, instead call push_str

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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Parameter, initialize variable "a"
    let mut expected = "
        %0 = Parameter
        %1 = Alloc
        %2 = Constant 0
         3 Store %2 at %1
        "
    .to_string();

    // Compare parameter with immediate 0 and enter the cycle
    expected = expected.clone()
        + " 4 IfFalse %0 == %2, goto 10
        %5 = Load %1
        %6 = Constant 1
        %7 = Add %5, %6
         8 Store %7 at %1
         9 Goto 4
        ";

    // Return void
    expected = expected.clone() + " 10 ReturnVoid";

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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Initialize variable "a"
    let mut expected = "
        %0 = Alloc
        %1 = Constant 0
         2 Store %1 at %0
        "
    .to_string();

    // Infinite loop
    expected = expected.clone()
        + "%3 = Load %0
        %4 = Constant 1
        %5 = Add %3, %4
         6 Store %5 at %0
         7 Goto 3
        ";

    // Return void
    expected = expected.clone() + " 8 ReturnVoid";

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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Initialize variable "a"
    let mut expected = "
        %0 = Alloc
        %1 = Constant 0
         2 Store %1 at %0
        "
    .to_string();

    // Compare "a" with immediate 1 and enter the cycle
    expected = expected.clone()
        + "%3 = Load %0
        %4 = Constant 1
         5 IfFalse %3 == %4, goto 14
        %6 = Load %0
        %7 = Add %6, %4
         8 Store %7 at %0
        ";

    // Compare "a" with immediate 4 if true we break else go to the end of the loop
    expected = expected.clone()
        + "%9 = Load %0
        %10 = Constant 4
         11 IfFalse %9 == %10, goto 13
         12 Goto 14
         13 Goto 3
        ";

    // Return void
    expected = expected.clone() + " 14 ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_conditional_loop_with_continues() {
    let source = "
    fn main() {
        let mut a: i64 = 0;

        while (a == 1) {
            a = a + 1;
            if (a == 4) {
                continue;
            }

            if (a == 3) {
                continue;
            }
        }
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // let mut a: i64 = 0;
    let mut expected = "
        %0 = Alloc
        %1 = Constant 0
         2 Store %1 at %0
        "
    .to_string();

    // while (a == 1) {
    //     a = a + 1;
    expected = expected.clone()
        + "%3 = Load %0
        %4 = Constant 1
         5 IfFalse %3 == %4, goto 18
        %6 = Load %0
        %7 = Add %6, %4
         8 Store %7 at %0
        ";

    // if (a == 4) {
    //     continue;
    // }
    expected = expected.clone()
        + "%9 = Load %0
        %10 = Constant 4
         11 IfFalse %9 == %10, goto 13
         12 Goto 3
        ";

    // if (a == 3) {
    //     continue;
    // }
    expected = expected.clone()
        + "%13 = Load %0
        %14 = Constant 3
         15 IfFalse %13 == %14, goto 17
         16 Goto 3
        ";

    // Go back at the begining of the loop's block
    expected = expected.clone()
        + " 17 Goto 3
        ";

    expected = expected.clone() + " 18 ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_conditional_nested_loops_with_continues() {
    let source = "
    fn main() {
        let mut a: i64 = 0;
        let mut b: i64 = 128;

        while (a < 8) {
            a = a + 1;

            if (a == 3) {
                continue;
            }

            while (b > 0) {
                b = b - 1;

                if (b == 4) {
                    continue;
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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // let mut a: i64 = 0;
    // let mut b: i64 = 128;
    let mut expected = "
        %0 = Alloc
        %1 = Constant 0
         2 Store %1 at %0
        %3 = Alloc
        %4 = Constant 128
         5 Store %4 at %3
        "
    .to_string();

    // while (a < 8) {
    //     a = a + 1;
    expected.push_str(
        "%6 = Load %0
        %7 = Constant 8
         8 IfFalse %6 < %7, goto 28
        %9 = Load %0
        %10 = Constant 1
        %11 = Add %9, %10
         12 Store %11 at %0
        ",
    );

    // if (a == 3) {
    //     continue;
    // }
    expected.push_str(
        "%13 = Load %0
        %14 = Constant 3
         15 IfFalse %13 == %14, goto 17
         16 Goto 6
        ",
    );

    // Inner loop
    // while (b > 0) {
    //     b = b - 1;
    expected.push_str(
        "%17 = Load %3
         18 IfFalse %17 > %1, goto 27
        %19 = Load %3
        %20 = Sub %19, %10
         21 Store %20 at %3
        ",
    );

    // if (b == 4) {
    //     continue;
    // }
    expected.push_str(
        "%22 = Load %3
        %23 = Constant 4
         24 IfFalse %22 == %23, goto 26
         25 Goto 17
        ",
    );

    // Go back to the comparison of the inner loop
    expected.push_str(
        " 26 Goto 17
        ",
    );

    // Go back to the comparison of the outer loop
    expected.push_str(
        " 27 Goto 6
        ",
    );

    expected.push_str(" 28 ReturnVoid");

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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // Initialize variable "a"
    let mut expected = "
        %0 = Alloc
        %1 = Constant 0
         2 Store %1 at %0
        "
    .to_string();

    // Loop's block
    expected = expected.clone()
        + "%3 = Load %0
        %4 = Constant 1
        %5 = Add %3, %4
         6 Store %5 at %0
        %7 = Load %0
        %8 = Constant 4
         9 IfFalse %7 == %8, goto 11
         10 Goto 12
         11 Goto 3
        ";

    // Return void
    expected = expected.clone() + " 12 ReturnVoid";

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
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump

    // 2 parameters
    let mut expected = "
        %0 = Parameter
        %1 = Parameter
        "
    .to_string();

    // Begining of outer loop
    expected = expected.clone()
        + "%2 = Constant 1
         3 IfFalse %0 == %2, goto 5
         4 Goto 10
        ";

    // Inner loop
    expected = expected.clone()
        + "%5 = Constant 2
         6 IfFalse %1 == %5, goto 8
         7 Goto 9
         8 Goto 5
        ";

    // End of outer loop, return (implicit) void (end of the function)
    expected = expected.clone()
        + " 9 Goto 2
         10 ReturnVoid";

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_call_no_arguments() {
    let source = "
    fn main() {
        print(0);
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 1);

    // Generate IR instructions
    let func = generate_ir(&funcs[0]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Constant 0
        %1 = Call print, args: %0
         2 ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_call_few_arguments() {
    let source = "
    fn abc(p1: i64, p2: i64) {}

    fn main() {
        abc(1, 2);
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 2);

    // Generate IR instructions
    let func = generate_ir(&funcs[1]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Constant 1
        %1 = Constant 2
        %2 = Call abc, args: %0, %1
         3 ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}

#[test]
fn generate_call_as_expression() {
    let source = "
    fn calc(param: i64) -> i64 {}

    fn main() {
        let mut num: i64 = calc(0);
    }
    "
    .to_string();

    // Parse source into the AST nodes
    let funcs = parse(source).unwrap();
    assert_eq!(funcs.len(), 2);

    // Generate IR instructions
    let func = generate_ir(&funcs[1]);
    let insts = func.insts();
    assert!(!insts.is_empty());

    // Dump these to a string
    let dump = dump(insts);

    // Create expected dump
    let expected = "
        %0 = Alloc
        %1 = Constant 0
        %2 = Call calc, args: %1
         3 Store %2 at %0
         4 ReturnVoid"
        .to_string();

    // Compare generated instructions with the expected ones
    assert_eq!(dump, expected);
}
