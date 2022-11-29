//! Intermediate representation (IR)

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

    IfFalse(Operand, Operand, Cc, Target),
    Goto(Target),
    Return(InstId),
    ReturnVoid,

    Call(String, Vec<InstId>),
}

impl InstData {
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

            InstData::IfFalse(op1, op2, cc, target) => {
                write!(f, "IfFalse %{} {} %{}, goto {}", op1, cc, op2, target)
            }
            InstData::Goto(target) => write!(f, "Goto {}", target),
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
        }
    }
}

use std::collections::HashMap;

pub struct Function {
    insts: Vec<InstData>,
    constants: HashMap<Value, InstId>,
}

impl Function {
    pub fn new() -> Function {
        const AVERAGE_MINIMUM_COUNT: usize = 20;
        Function {
            insts: Vec::<InstData>::with_capacity(AVERAGE_MINIMUM_COUNT),
            constants: HashMap::new(),
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

    pub fn create_inst(&mut self, data: InstData) -> InstId {
        self.insts.push(data);
        InstId(self.insts.len() - 1)
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
