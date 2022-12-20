use ctl::optimizer::ir::basic_block::{BasicBlock, BlockId, InstNode};
use ctl::optimizer::ir::function::Function;
use ctl::optimizer::ir::inst::{Cc, InstData, InstId};

struct FuncRaw(*mut Function);
unsafe impl Sync for FuncRaw {}

impl FuncRaw {
    fn get(&self) -> *mut Function {
        self.0
    }

    fn set(&mut self, value: *mut Function) {
        self.0 = value;
    }
}

const NULL_POINTER: *mut Function = 0 as *mut Function;

static mut FUNC: FuncRaw = FuncRaw(NULL_POINTER);

static mut CUR_BLOCK: BlockId = BlockId(0);
static mut CUR_INST: InstId = InstId(0);

pub fn get_func() -> &'static mut Function {
    unsafe { &mut *FUNC.get() }
}

pub struct Constructor {}

/// After filling the Function with the basic blocks this adds the predecessors
pub fn function(_insts_len: Constructor, _blocks: &[Constructor]) {
    unsafe {
        debug_assert_ne!(FUNC.get(), NULL_POINTER);

        for (id, block) in (*FUNC.get()).blocks_mut().iter().enumerate() {
            for succ in block.succs() {
                (*FUNC.get()).blocks_mut()[succ.0].add_pred(BlockId(id));
            }
        }
    }
}

pub fn init(insts_len: usize, blocks_len: usize) -> Constructor {
    // Allocate memory for Function using Box
    let smart = Box::new(Function::new(Default::default()));
    let func_raw = Box::into_raw(smart);

    unsafe {
        if FUNC.get() != NULL_POINTER {
            // Free memory from old Function
            {
                Box::from_raw(FUNC.get());
            }
        }
        FUNC.set(func_raw);
    }

    // Initialize insts, layout and blocks
    unsafe {
        *(*FUNC.get()).insts_mut() = vec![InstData::Invalid; insts_len];
        *(*FUNC.get()).layout_mut() = vec![InstNode::new(); insts_len];

        *(*FUNC.get()).blocks_mut() = vec![BasicBlock::new(); blocks_len];
    }

    Constructor {}
}

pub fn basic_block(id: usize) -> Constructor {
    unsafe {
        debug_assert_ne!(FUNC.get(), NULL_POINTER);
        CUR_BLOCK = BlockId(id);
    }

    Constructor {}
}

/// The same opcodes as in InstData except of IfFalse and Goto instructions
pub enum Opcode {
    Constant,
    Parameter,
    Alloc,
    Store,
    Load,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    Neg,
    Return,
    ReturnVoid,
    Call,
    Branch,
    Jump,
}

/// Creates an instruction with id and opcode.
pub fn inst(id: usize, opcode: Opcode) -> Constructor {
    let data = match opcode {
        Opcode::Constant => InstData::Constant(Default::default()),
        Opcode::Parameter => InstData::Parameter,
        Opcode::Alloc => InstData::Alloc,
        Opcode::Store => InstData::Store(Default::default(), Default::default()),
        Opcode::Load => InstData::Load(Default::default()),
        Opcode::Add => InstData::Add(Default::default(), Default::default()),
        Opcode::Sub => InstData::Sub(Default::default(), Default::default()),
        Opcode::Mul => InstData::Mul(Default::default(), Default::default()),
        Opcode::Div => InstData::Div(Default::default(), Default::default()),
        Opcode::Mod => InstData::Mod(Default::default(), Default::default()),
        Opcode::Shl => InstData::Shl(Default::default(), Default::default()),
        Opcode::Shr => InstData::Shr(Default::default(), Default::default()),
        Opcode::Neg => InstData::Neg(Default::default()),
        Opcode::Return => InstData::Return(Default::default()),
        Opcode::ReturnVoid => InstData::ReturnVoid,
        Opcode::Call => InstData::Call(Default::default(), Default::default()),
        Opcode::Branch => InstData::Branch(Default::default(), Default::default(), Cc::Invalid),
        Opcode::Jump => InstData::Jump,
    };

    unsafe {
        debug_assert_ne!(FUNC.get(), NULL_POINTER);
        CUR_INST = InstId(id);
        (*FUNC.get()).insts_mut()[CUR_INST.0] = data;
        (*FUNC.get()).append_inst(CUR_INST, CUR_BLOCK);
    }

    Constructor {}
}

impl Constructor {
    pub fn succs(&self, elems: &[usize]) -> Self {
        unsafe {
            debug_assert_ne!(FUNC.get(), NULL_POINTER);
        }
        for el in elems {
            unsafe {
                (*FUNC.get()).blocks_mut()[CUR_BLOCK.0].add_succ(BlockId(*el));
            }
        }
        Constructor {}
    }

    pub fn insts(&self, _: &[Constructor]) -> Self {
        Constructor {}
    }

    pub fn value(&self, data: i64) -> Self {
        unsafe {
            debug_assert_ne!(FUNC.get(), NULL_POINTER);
        }

        let inst_data: &mut InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };
        match inst_data {
            InstData::Constant(ref mut value) => *value = data,
            _ => panic!("value() called not for Constant instruction"),
        }

        Constructor {}
    }

    /// Sets inputs to an instruction.
    pub fn inputs(&self, args: &[usize]) -> Self {
        unsafe {
            debug_assert_ne!(FUNC.get(), NULL_POINTER);
        }

        let inst_data: &mut InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };

        match inst_data {
            InstData::Store(ref mut value, _) => {
                debug_assert_eq!(
                    args.len(),
                    1,
                    "Instruction with ID {}: Store should have only one input (value to store) but {} inputs were given",
                    unsafe{ CUR_INST.0 }, args.len()
                );
                *value = InstId(args[0]);
            }
            InstData::Load(ref mut ptr) => {
                debug_assert_eq!(
                    args.len(),
                    1,
                    "Instruction with ID {}: Load should have only one input (pointer to the variable) but {} inputs were given",
                    unsafe{ CUR_INST.0 }, args.len()
                );
                *ptr = InstId(args[0]);
            }

            InstData::Add(ref mut op1, ref mut op2) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Add should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Sub(ref mut op1, ref mut op2) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Sub should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Mul(ref mut op1, ref mut op2) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Mul should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Div(ref mut op1, ref mut op2) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Div should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Mod(ref mut op1, ref mut op2) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Mod should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Shl(ref mut op1, ref mut op2) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Shl should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Shr(ref mut op1, ref mut op2) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Shr should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }

            InstData::Neg(ref mut op) => {
                debug_assert_eq!(
                    args.len(),
                    1,
                    "Instruction with ID {}: Neg should have only one input (value) but {} inputs given",
                    unsafe { CUR_INST.0 }, args.len()
                );
                *op = InstId(args[0]);
            }

            InstData::Return(ref mut value) => {
                debug_assert_eq!(
                    args.len(),
                    1,
                    "Instruction with ID {}: Return should have only one input (value) but {} inputs given",
                    unsafe { CUR_INST.0 }, args.len()
                );
                *value = InstId(args[0]);
            }

            InstData::Call(_, ref mut params) => {
                let mut ids = Vec::new();
                for arg in args {
                    ids.push(InstId(*arg));
                }

                *params = ids;
            }

            InstData::Branch(ref mut op1, ref mut op2, _) => {
                debug_assert_eq!(
                    args.len(),
                    2,
                    "Instruction with ID {}: Branch should have only 2 inputs but {} were given",
                    unsafe { CUR_INST.0 },
                    args.len()
                );
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }

            InstData::Alloc
            | InstData::Constant(_)
            | InstData::Jump
            | InstData::Parameter
            | InstData::ReturnVoid => {
                panic!(
                    "Instruction with ID {}: should not have an input but {} inputs given",
                    unsafe { CUR_INST.0 },
                    args.len()
                )
            }

            InstData::IfFalse(_, _, _, _) | InstData::Goto(_) => {
                panic!(
                    "Instruction with ID {}: Such an instruction should not be in current stage",
                    unsafe { CUR_INST.0 },
                )
            }

            InstData::Invalid => panic!("Invalid should not be created in the ir constructor"),
        };

        Constructor {}
    }

    /// Sets destination operand to Store instruction.
    pub fn dest(&self, d: usize) -> Self {
        unsafe {
            debug_assert_ne!(FUNC.get(), NULL_POINTER);
        }

        let inst_data: &mut InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };
        match inst_data {
            InstData::Store(_, ref mut dest) => {
                *dest = InstId(d);
            }

            _ => panic!("Only the Store instruction can have a destination"),
        };

        Constructor {}
    }

    /// Sets the condition code to Branch instruction.
    pub fn cc(&self, cond: Cc) -> Self {
        unsafe {
            debug_assert_ne!(FUNC.get(), NULL_POINTER);
        }

        let inst_data: &mut InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };
        match inst_data {
            InstData::Branch(_, _, ref mut c) => {
                *c = cond;
            }

            InstData::IfFalse(_, _, _, _) => panic!("IfFalse should not be at this stage"),
            _ => panic!("Only the Branch instruction can have a condition code"),
        };

        Constructor {}
    }
}

pub fn compare_functions(f1: &Function, f2: &Function) -> Result<(), String> {
    if f1.blocks().len() != f2.blocks().len() {
        return Err("Different length of the blocks".to_string());
    }

    for (id, block) in f1.blocks().iter().enumerate() {
        if block.succs() != f2.blocks()[id].succs() {
            return Err("Fields succs differ".to_string());
        }

        let b = BlockId(id);
        unsafe {
            let mut to_inst1 = f1.blocks()[b.0].first() as *const Option<InstId>;
            while let Some(inst_id) = *to_inst1 {
                let in1 = &f1.insts()[inst_id.0];
                let in2 = &f2.insts()[inst_id.0];
                if in1 != in2 {
                    return Err(format!(
                        "Instructions differ: ({}) and ({})",
                        in1.dump(inst_id),
                        in2.dump(inst_id)
                    ));
                }
                to_inst1 = f1.layout()[inst_id.0].next() as *const Option<InstId>;
            }
        }
    }

    Ok(())
}

pub fn dump() -> String {
    unsafe { (*FUNC.get()).dump() }
}
