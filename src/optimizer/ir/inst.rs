/// Condition code
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

#[derive(Clone, PartialEq)]
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

    // An invalid instruction. Used in order to avoid Option.
    Invalid,
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

            InstData::Invalid => panic!("No dump for Invalid instruction"),
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

#[derive(Clone, Copy, PartialEq)]
pub struct BlockId(pub usize);

#[derive(Clone)]
pub struct InstNode {
    block: Option<BlockId>,
    next: Option<InstId>,
}

impl InstNode {
    pub fn new() -> Self {
        Self {
            block: None,
            next: None,
        }
    }

    pub fn block(&self) -> BlockId {
        self.block.unwrap()
    }

    pub fn next(&self) -> &Option<InstId> {
        &self.next
    }
}
