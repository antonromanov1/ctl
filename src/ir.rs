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

pub type Id = usize;
pub type Operand = usize;
type Dest = usize;
// Branch target which is an instructions number
pub type Target = usize;

pub type Value = i64;

// #[derive(Debug, PartialEq)]
pub enum Inst {
    Constant(Id, Value),
    Parameter(Id),

    Alloc(Id),
    Store(Id, Operand, Dest),
    Load(Id, Operand),

    // Binary instructions
    Add(Id, Operand, Operand),
    Sub(Id, Operand, Operand),
    Mul(Id, Operand, Operand),
    Div(Id, Operand, Operand),
    Mod(Id, Operand, Operand),
    Shl(Id, Operand, Operand),
    Shr(Id, Operand, Operand),

    Neg(Id, Operand),

    IfFalse(Id, Operand, Operand, Cc, Target),
    Goto(Id, Target),
    Return(Id, Operand),
    ReturnVoid(Id),

    Call(Id, String, Vec<Operand>),
}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inst::Constant(id, value) => write!(f, "%{} = Constant {}", id, value),
            Inst::Parameter(id) => write!(f, "%{} = Parameter", id),
            Inst::Alloc(id) => write!(f, "%{} = Alloc", id),
            Inst::Store(id, src, dest) => write!(f, " {} Store %{} at %{}", id, src, dest),
            Inst::Load(id, op) => write!(f, "%{} = Load {}", id, op),

            Inst::Add(id, op1, op2) => write!(f, "%{} = Add %{}, %{}", id, op1, op2),
            Inst::Sub(id, op1, op2) => write!(f, "%{} = Sub %{}, %{}", id, op1, op2),
            Inst::Mul(id, op1, op2) => write!(f, "%{} = Mul %{}, %{}", id, op1, op2),
            Inst::Div(id, op1, op2) => write!(f, "%{} = Div %{}, %{}", id, op1, op2),
            Inst::Mod(id, op1, op2) => write!(f, "%{} = Mod %{}, %{}", id, op1, op2),
            Inst::Shl(id, op1, op2) => write!(f, "%{} = Shl %{}, %{}", id, op1, op2),
            Inst::Shr(id, op1, op2) => write!(f, "%{} = Shr %{}, %{}", id, op1, op2),

            Inst::Neg(id, op) => write!(f, "%{} = Neg %{}", id, op),

            Inst::IfFalse(id, op1, op2, cc, target) => {
                write!(
                    f,
                    " {} IfFalse %{} {} %{}, goto {}",
                    id, op1, cc, op2, target
                )
            }
            Inst::Goto(id, target) => write!(f, " {} Goto {}", id, target),
            Inst::Return(id, value) => write!(f, " {} Return %{}", id, value),
            Inst::ReturnVoid(id) => write!(f, " {} ReturnVoid", id),

            Inst::Call(id, name, args) => {
                let mut s = String::new();
                s.push_str(&format!("%{} = Call {}, args: ", id, name));

                for (i, arg) in args.iter().enumerate() {
                    if i != args.len() - 1 {
                        s.push_str(&format!("%{}, ", arg));
                    } else {
                        s.push_str(&format!("%{}", arg));
                    }
                }

                write!(f, "{}", s)
            }
        }
    }
}

use std::collections::HashMap;

pub struct Function {
    insts: Vec<Inst>,

    #[allow(dead_code)]
    constants: HashMap<Value, Id>,
}

impl Function {
    pub fn new(insts: Vec<Inst>, constants: HashMap<Value, Id>) -> Function {
        Function {
            insts: insts,
            constants: constants,
        }
    }

    pub fn get_insts(&self) -> &[Inst] {
        &self.insts
    }
}
