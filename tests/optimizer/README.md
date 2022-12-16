# Tests for the optimizer

## IR Constructor

**Graph** is a control flow graph which nodes contain intermediate instructions ending with a
terminator.

IR **Constructor** is a way of creating unnatural graph which is not come from any actual input
source code. It is implemented in module `ir_constructor`.rs.

Let's take a look at an **example**:

```rust
function(
    init(9, 3),
    &[
        basic_block(0).succs(&[1, 2]).insts(&[
            inst(0, Opcode::Parameter),
            inst(1, Opcode::Alloc),
            inst(2, Opcode::Constant).value(0),
            inst(3, Opcode::Store).inputs(&[2]).dest(1),
            inst(4, Opcode::Branch).inputs(&[0, 2]).cc(Cc::Eq),
        ]),
        basic_block(1).succs(&[2]).insts(&[
            inst(5, Opcode::Constant).value(1),
            inst(6, Opcode::Store).inputs(&[5]).dest(1),
            inst(8, Opcode::Jump),
        ]),
        basic_block(2).insts(&[inst(7, Opcode::ReturnVoid)]),
    ],
);
```

This is a valid Rust code in a test function. What does each call inside mean:

General information:
* **function** creates a new graph.
* **init**(9, 3) means that we have in our graph at least **9** instructions and **3** basic blocks.

Basic block:
* **basic_block**(0) creates a new basic block with ID **0** inside the current graph.
* **succs**(&[1, 2]) adds basic blocks with IDs **1** and **2** as successors to the current one (arcs from
current to the successors).
* **insts**(&[...]) adds instructions to the current basic block.

Instruction:
* **inst**(0, Opcode::Parameter) creates an IR instruction **Parameter** with ID **0**.
* **value**(0) sets a value **0** to the current instruction if it is a **Constant**.
* **inputs**(&[0, 2]) adds instructions with IDs **0** and **2** as inputs (input operands) to the
current instruction. This is an **SSA** form ([wiki](https://en.wikipedia.org/wiki/Static_single-assignment_form)) therefore each instruction (except Store, Branch,
Jump, Return(Void)) is associated with the resulting value (**SSA value**) which has the only
one definition, this instrucion.
* **dest**(1) sets instruction **Alloc** with ID **1** as a destination pointer to the current Store
instruction.
* **cc**(Cc::Eq) sets condition code equality to the current **Branch** instruction.
