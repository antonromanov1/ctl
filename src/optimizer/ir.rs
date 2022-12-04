//! Intermediate representation (IR)

// Condition code
#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InstId(pub usize);

impl fmt::Display for InstId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Operand = InstId;
type Dest = InstId;
// Branch target
type Target = InstId;
pub type Value = i64;

// #[derive(Debug, PartialEq)]
pub enum InstData {
    Constant(Value),
    Parameter,

    Alloc,
    Store(InstId, Dest),
    Load(InstId),

    // Binary instructions
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    Div(Operand, Operand),
    Mod(Operand, Operand),
    Shl(Operand, Operand),
    Shr(Operand, Operand),

    Neg(Operand),

    Return(InstId),
    ReturnVoid,

    Call(String, Vec<InstId>),

    // Control flow instructions used during translation from AST to linear IR
    // (inst_builder module). `Target`s are instructions to which control is
    // transferred.
    IfFalse(Operand, Operand, Cc, Target),
    Goto(Target),

    // Control flow instructions which are built during translation from linear
    // IR to control flow graph with instructions in the basic blocks. Targets
    // are placed as the successors of each BasicBlock.
    Branch(Operand, Operand, Cc),
    Jump,
}

impl InstData {
    pub fn target(&self) -> Option<InstId> {
        match self {
            Self::IfFalse(_, _, _, target) | Self::Goto(target) => Some(*target),
            _ => None,
        }
    }

    pub fn set_target(&mut self, new_target: Target) {
        match self {
            Self::IfFalse(_, _, _, ref mut target) | Self::Goto(ref mut target) => {
                *target = new_target
            }
            _ => std::unreachable!(),
        }
    }
}

impl fmt::Display for InstData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InstData::Constant(value) => write!(f, "Constant {}", value),
            InstData::Parameter => write!(f, "Parameter"),
            InstData::Alloc => write!(f, "Alloc"),
            InstData::Store(src, dest) => write!(f, "Store %{} at %{}", src, dest),
            InstData::Load(op) => write!(f, "Load %{}", op),

            InstData::Add(op1, op2) => write!(f, "Add %{}, %{}", op1, op2),
            InstData::Sub(op1, op2) => write!(f, "Sub %{}, %{}", op1, op2),
            InstData::Mul(op1, op2) => write!(f, "Mul %{}, %{}", op1, op2),
            InstData::Div(op1, op2) => write!(f, "Div %{}, %{}", op1, op2),
            InstData::Mod(op1, op2) => write!(f, "Mod %{}, %{}", op1, op2),
            InstData::Shl(op1, op2) => write!(f, "Shl %{}, %{}", op1, op2),
            InstData::Shr(op1, op2) => write!(f, "Shr %{}, %{}", op1, op2),

            InstData::Neg(op) => write!(f, "Neg %{}", op),

            InstData::Return(value) => write!(f, "Return %{}", value),
            InstData::ReturnVoid => write!(f, "ReturnVoid"),

            InstData::Call(name, args) => {
                write!(f, "Call {}, args: ", name)?;

                for (i, arg) in args.iter().enumerate() {
                    if i != args.len() - 1 {
                        write!(f, "%{}, ", arg)?;
                    } else {
                        write!(f, "%{}", arg)?;
                    }
                }

                Ok(())
            }

            InstData::IfFalse(op1, op2, cc, target) => {
                write!(f, "IfFalse %{} {} %{}, goto {}", op1, cc, op2, target)
            }
            InstData::Goto(target) => write!(f, "Goto {}", target),

            InstData::Branch(op1, op2, cc) => {
                write!(f, "Branch %{} {} %{}", op1, cc, op2)
            }
            InstData::Jump => write!(f, "Jump"),
        }
    }
}

impl InstData {
    pub fn dump(&self, id: InstId) -> String {
        match self {
            // Instructions which don't produce a value
            InstData::Store(_, _)
            | InstData::Goto(_)
            | InstData::IfFalse(_, _, _, _)
            | InstData::Jump
            | InstData::Branch(_, _, _)
            | InstData::ReturnVoid
            | InstData::Return(_) => format!(" {} {}", id, self),

            _ => format!("%{} = {}", id, self),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BlockId(pub usize);

pub struct InstNode {
    block: Option<BlockId>,
    next: Option<InstId>,
}

impl InstNode {
    fn new() -> Self {
        Self {
            block: None,
            next: None,
        }
    }

    pub fn block(&self) -> BlockId {
        self.block.unwrap()
    }
}

pub struct BasicBlock {
    // Predecessors and successors
    preds: Vec<BlockId>,
    succs: Vec<BlockId>,

    // First and last instructions
    first: Option<InstId>,
    last: Option<InstId>,
}

impl BasicBlock {
    fn new() -> Self {
        Self {
            preds: Vec::new(),
            succs: Vec::new(),
            first: None,
            last: None,
        }
    }

    pub fn add_pred(&mut self, pred: BlockId) {
        self.preds.push(pred);
    }

    pub fn add_succ(&mut self, succ: BlockId) {
        self.succs.push(succ);
    }

    pub fn last(&self) -> &Option<InstId> {
        &self.last
    }

    fn dump_preds(&self) -> String {
        let mut result = String::new();

        if self.preds.is_empty() {
            return result;
        }

        for pred in self.preds.iter().take(self.preds.len() - 1) {
            result.push_str(&format!("{}, ", pred.0));
        }
        result.push_str(&format!("{}", self.preds.last().unwrap().0));

        result
    }

    fn dump_succs(&self) -> String {
        let mut result = String::new();

        if self.succs.is_empty() {
            return result;
        }

        for pred in self.succs.iter().take(self.succs.len() - 1) {
            result.push_str(&format!("{}, ", pred.0));
        }
        result.push_str(&format!("{}", self.succs.last().unwrap().0));

        result
    }

    fn dump(&self, insts: &[InstData], layout: &[InstNode]) -> String {
        let mut result = String::new();

        if let None = self.first {
            return result;
        }

        result.push_str(&format!(
            "preds: [{}] succs: [{}]\n",
            self.dump_preds(),
            self.dump_succs()
        ));
        let mut to_inst = &self.first;
        while let Some(id) = to_inst {
            result.push_str(&insts[to_inst.unwrap().0].dump(*id));
            result.push('\n');

            to_inst = &layout[to_inst.unwrap().0].next;
        }

        result
    }
}

use std::collections::HashMap;

pub struct Function {
    name: String,
    insts: Vec<InstData>,
    constants: HashMap<Value, InstId>,
    layout: Vec<InstNode>,
    blocks: Vec<BasicBlock>,
}

impl Function {
    pub fn new(name: String) -> Function {
        const AVERAGE_MINIMUM_COUNT: usize = 20;

        Function {
            name: name,
            insts: Vec::<InstData>::with_capacity(AVERAGE_MINIMUM_COUNT),
            constants: HashMap::new(),
            layout: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.insts.len()
    }

    pub fn insts(&self) -> &[InstData] {
        &self.insts
    }

    pub fn constants(&self) -> &HashMap<Value, InstId> {
        &self.constants
    }

    pub fn constants_mut(&mut self) -> &mut HashMap<Value, InstId> {
        &mut self.constants
    }

    pub fn layout(&self) -> &Vec<InstNode> {
        &self.layout
    }

    pub fn blocks(&self) -> &Vec<BasicBlock> {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut Vec<BasicBlock> {
        &mut self.blocks
    }

    pub fn create_inst(&mut self, data: InstData) -> InstId {
        self.insts.push(data);
        self.layout.push(InstNode::new());
        InstId(self.insts.len() - 1)
    }

    pub fn create_block(&mut self) -> BlockId {
        let len = self.blocks.len();
        self.blocks.push(BasicBlock::new());
        BlockId(len)
    }

    pub fn append_inst(&mut self, inst: InstId, block: BlockId) {
        self.layout[inst.0].block = Some(block);
        debug_assert!(
            block.0 < self.blocks.len(),
            "No block {} in Function",
            block.0
        );
        let bb = &mut self.blocks[block.0];

        if let None = bb.first {
            debug_assert!(
                matches!(bb.last, None),
                "BB: {}. First instruction is not set but last is",
                block.0
            );
            bb.first = Some(inst);
            bb.last = Some(inst);
            return;
        }

        if let None = bb.last {
            debug_assert!(
                matches!(bb.first, None),
                "BB: {}. First instruction is not set but last is",
                block.0
            );
        }

        let last_node = &mut self.layout[bb.last.unwrap().0];
        debug_assert!(
            matches!(last_node.next, None),
            "Last instruction in BB {} has the next one",
            block.0
        );
        last_node.next = Some(inst);
        bb.last = Some(inst);
    }
}

impl Function {
    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("Function {}:\n\n", self.name));

        for (id, block) in self.blocks.iter().enumerate() {
            result.push_str(&format!("BB {}: ", id));
            result.push_str(&block.dump(&self.insts, &self.layout));
            result.push('\n');
        }

        result
    }
}

impl std::ops::Index<InstId> for Function {
    type Output = InstData;

    fn index(&self, id: InstId) -> &Self::Output {
        &self.insts[id.0]
    }
}

impl std::ops::IndexMut<InstId> for Function {
    fn index_mut(&mut self, id: InstId) -> &mut Self::Output {
        &mut self.insts[id.0]
    }
}
