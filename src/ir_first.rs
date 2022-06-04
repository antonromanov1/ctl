//! First intermediate representation (IR)
//!
//! This module provides generating of first IR from AST.

use crate::parser::Func;
use crate::parser::Node;

use std::collections::HashMap;

// Condition code
#[derive(Debug, PartialEq)]
pub enum Cc {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    Invalid,
}

use std::fmt;

impl fmt::Display for Cc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::Le => write!(f, "<="),
            Self::Ge => write!(f, ">="),
            Self::Invalid => std::unreachable!(),
        }
    }
}

// Variable number
type Var = u16;

// Branch target which is an instructions number
type Target = u64;

// #[derive(Debug, PartialEq)]
pub enum Inst {
    Parameter(Var),

    // Move immediate value
    MoveImm(Var, i64),
    Move(Var, Var),

    // Arithmetic binary instructions.
    // Destination, source 1, source 2
    Add(Var, Var, Var),
    Sub(Var, Var, Var),
    Mul(Var, Var, Var),
    Div(Var, Var, Var),
    Mod(Var, Var, Var),

    // Negating instruction.
    // Destination, source
    Neg(Var, Var),

    // Shifts.
    // Destination, source 1, source 2
    Shl(Var, Var, Var),
    Shr(Var, Var, Var),

    IfFalse(Var, Var, Cc, Target),
    Goto(Target),
    Return(Var),
    ReturnVoid,

    Call(Option<Var>, String, Vec<Var>),
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inst::Parameter(var) => write!(f, "v{} = Parameter", var),
            Inst::MoveImm(dest, constant) => write!(f, "MoveImm v{}, {}", dest, constant),
            Inst::Move(dest, source) => write!(f, "Move v{}, v{}", dest, source),

            Inst::Add(dest, op1, op2) => write!(f, "v{} = Add(v{}, v{})", dest, op1, op2),
            Inst::Sub(dest, op1, op2) => write!(f, "v{} = Sub(v{}, v{})", dest, op1, op2),
            Inst::Mul(dest, op1, op2) => write!(f, "v{} = Mul(v{}, v{})", dest, op1, op2),
            Inst::Div(dest, op1, op2) => write!(f, "v{} = Div(v{}, v{})", dest, op1, op2),
            Inst::Mod(dest, op1, op2) => write!(f, "v{} = Mod(v{}, v{})", dest, op1, op2),

            Inst::Neg(dest, val) => write!(f, "v{} = Neg(v{})", dest, val),

            Inst::Shl(dest, op1, op2) => write!(f, "v{} = Shl(v{}, v{})", dest, op1, op2),
            Inst::Shr(dest, op1, op2) => write!(f, "v{} = Shr(v{}, v{})", dest, op1, op2),

            Inst::IfFalse(op1, op2, cc, target) => {
                write!(f, "IfFalse v{} {} v{}, goto {}", op1, cc, op2, target)
            }
            Inst::Goto(target) => write!(f, "Goto {}", target),
            Inst::Return(value) => write!(f, "Return v{}", value),
            Inst::ReturnVoid => write!(f, "ReturnVoid"),

            Inst::Call(dest, name, args) => {
                let mut s = String::new();
                if let Some(d) = dest {
                    s.push_str(&format!("v{} = Call {}, args: ", d, name));
                } else {
                    s.push_str(&format!("Call {}, args: ", name));
                }

                for (i, arg) in args.iter().enumerate() {
                    if i != args.len() - 1 {
                        s.push_str(&format!("v{}, ", arg));
                    } else {
                        s.push_str(&format!("v{}", arg));
                    }
                }

                write!(f, "{}", s)
            }
        }
    }
}

fn find_dest(variables: &HashMap<String, u16>, name: &String) -> u16 {
    let dest = variables.get(name);
    *dest.unwrap()
}

// Structure which is used during generating first IR.
// insts - already generated instructions
// vars  - map (variable name from AST -> variable number in the IR)
// count - number of variables
// breaks - vector of vectors of indexes (in `insts` vector) of Goto (break) instructions.
// cur_loop - index of first instruction of the currently handling loop.
struct Prepare {
    insts: Vec<Inst>,
    vars: HashMap<String, u16>,
    count: u16,
    breaks: Vec<Vec<usize>>,
    cur_loop: u64,
}

impl Prepare {
    fn new() -> Self {
        Self {
            insts: Vec::new(),
            vars: HashMap::new(),
            count: 0,
            breaks: Vec::new(),

            // Invalid value at the begining
            cur_loop: u64::MAX,
        }
    }
}

fn gen_and_check(expr: &Node, prep: &mut Prepare) -> u16 {
    let source = expr.generate(prep);
    if source.is_none() {
        println!("Variable for expression is not defined");
        std::process::exit(1);
    }
    source.unwrap()
}

fn gen_expr(expr: &Node, prep: &mut Prepare, dest: u16) {
    let source = gen_and_check(expr, prep);
    let move_inst = Inst::Move(dest, source);
    prep.insts.push(move_inst);
}

fn generate_let(prep: &mut Prepare, name: &String, expr: &Node) {
    let dest_or_none = prep.vars.get(name);

    let dest: u16;
    if let Some(d) = dest_or_none {
        dest = *d;
    } else {
        prep.vars.insert((*name).clone(), prep.count);
        dest = prep.count;
        prep.count += 1;
    }

    gen_expr(expr, prep, dest);
}

enum OpType {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
}

fn gen_arith_or_shift(prep: &mut Prepare, left: &Node, right: &Node, op: OpType) -> Option<u16> {
    let op1 = gen_and_check(left, prep);
    let op2 = gen_and_check(right, prep);
    let arith = match op {
        OpType::Add => Inst::Add(prep.count, op1, op2),
        OpType::Sub => Inst::Sub(prep.count, op1, op2),
        OpType::Mul => Inst::Mul(prep.count, op1, op2),
        OpType::Div => Inst::Div(prep.count, op1, op2),
        OpType::Mod => Inst::Mod(prep.count, op1, op2),
        OpType::Shl => Inst::Shl(prep.count, op1, op2),
        OpType::Shr => Inst::Shr(prep.count, op1, op2),
    };
    prep.count += 1;
    prep.insts.push(arith);

    Some(prep.count - 1)
}

fn gen_operand(prep: &mut Prepare, op: &Node) -> u16 {
    if let Node::Id(_name) = op {
        let option = op.generate(prep);
        return option.unwrap();
    }
    if let Node::Integer(_val) = op {
        let option = op.generate(prep);
        return option.unwrap();
    }

    std::unreachable!("Comparison operand can only be identifier or integer literal");
}

fn gen_operands_cc(prep: &mut Prepare, cond: &Node) -> (Var, Var, Cc) {
    let op1;
    let op2;

    match cond {
        Node::Eq(child1, child2) => {
            op1 = gen_operand(prep, child1);
            op2 = gen_operand(prep, child2);
            (op1, op2, Cc::Eq)
        }

        Node::Ne(child1, child2) => {
            op1 = gen_operand(prep, child1);
            op2 = gen_operand(prep, child2);
            (op1, op2, Cc::Ne)
        }

        Node::Le(child1, child2) => {
            op1 = gen_operand(prep, child1);
            op2 = gen_operand(prep, child2);
            (op1, op2, Cc::Le)
        }

        Node::Ge(child1, child2) => {
            op1 = gen_operand(prep, child1);
            op2 = gen_operand(prep, child2);
            (op1, op2, Cc::Ge)
        }

        Node::Lt(child1, child2) => {
            op1 = gen_operand(prep, child1);
            op2 = gen_operand(prep, child2);
            (op1, op2, Cc::Lt)
        }

        Node::Gt(child1, child2) => {
            op1 = gen_operand(prep, child1);
            op2 = gen_operand(prep, child2);
            (op1, op2, Cc::Gt)
        }
        _ => panic!("Expected eq, ne, le, ge, got {}", (*cond)),
    }
}

// Push partly set IfFalse instruction and get index of it
fn push_part_if_get_index(prep: &mut Prepare, op1: Var, op2: Var, cc: Cc) -> usize {
    let if_false = Inst::IfFalse(op1, op2, cc, u64::MAX);
    let if_index = prep.insts.len();
    prep.insts.push(if_false);
    if_index
}

// Target instruction of the branch is the instruction after the last instruction of the true
// successor block. Last instruction of the true successor block is Goto
//
// Example:
// if (condition) {
//     block
// } else {
//     alter block
// }
//
// Built IR:
// 0 IfFalse condition Goto 3
// 1 block
// 2 Goto 4
// 3 alter block
// 4 Instruction after the branching
fn generate_if(prep: &mut Prepare, cond: &Node, block: &Node, alter: &Option<Box<Node>>) {
    // (1) Generate operands of the comparison, compute the condition code
    let (op1, op2, cc) = gen_operands_cc(prep, cond);

    // (2) Create empty IfFalse instruction, add it to the vector and remember its position
    //     in order to write the target instruction later after generating instructions for the
    //     true successor block.
    let if_index = push_part_if_get_index(prep, op1, op2, cc);

    // (3) Generate IR instructions for the true successor block.
    block.generate(prep);

    // (4) Compute target IR instruction of this If Node. If there is a false successor then
    //     create a Goto and generate instructions for false successor.
    let mut if_target = prep.insts.len() as u64;
    if let Some(block_ptr) = alter {
        let goto = Inst::Goto(u64::MAX);
        let goto_index = prep.insts.len();
        prep.insts.push(goto);
        if_target += 1;

        (*block_ptr).generate(prep);
        let goto = Inst::Goto(prep.insts.len() as u64);
        prep.insts[goto_index] = goto;
    }

    // (5) Complete IfFalse instruction and write its target to remembered position in the vector.
    if let Inst::IfFalse(_, _, _, ref mut target) = prep.insts[if_index] {
        *target = if_target;
    } else {
        std::unreachable!("Instruction with index {} is not IfFalse", if_index);
    }
}

fn set_breaks(prep: &mut Prepare) {
    let after_last = prep.insts.len() as u64;
    for break_ in prep.breaks.last().unwrap().iter() {
        debug_assert!(matches!(prep.insts[*break_], Inst::Goto { .. }));
        prep.insts[*break_] = Inst::Goto(after_last);
    }
}

// Target instruction of the branch is the instruction after the last instruction of the
// block.
//
// Example:
// while (condition) {
//     block
// }
//
// Built IR:
// 0 IfFalse condition Goto 2
// 1 block
// goto 1
// 2 Next instruction
//
// Every element of Prepare's breaks vector is a vector of numbers of the
// generated Goto (Break) instructions. At the end of generating while cycle we have to go through
// the last vector and write target instructions (which is instruction after the last instruction)
// to these Goto's. At the begining of the generating while we push a new vector there and
// pop at the end.
fn generate_while(prep: &mut Prepare, cond: &Node, block: &Node) {
    // (1) Push vector of breaks for this cycle
    prep.breaks.push(Vec::new());

    // (2) Generate operands of the comparison, compute the condition code
    let (op1, op2, cc) = gen_operands_cc(prep, cond);

    // (3) Create IfFalse instruction with no target, add it to the vector and remember its position
    //     in order to write the target instruction later after generating instructions for the
    //     block. Remember previous loop position in `old_loop`. Set current loop position.
    let if_index = push_part_if_get_index(prep, op1, op2, cc);
    let old_loop = prep.cur_loop;
    prep.cur_loop = if_index as u64;

    // (4) Generate IR instructions for the block.
    block.generate(prep);

    // (5) Insert the goto on the begining of the block.
    let goto_begin = Inst::Goto(if_index as u64);
    prep.insts.push(goto_begin);

    // (6) Compute target instruction of the IfFalse instruction. Write it to the already created
    //     in step 2 IfFalse.
    let if_target = prep.insts.len();
    if let Inst::IfFalse(_, _, _, ref mut target) = prep.insts[if_index] {
        *target = if_target as u64;
    } else {
        std::unreachable!("Instruction with index {} is not IfFalse", if_index);
    }

    set_breaks(prep);

    // (7) Pop vector of breaks for this cycle
    let breaks = prep.breaks.pop();
    debug_assert!(matches!(breaks, Some { .. }));

    // (8) Save remembered previous loop
    prep.cur_loop = old_loop;
}

// Steps made in this function are described in function `generate_while` therefore these are not
// given here.
fn generate_infinite_loop(prep: &mut Prepare, block: &Node) {
    prep.breaks.push(Vec::new());

    let loop_begin = prep.insts.len() as u64;
    let old_loop = prep.cur_loop;
    prep.cur_loop = loop_begin;

    block.generate(prep);

    let goto = Inst::Goto(loop_begin);
    prep.insts.push(goto);

    set_breaks(prep);
    let breaks = prep.breaks.pop();
    debug_assert!(matches!(breaks, Some { .. }));

    // Save remembered previous loop
    prep.cur_loop = old_loop;
}

fn generate_call(
    prep: &mut Prepare,
    name: &String,
    arg_nodes: &[Node],
    is_subexpr: bool,
) -> Option<u16> {
    // Determine or create variables for the arguments
    let mut args = Vec::new();
    for node in (*arg_nodes).iter() {
        let ptr = Box::new((*node).clone());
        let arg = gen_and_check(&ptr, prep);
        args.push(arg);
    }

    if is_subexpr {
        let call = Inst::Call(Some(prep.count), (*name).clone(), args);
        prep.insts.push(call);
        prep.count += 1;
        Some(prep.count - 1)
    } else {
        let call = Inst::Call(None, (*name).clone(), args);
        prep.insts.push(call);
        None
    }
}

impl Node {
    fn generate(&self, prep: &mut Prepare) -> Option<u16> {
        // When we meet identifier we try to find it in the HashMap and extract from it the number
        // of the IR variable.
        if let Self::Id(name) = self {
            let var_num = prep.vars.get(name);
            return Some(*var_num.unwrap());
        }

        // Creates new variable, instruction MoveImm which writes num to this variable and returns
        // the variable number.
        if let Self::Integer(num) = self {
            let inst = Inst::MoveImm(prep.count, *num);
            prep.count += 1;
            prep.insts.push(inst);
            return Some(prep.count - 1);
        }

        if let Self::Let(name, expr) = self {
            generate_let(prep, name, expr);
            return None;
        }

        if let Self::Assign(name, expr) = self {
            let dest = find_dest(&prep.vars, name);
            gen_expr(expr, prep, dest);
            return None;
        }

        if let Self::Add(left, right) = self {
            let dest = gen_arith_or_shift(prep, left, right, OpType::Add);
            return dest;
        }

        if let Self::Sub(left, right) = self {
            let dest = gen_arith_or_shift(prep, left, right, OpType::Sub);
            return dest;
        }

        if let Self::Mul(left, right) = self {
            let dest = gen_arith_or_shift(prep, left, right, OpType::Mul);
            return dest;
        }

        if let Self::Div(left, right) = self {
            let dest = gen_arith_or_shift(prep, left, right, OpType::Div);
            return dest;
        }

        if let Self::Mod(left, right) = self {
            let dest = gen_arith_or_shift(prep, left, right, OpType::Mod);
            return dest;
        }

        if let Self::If(cond, block, alter) = self {
            generate_if(prep, cond, block, alter);
            return None;
        }

        if let Self::While(cond, block) = self {
            if let Node::True = **cond {
                generate_infinite_loop(prep, block);
            } else {
                generate_while(prep, cond, block);
            }

            return None;
        }

        if let Self::Break = self {
            debug_assert!(!prep.breaks.is_empty());
            let goto = Inst::Goto(u64::MAX);
            prep.breaks.last_mut().unwrap().push(prep.insts.len());
            prep.insts.push(goto);

            return None;
        }

        if let Self::Continue = self {
            let goto = Inst::Goto(prep.cur_loop);
            prep.insts.push(goto);

            return None;
        }

        if let Self::Block(nodes) = self {
            for node in &**nodes {
                node.generate(prep);
            }

            return None;
        }

        if let Self::Call(name, arg_nodes, is_subexpr) = self {
            let ret_var = generate_call(prep, name, arg_nodes, *is_subexpr);
            return ret_var;
        }

        if let Self::Return(val) = self {
            let var = gen_and_check(val, prep);
            let ret = Inst::Return(var);
            prep.insts.push(ret);
            return None;
        }

        if let Self::Neg(val) = self {
            let var = gen_and_check(val, prep);
            let neg = Inst::Neg(prep.count, var);
            prep.count += 1;
            prep.insts.push(neg);
            return Some(prep.count - 1);
        }

        if let Self::Shl(left, right) = self {
            let dest = gen_arith_or_shift(prep, left, right, OpType::Shl);
            return dest;
        }

        if let Self::Shr(left, right) = self {
            let dest = gen_arith_or_shift(prep, left, right, OpType::Shr);
            return dest;
        }

        if let Self::ReturnVoid = self {
            let return_void = Inst::ReturnVoid;
            prep.insts.push(return_void);
            return None;
        }

        std::unreachable!();
    }
}

// Main function on generating IR from AST. Generates vector of IR instructions.
pub fn generate_insts(func: &Func) -> Vec<Inst> {
    let mut prep = Prepare::new();

    // First instructions are the parameters of the function. Each parameter corresponds to an IR
    // variable.
    for param in func.get_params() {
        prep.vars.insert(param.clone(), prep.count);
        let inst = Inst::Parameter(prep.count);
        prep.count += 1;
        prep.insts.push(inst);
    }

    for stmt in func.get_stmts() {
        stmt.generate(&mut prep);
    }

    // Check does function have statements. It is needed in the next check on return.
    if func.get_stmts().is_empty() {
        return prep.insts;
    }

    // If in the AST the last statement is not Return than return is implicit and in IR we have it
    // explicit
    if let Node::Return(_expr) = func.get_stmts().last().unwrap() {
    } else {
        let ret = Inst::ReturnVoid;
        prep.insts.push(ret);
    }

    prep.insts
}
