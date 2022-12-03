use crate::optimizer::ir::{Function, InstData, InstId};

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

    for (leader, next) in leaders.iter().zip(leaders.iter().skip(1)) {
        let bb = f.create_block();

        let mut id = *leader;
        while id < *next {
            f.append_inst(InstId(id), bb);
            id += 1;
        }
    }

    let last = f.create_block();
    let mut id = *leaders.last().unwrap();
    while id < f.insts().len() {
        f.append_inst(InstId(id), last);
        id += 1;
    }
}
