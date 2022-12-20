use crate::optimizer::ir::basic_block::BlockId;
use crate::optimizer::ir::function::Function;
use crate::optimizer::ir::inst::{InstData, InstId};

fn find_leaders(insts: &[InstData]) -> Vec<usize> {
    let mut leaders = Vec::new();

    for (i, data) in insts.iter().enumerate() {
        // First instruction is a leader
        if i == 0 {
            leaders.push(i);
        }

        // Target instruction and instruction following branch are leaders
        if let Some(target) = data.target() {
            leaders.push(target.0);
            leaders.push(i + 1);
        }
    }

    leaders.sort_unstable();
    leaders.dedup();

    leaders
}

pub fn build_intermediate_representation(f: &mut Function) {
    debug_assert!(!f.insts().is_empty());
    debug_assert!(f.blocks().is_empty());

    let leaders = find_leaders(f.insts());

    // Create a basic block for each leader except the last one. Fill it with
    // the respective instructions before the next leader.
    for (leader, next) in leaders.iter().zip(leaders.iter().skip(1)) {
        let bb = f.create_block();

        let mut id = *leader;
        while id < *next {
            f.append_inst(InstId(id), bb);
            id += 1;
        }
    }

    // Create a basic block for the last leader. Fill it with the respective
    // instructions from the last leader to the last one instruction inclusive.
    let last = f.create_block();
    let mut id = *leaders.last().unwrap();
    while id < f.insts().len() {
        f.append_inst(InstId(id), last);
        id += 1;
    }

    let mut current = 0;
    while current < f.blocks().len() - 1 {
        let last_inst = f.blocks()[current].last().unwrap();
        match &f[last_inst] {
            InstData::IfFalse(op1, op2, cc, target) => {
                let (op1_clone, op2_clone, cc_clone) = (op1.clone(), op2.clone(), cc.clone());

                // Add arcs from the current basic block to the target ones.
                // True successor goes first.
                let target_block = f.layout()[target.0].block();

                // Arc to the true successor
                f.blocks_mut()[current].add_succ(BlockId(current + 1));
                f.blocks_mut()[current + 1].add_pred(BlockId(current));

                // Arc to the false successor
                f.blocks_mut()[current].add_succ(target_block);
                f.blocks_mut()[target_block.0].add_pred(BlockId(current));

                // Translate IfFalse to Branch
                f[last_inst] = InstData::Branch(op1_clone, op2_clone, cc_clone);
            }
            InstData::Goto(target) => {
                // Add an arc from the current basic block to the target one
                let target_block = f.layout()[target.0].block();
                f.blocks_mut()[target_block.0].add_pred(BlockId(current));
                f.blocks_mut()[current].add_succ(target_block);

                // Translate Goto to Jump
                f[last_inst] = InstData::Jump;
            }
            _ => (),
        };

        // If the last instruction is not a Branch or a Jump then just add an
        // arc from current to the next basic block.
        match &f[last_inst] {
            InstData::Branch(_, _, _) | InstData::Jump => (),
            _ => {
                f.blocks_mut()[current].add_succ(BlockId(current + 1));
                f.blocks_mut()[current + 1].add_pred(BlockId(current));

                let jump = f.create_inst(InstData::Jump);
                f.append_inst(jump, BlockId(current));
            }
        };

        current += 1;
    }
}
