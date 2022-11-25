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
struct IrBuilder {
    insts: Vec<InstData>,
    constants: HashMap<Value, InstId>,
    vars: HashMap<String, InstId>,
    breaks: Vec<Vec<InstId>>,
    cur_loop: InstId,
}

impl IrBuilder {
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

impl IrBuilder {
    fn gen_and_check(&mut self, expr: &Node) -> InstId {
        let source = self.generate(expr);
        if source.is_none() {
            println!("Variable for expression is not defined");
            std::process::exit(1);
        }
        source.unwrap()
    }

    fn gen_value_assign(&mut self, expr: &Node, dest: InstId) {
        let source = self.gen_and_check(expr);
        let move_inst = InstData::Store(source, dest);
        self.insts.push(move_inst);
    }

    fn generate_let(&mut self, name: &String, expr: &Node) {
        assert_eq!(self.vars.get(name), None);

        let id = InstId(self.insts.len());
        self.vars.insert((*name).clone(), id);
        self.insts.push(InstData::Alloc);

        self.gen_value_assign(expr, id);
    }
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

impl IrBuilder {
    fn gen_arith_or_shift(&mut self, left: &Node, right: &Node, op: OpType) -> Option<InstId> {
        let op1 = self.gen_and_check(left);
        let op2 = self.gen_and_check(right);
        let arith = match op {
            OpType::Add => InstData::Add(op1, op2),
            OpType::Sub => InstData::Sub(op1, op2),
            OpType::Mul => InstData::Mul(op1, op2),
            OpType::Div => InstData::Div(op1, op2),
            OpType::Mod => InstData::Mod(op1, op2),
            OpType::Shl => InstData::Shl(op1, op2),
            OpType::Shr => InstData::Shr(op1, op2),
        };
        self.insts.push(arith);

        Some(InstId(self.insts.len() - 1))
    }

    fn gen_operand(&mut self, op: &Node) -> InstId {
        if let Node::Id(_name) = op {
            let option = self.generate(op);
            return option.unwrap();
        }
        if let Node::Integer(_val) = op {
            let option = self.generate(op);
            return option.unwrap();
        }

        std::unreachable!("Comparison operand can only be identifier or integer literal");
    }

    fn gen_operands_cc(&mut self, cond: &Node) -> (InstId, InstId, Cc) {
        let op1;
        let op2;

        match cond {
            Node::Eq(child1, child2) => {
                op1 = self.gen_operand(child1);
                op2 = self.gen_operand(child2);
                (op1, op2, Cc::Eq)
            }

            Node::Ne(child1, child2) => {
                op1 = self.gen_operand(child1);
                op2 = self.gen_operand(child2);
                (op1, op2, Cc::Ne)
            }

            Node::Le(child1, child2) => {
                op1 = self.gen_operand(child1);
                op2 = self.gen_operand(child2);
                (op1, op2, Cc::Le)
            }

            Node::Ge(child1, child2) => {
                op1 = self.gen_operand(child1);
                op2 = self.gen_operand(child2);
                (op1, op2, Cc::Ge)
            }

            Node::Lt(child1, child2) => {
                op1 = self.gen_operand(child1);
                op2 = self.gen_operand(child2);
                (op1, op2, Cc::Lt)
            }

            Node::Gt(child1, child2) => {
                op1 = self.gen_operand(child1);
                op2 = self.gen_operand(child2);
                (op1, op2, Cc::Gt)
            }
            _ => panic!("Expected eq, ne, le, ge, got {}", (*cond)),
        }
    }
}

/// Generating IR for the control flow AST nodes
impl IrBuilder {
    // Push partly set IfFalse instruction and get index of it
    fn push_part_if_get_index(&mut self, op1: InstId, op2: InstId, cc: Cc) -> InstId {
        let if_id = InstId(self.insts.len());
        let if_false = InstData::IfFalse(op1, op2, cc, Default::default());
        self.insts.push(if_false);

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
    fn generate_if(&mut self, cond: &Node, block: &Node, alter: &Option<Box<Node>>) {
        // (1) Generate operands of the comparison, compute the condition code
        let (op1, op2, cc) = self.gen_operands_cc(cond);

        // (2) Create empty IfFalse instruction, add it to the vector and remember its position
        //     in order to write the target instruction later after generating instructions for the
        //     true successor block.
        let if_index = self.push_part_if_get_index(op1, op2, cc);

        // (3) Generate IR instructions for the true successor block.
        self.generate(block);

        // (4) Compute target IR instruction of this If Node. If there is a false successor then
        //     create a Goto and generate instructions for false successor.
        let mut if_target = InstId(self.insts.len());
        if let Some(block_ptr) = alter {
            let goto = InstData::Goto(Default::default());
            let goto_index = self.insts.len();
            self.insts.push(goto);
            if_target.0 += 1;

            self.generate(&**block_ptr);
            let goto = InstData::Goto(InstId(self.insts.len()));
            self.insts[goto_index] = goto;
        }

        // (5) Complete IfFalse instruction and write its target to remembered position in the vector.
        if let InstData::IfFalse(_, _, _, ref mut target) = self.insts[if_index.0] {
            *target = if_target;
        } else {
            std::unreachable!("Instruction with index {} is not IfFalse", if_index);
        }
    }

    fn set_breaks(&mut self) {
        let after_last = InstId(self.insts.len());
        for break_id in self.breaks.last().unwrap().iter() {
            debug_assert!(matches!(self.insts[(*break_id).0], InstData::Goto { .. }));
            self.insts[(*break_id).0] = InstData::Goto(after_last);
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
    // Every element of IrBuilder's breaks vector is a vector of numbers of the
    // generated Goto (Break) instructions. At the end of generating while cycle we have to go through
    // the last vector and write target instructions (which is instruction after the last instruction)
    // to these Goto's. At the begining of the generating while we push a new vector there and
    // pop at the end.
    fn generate_while(&mut self, cond: &Node, block: &Node) {
        // (1) Push vector of breaks for this cycle
        self.breaks.push(Vec::new());

        // (2) Generate operands of the comparison, compute the condition code
        let (op1, op2, cc) = self.gen_operands_cc(cond);

        // (3) Create IfFalse instruction with no target, add it to the vector and remember its position
        //     in order to write the target instruction later after generating instructions for the
        //     block. Remember previous loop position in `old_loop`. Set current loop position.
        let if_index = self.push_part_if_get_index(op1, op2, cc);
        let old_loop = self.cur_loop;

        // (4) Determine the begining of the loop
        let begin: InstId;
        if let InstData::Load(_) = self.insts[op1.0] {
            begin = op1;
        } else if let InstData::Load(_) = self.insts[op2.0] {
            begin = op2;
        } else {
            begin = if_index;
        }

        self.cur_loop = begin;

        // (5) Generate IR instructions for the block.
        self.generate(block);

        // (6) Insert the goto on the begining of the block.
        let goto_begin = InstData::Goto(begin);
        self.insts.push(goto_begin);

        // (7) Compute target instruction of the IfFalse instruction. Write it to the already created
        //     in step 2 IfFalse.
        let if_target = InstId(self.insts.len());
        if let InstData::IfFalse(_, _, _, ref mut target) = self.insts[if_index.0] {
            *target = if_target;
        } else {
            std::unreachable!("Instruction with index {} is not IfFalse", if_index);
        }

        self.set_breaks();

        // (8) Pop vector of breaks for this cycle
        let breaks = self.breaks.pop();
        debug_assert!(matches!(breaks, Some { .. }));

        // (9) Save remembered previous loop
        self.cur_loop = old_loop;
    }

    // Steps made in this function (except determining begining of the loop) are described in function
    // `generate_while` therefore these are not given here.
    fn generate_infinite_loop(&mut self, block: &Node) {
        self.breaks.push(Vec::new());

        let loop_begin = InstId(self.insts.len());
        let old_loop = self.cur_loop;
        self.cur_loop = loop_begin;

        self.generate(block);

        let goto = InstData::Goto(loop_begin);
        self.insts.push(goto);

        self.set_breaks();
        let breaks = self.breaks.pop();
        debug_assert!(matches!(breaks, Some { .. }));

        // Save remembered previous loop
        self.cur_loop = old_loop;
    }
}

impl IrBuilder {
    fn generate_call(&mut self, name: &String, arg_nodes: &[Node]) -> Option<InstId> {
        // Determine or create variables for the arguments
        let mut args = Vec::new();
        for node in (*arg_nodes).iter() {
            let ptr = Box::new((*node).clone());
            let arg = self.gen_and_check(&ptr);
            args.push(arg);
        }

        let call = InstData::Call((*name).clone(), args);
        self.insts.push(call);

        Some(InstId(self.insts.len() - 1))
    }
}

impl IrBuilder {
    /// Takes an AST node, checks its type and generates the IR
    fn generate(&mut self, node: &Node) -> Option<InstId> {
        // When we meet identifier we try to find it in the HashMap and extract from it the number
        // of the IR variable.
        if let Node::Id(name) = node {
            let var_num = *self.vars.get(name).unwrap();
            if let InstData::Parameter = self.insts[var_num.0] {
                return Some(var_num);
            }

            self.insts.push(InstData::Load(var_num));

            return Some(InstId(self.insts.len() - 1));
        }

        // Creates new variable, instruction MoveImm which writes num to this variable and returns
        // the variable number.
        if let Node::Integer(num) = node {
            return Some(self.find_or_create_constant(*num));
        }

        if let Node::Let(name, expr) = node {
            self.generate_let(name, expr);
            return None;
        }

        if let Node::Assign(name, expr) = node {
            let dest = *self.vars.get(name).unwrap();
            self.gen_value_assign(expr, dest);
            return None;
        }

        if let Node::Add(left, right) = node {
            let dest = self.gen_arith_or_shift(left, right, OpType::Add);
            return dest;
        }

        if let Node::Sub(left, right) = node {
            let dest = self.gen_arith_or_shift(left, right, OpType::Sub);
            return dest;
        }

        if let Node::Mul(left, right) = node {
            let dest = self.gen_arith_or_shift(left, right, OpType::Mul);
            return dest;
        }

        if let Node::Div(left, right) = node {
            let dest = self.gen_arith_or_shift(left, right, OpType::Div);
            return dest;
        }

        if let Node::Mod(left, right) = node {
            let dest = self.gen_arith_or_shift(left, right, OpType::Mod);
            return dest;
        }

        if let Node::If(cond, block, alter) = node {
            self.generate_if(cond, block, alter);
            return None;
        }

        if let Node::While(cond, block) = node {
            if let Node::True = **cond {
                self.generate_infinite_loop(block);
            } else if let Node::False = **cond {
                self.generate(block);
            } else {
                self.generate_while(cond, block);
            }

            return None;
        }

        if let Node::Break = node {
            debug_assert!(!self.breaks.is_empty());
            let goto = InstData::Goto(Default::default());
            self.breaks
                .last_mut()
                .unwrap()
                .push(InstId(self.insts.len()));
            self.insts.push(goto);

            return None;
        }

        if let Node::Continue = node {
            let goto = InstData::Goto(self.cur_loop);
            self.insts.push(goto);

            return None;
        }

        if let Node::Block(nodes) = node {
            for n in &**nodes {
                self.generate(n);
            }

            return None;
        }

        if let Node::Call(name, arg_nodes, _) = node {
            let ret_var = self.generate_call(name, arg_nodes);
            return ret_var;
        }

        if let Node::Return(val) = node {
            let var = self.gen_and_check(val);
            let ret = InstData::Return(var);
            self.insts.push(ret);
            return None;
        }

        if let Node::Neg(val) = node {
            let var = self.gen_and_check(val);
            let neg = InstData::Neg(var);
            self.insts.push(neg);
            return Some(InstId(self.insts.len() - 1));
        }

        if let Node::Shl(left, right) = node {
            let dest = self.gen_arith_or_shift(left, right, OpType::Shl);
            return dest;
        }

        if let Node::Shr(left, right) = node {
            let dest = self.gen_arith_or_shift(left, right, OpType::Shr);
            return dest;
        }

        if let Node::ReturnVoid = node {
            let return_void = InstData::ReturnVoid;
            self.insts.push(return_void);
            return None;
        }

        std::unreachable!();
    }
}

// Main function on generating IR from AST. Generates vector of IR instructions and (a hash map with
// constants).
pub fn generate_ir(func: &parser::Func) -> ir::Function {
    let mut builder = IrBuilder::new();

    // First instructions are the parameters of the function. Each parameter corresponds to an IR
    // variable.
    for param in func.params() {
        builder
            .vars
            .insert(param.clone(), InstId(builder.insts.len()));
        let inst = InstData::Parameter;
        builder.insts.push(inst);
    }

    for stmt in func.stmts() {
        builder.generate(stmt);
    }

    let ret = InstData::ReturnVoid;

    // Check does function have statements. It is needed in the next check on return.
    if func.stmts().is_empty() {
        builder.insts.push(ret);
        return ir::Function::new(builder.insts, builder.constants);
    }

    // If in the AST the last statement is not Return than return is implicit and in IR we have it
    // explicit
    let last = func.stmts().last().unwrap();
    if !(matches!(last, Node::Return { .. }) || matches!(last, Node::ReturnVoid)) {
        builder.insts.push(ret);
    }

    ir::Function::new(builder.insts, builder.constants)
}
