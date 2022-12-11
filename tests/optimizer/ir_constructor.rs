use ctl::optimizer::ir;

struct FuncRaw(*mut ir::Function);
unsafe impl Sync for FuncRaw {}

impl FuncRaw {
    fn get(&self) -> *mut ir::Function {
        self.0
    }

    fn set(&mut self, value: *mut ir::Function) {
        self.0 = value;
    }
}

const NULL_POINTER: *mut ir::Function = 0 as *mut ir::Function;

static mut FUNC: FuncRaw = FuncRaw(NULL_POINTER);

static mut CUR_BLOCK: ir::BlockId = ir::BlockId(0);
static mut CUR_INST: ir::InstId = ir::InstId(0);

pub fn get_func() -> &'static mut ir::Function {
    unsafe { &mut *FUNC.get() }
}

pub struct Constructor {}

/// After filling the Function with the basic blocks this adds the predecessors
pub fn function(_insts_len: Constructor, _blocks: &[Constructor]) {
    unsafe {
        debug_assert_ne!(FUNC.get(), NULL_POINTER);

        for (id, block) in (*FUNC.get()).blocks_mut().iter().enumerate() {
            for succ in block.succs() {
                (*FUNC.get()).blocks_mut()[succ.0].add_pred(ir::BlockId(id));
            }
        }
    }
}

pub fn init(insts_len: usize, blocks_len: usize) -> Constructor {
    // Allocate memory for Function using Box
    let smart = Box::new(ir::Function::new(Default::default()));
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
        *(*FUNC.get()).insts_mut() = vec![ir::InstData::Invalid; insts_len];
        *(*FUNC.get()).layout_mut() = vec![ir::InstNode::new(); insts_len];

        *(*FUNC.get()).blocks_mut() = vec![ir::BasicBlock::new(); blocks_len];
    }

    Constructor {}
}

pub fn basic_block(id: usize) -> Constructor {
    unsafe {
        debug_assert_ne!(FUNC.get(), NULL_POINTER);
        CUR_BLOCK = ir::BlockId(id);
    }

    Constructor {}
}

/// The same opcodes as in ir::InstData except of IfFalse and Goto instructions
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
    use ir::InstData;
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
        Opcode::Branch => InstData::Branch(Default::default(), Default::default(), ir::Cc::Invalid),
        Opcode::Jump => InstData::Jump,
    };

    unsafe {
        debug_assert_ne!(FUNC.get(), NULL_POINTER);
        CUR_INST = ir::InstId(id);
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
                (*FUNC.get()).blocks_mut()[CUR_BLOCK.0].add_succ(ir::BlockId(*el));
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

        let inst_data: &mut ir::InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };
        match inst_data {
            ir::InstData::Constant(ref mut value) => *value = data,
            _ => panic!("value() called not for Constant instruction"),
        }

        Constructor {}
    }

    /// Sets inputs to an instruction.
    pub fn inputs(&self, args: &[usize]) -> Self {
        unsafe {
            debug_assert_ne!(FUNC.get(), NULL_POINTER);
        }

        let inst_data: &mut ir::InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };
        use ir::InstData;
        use ir::InstId;

        match inst_data {
            InstData::Store(ref mut value, _) => {
                debug_assert_eq!(
                    args.len(),
                    1,
                    "Store should have only one input - value to store"
                );
                *value = InstId(args[0]);
            }
            InstData::Load(ref mut ptr) => {
                debug_assert_eq!(
                    args.len(),
                    1,
                    "Load should have only one input - pointer to the variable"
                );
                *ptr = InstId(args[0]);
            }

            InstData::Add(ref mut op1, ref mut op2) => {
                debug_assert_eq!(args.len(), 2, "Add should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Sub(ref mut op1, ref mut op2) => {
                debug_assert_eq!(args.len(), 2, "Sub should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Mul(ref mut op1, ref mut op2) => {
                debug_assert_eq!(args.len(), 2, "Mul should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Div(ref mut op1, ref mut op2) => {
                debug_assert_eq!(args.len(), 2, "Div should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Mod(ref mut op1, ref mut op2) => {
                debug_assert_eq!(args.len(), 2, "Mod should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Shl(ref mut op1, ref mut op2) => {
                debug_assert_eq!(args.len(), 2, "Shl should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }
            InstData::Shr(ref mut op1, ref mut op2) => {
                debug_assert_eq!(args.len(), 2, "Shr should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }

            InstData::Neg(ref mut op) => {
                debug_assert_eq!(args.len(), 1, "Neg should have only one input - value");
                *op = InstId(args[0]);
            }

            InstData::Return(ref mut value) => {
                debug_assert_eq!(args.len(), 1, "Return should have only one input - value");
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
                debug_assert_eq!(args.len(), 2, "Branch should have only 2 inputs");
                *op1 = InstId(args[0]);
                *op2 = InstId(args[1]);
            }

            InstData::Alloc
            | InstData::Constant(_)
            | InstData::Jump
            | InstData::Parameter
            | InstData::ReturnVoid => panic!("Such an instruction can not have an input"),

            InstData::IfFalse(_, _, _, _) | InstData::Goto(_) => {
                panic!("Such an instruction should not be in current stage")
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

        let inst_data: &mut ir::InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };
        match inst_data {
            ir::InstData::Store(_, ref mut dest) => {
                *dest = ir::InstId(d);
            }

            _ => panic!("Only the Store instruction can have a destination"),
        };

        Constructor {}
    }

    /// Sets the condition code to Branch instruction.
    pub fn cc(&self, cond: ir::Cc) -> Self {
        unsafe {
            debug_assert_ne!(FUNC.get(), NULL_POINTER);
        }

        let inst_data: &mut ir::InstData = unsafe { &mut (*FUNC.get()).insts_mut()[CUR_INST.0] };
        match inst_data {
            ir::InstData::Branch(_, _, ref mut c) => {
                *c = cond;
            }

            ir::InstData::IfFalse(_, _, _, _) => panic!("IfFalse should not be at this stage"),
            _ => panic!("Only the Branch instruction can have a condition code"),
        };

        Constructor {}
    }
}

pub fn compare_functions(f1: &ir::Function, f2: &ir::Function) -> Result<(), String> {
    if f1.blocks().len() != f2.blocks().len() {
        return Err("Different length of the blocks".to_string());
    }

    for (id, block) in f1.blocks().iter().enumerate() {
        if block.succs() != f2.blocks()[id].succs() {
            return Err("Fields succs differ".to_string());
        }

        let b = ir::BlockId(id);
        unsafe {
            let mut to_inst1 = f1.blocks()[b.0].first() as *const Option<ir::InstId>;
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
                to_inst1 = f1.layout()[inst_id.0].next() as *const Option<ir::InstId>;
            }
        }
    }

    Ok(())
}

pub fn dump() -> String {
    unsafe { (*FUNC.get()).dump() }
}
