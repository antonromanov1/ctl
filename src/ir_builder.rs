//! This module provides generating of intermediate representation from abstract
//! syntax tree

use crate::ir;
use crate::ir::Cc;
use crate::ir::InstData;
use crate::ir::InstId;
use crate::ir::Value;

use crate::parser;
use crate::parser::Node;

use std::collections::HashMap;

// Structure which is used during generating IR.
// insts - already generated instructions
// vars  - map (variable name from AST -> instruction number in the IR)
// breaks - vector of vectors of indexes (in `insts` vector) of Goto (break) instructions.
// cur_loop - index of first instruction of the currently handling loop.
struct Prepare {
    insts: Vec<InstData>,
    constants: HashMap<Value, InstId>,
    vars: HashMap<String, InstId>,
    breaks: Vec<Vec<InstId>>,
    cur_loop: InstId,
}

impl Prepare {
    fn new() -> Self {
        Self {
            insts: Vec::new(),
            constants: HashMap::new(),
            vars: HashMap::new(),
            breaks: Vec::new(),

            // Invalid value at the begining
            cur_loop: Default::default(),
        }
    }

    fn find_or_create_constant(&mut self, value: i64) -> InstId {
        if let Some(inst) = self.constants.get(&value) {
            return *inst;
        }

        let inst = InstData::Constant(value);
        self.insts.push(inst);
        let inst_num = InstId(self.insts.len() - 1);
        self.constants.insert(value, inst_num);

        inst_num
    }
}

fn gen_and_check(expr: &Node, prep: &mut Prepare) -> InstId {
    let source = expr.generate(prep);
    if source.is_none() {
        println!("Variable for expression is not defined");
        std::process::exit(1);
    }
    source.unwrap()
}

fn gen_value_assign(expr: &Node, prep: &mut Prepare, dest: InstId) {
    let source = gen_and_check(expr, prep);
    let move_inst = InstData::Store(source, dest);
    prep.insts.push(move_inst);
}

fn generate_let(prep: &mut Prepare, name: &String, expr: &Node) {
    assert_eq!(prep.vars.get(name), None);

    let id = InstId(prep.insts.len());
    prep.vars.insert((*name).clone(), id);
    prep.insts.push(InstData::Alloc);

    gen_value_assign(expr, prep, id);
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

fn gen_arith_or_shift(prep: &mut Prepare, left: &Node, right: &Node, op: OpType) -> Option<InstId> {
    let op1 = gen_and_check(left, prep);
    let op2 = gen_and_check(right, prep);
    let arith = match op {
        OpType::Add => InstData::Add(op1, op2),
        OpType::Sub => InstData::Sub(op1, op2),
        OpType::Mul => InstData::Mul(op1, op2),
        OpType::Div => InstData::Div(op1, op2),
        OpType::Mod => InstData::Mod(op1, op2),
        OpType::Shl => InstData::Shl(op1, op2),
        OpType::Shr => InstData::Shr(op1, op2),
    };
    prep.insts.push(arith);

    Some(InstId(prep.insts.len() - 1))
}

fn gen_operand(prep: &mut Prepare, op: &Node) -> InstId {
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

fn gen_operands_cc(prep: &mut Prepare, cond: &Node) -> (InstId, InstId, Cc) {
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
fn push_part_if_get_index(prep: &mut Prepare, op1: InstId, op2: InstId, cc: Cc) -> InstId {
    let if_id = InstId(prep.insts.len());
    let if_false = InstData::IfFalse(op1, op2, cc, Default::default());
    prep.insts.push(if_false);

    if_id
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
    let mut if_target = InstId(prep.insts.len());
    if let Some(block_ptr) = alter {
        let goto = InstData::Goto(Default::default());
        let goto_index = prep.insts.len();
        prep.insts.push(goto);
        if_target.0 += 1;

        (*block_ptr).generate(prep);
        let goto = InstData::Goto(InstId(prep.insts.len()));
        prep.insts[goto_index] = goto;
    }

    // (5) Complete IfFalse instruction and write its target to remembered position in the vector.
    if let InstData::IfFalse(_, _, _, ref mut target) = prep.insts[if_index.0] {
        *target = if_target;
    } else {
        std::unreachable!("Instruction with index {} is not IfFalse", if_index);
    }
}

fn set_breaks(prep: &mut Prepare) {
    let after_last = InstId(prep.insts.len());
    for break_id in prep.breaks.last().unwrap().iter() {
        debug_assert!(matches!(prep.insts[(*break_id).0], InstData::Goto { .. }));
        prep.insts[(*break_id).0] = InstData::Goto(after_last);
    }
}

// Target instruction of the branch is the instruction after the last instruction of the
// block.
//
// Example:
// a = Load
// b = Load
// while (a cmp b) {
//     block
// }
//
// Built IR:
// -2 Load
// -1 Load
// 0 IfFalse condition Goto 2
// 1 block
// goto -2
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

    // (4) Determine the begining of the loop
    let begin: InstId;
    if let InstData::Load(_) = prep.insts[op1.0] {
        begin = op1;
    } else if let InstData::Load(_) = prep.insts[op2.0] {
        begin = op2;
    } else {
        begin = if_index;
    }

    prep.cur_loop = begin;

    // (5) Generate IR instructions for the block.
    block.generate(prep);

    // (6) Insert the goto on the begining of the block.
    let goto_begin = InstData::Goto(begin);
    prep.insts.push(goto_begin);

    // (7) Compute target instruction of the IfFalse instruction. Write it to the already created
    //     in step 2 IfFalse.
    let if_target = InstId(prep.insts.len());
    if let InstData::IfFalse(_, _, _, ref mut target) = prep.insts[if_index.0] {
        *target = if_target;
    } else {
        std::unreachable!("Instruction with index {} is not IfFalse", if_index);
    }

    set_breaks(prep);

    // (8) Pop vector of breaks for this cycle
    let breaks = prep.breaks.pop();
    debug_assert!(matches!(breaks, Some { .. }));

    // (9) Save remembered previous loop
    prep.cur_loop = old_loop;
}

// Steps made in this function (except determining begining of the loop) are described in function
// `generate_while` therefore these are not given here.
fn generate_infinite_loop(prep: &mut Prepare, block: &Node) {
    prep.breaks.push(Vec::new());

    let loop_begin = InstId(prep.insts.len());
    let old_loop = prep.cur_loop;
    prep.cur_loop = loop_begin;

    block.generate(prep);

    let goto = InstData::Goto(loop_begin);
    prep.insts.push(goto);

    set_breaks(prep);
    let breaks = prep.breaks.pop();
    debug_assert!(matches!(breaks, Some { .. }));

    // Save remembered previous loop
    prep.cur_loop = old_loop;
}

fn generate_call(prep: &mut Prepare, name: &String, arg_nodes: &[Node]) -> Option<InstId> {
    // Determine or create variables for the arguments
    let mut args = Vec::new();
    for node in (*arg_nodes).iter() {
        let ptr = Box::new((*node).clone());
        let arg = gen_and_check(&ptr, prep);
        args.push(arg);
    }

    let call = InstData::Call((*name).clone(), args);
    prep.insts.push(call);

    Some(InstId(prep.insts.len() - 1))
}

impl Node {
    fn generate(&self, prep: &mut Prepare) -> Option<InstId> {
        // When we meet identifier we try to find it in the HashMap and extract from it the number
        // of the IR variable.
        if let Self::Id(name) = self {
            let var_num = *prep.vars.get(name).unwrap();
            if let InstData::Parameter = prep.insts[var_num.0] {
                return Some(var_num);
            }

            prep.insts.push(InstData::Load(var_num));

            return Some(InstId(prep.insts.len() - 1));
        }

        // Creates new variable, instruction MoveImm which writes num to this variable and returns
        // the variable number.
        if let Self::Integer(num) = self {
            return Some(prep.find_or_create_constant(*num));
        }

        if let Self::Let(name, expr) = self {
            generate_let(prep, name, expr);
            return None;
        }

        if let Self::Assign(name, expr) = self {
            let dest = *prep.vars.get(name).unwrap();
            gen_value_assign(expr, prep, dest);
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
            } else if let Node::False = **cond {
                block.generate(prep);
            } else {
                generate_while(prep, cond, block);
            }

            return None;
        }

        if let Self::Break = self {
            debug_assert!(!prep.breaks.is_empty());
            let goto = InstData::Goto(Default::default());
            prep.breaks
                .last_mut()
                .unwrap()
                .push(InstId(prep.insts.len()));
            prep.insts.push(goto);

            return None;
        }

        if let Self::Continue = self {
            let goto = InstData::Goto(prep.cur_loop);
            prep.insts.push(goto);

            return None;
        }

        if let Self::Block(nodes) = self {
            for node in &**nodes {
                node.generate(prep);
            }

            return None;
        }

        if let Self::Call(name, arg_nodes, _) = self {
            let ret_var = generate_call(prep, name, arg_nodes);
            return ret_var;
        }

        if let Self::Return(val) = self {
            let var = gen_and_check(val, prep);
            let ret = InstData::Return(var);
            prep.insts.push(ret);
            return None;
        }

        if let Self::Neg(val) = self {
            let var = gen_and_check(val, prep);
            let neg = InstData::Neg(var);
            prep.insts.push(neg);
            return Some(InstId(prep.insts.len() - 1));
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
            let return_void = InstData::ReturnVoid;
            prep.insts.push(return_void);
            return None;
        }

        std::unreachable!();
    }
}

// Main function on generating IR from AST. Generates vector of IR instructions and (a hash map with
// constants).
pub fn generate_ir(func: &parser::Func) -> ir::Function {
    let mut prep = Prepare::new();

    // First instructions are the parameters of the function. Each parameter corresponds to an IR
    // variable.
    for param in func.params() {
        prep.vars.insert(param.clone(), InstId(prep.insts.len()));
        let inst = InstData::Parameter;
        prep.insts.push(inst);
    }

    for stmt in func.stmts() {
        stmt.generate(&mut prep);
    }

    let ret = InstData::ReturnVoid;

    // Check does function have statements. It is needed in the next check on return.
    if func.stmts().is_empty() {
        prep.insts.push(ret);
        return ir::Function::new(prep.insts, prep.constants);
    }

    // If in the AST the last statement is not Return than return is implicit and in IR we have it
    // explicit
    let last = func.stmts().last().unwrap();
    if !(matches!(last, Node::Return { .. }) || matches!(last, Node::ReturnVoid)) {
        prep.insts.push(ret);
    }

    ir::Function::new(prep.insts, prep.constants)
}
