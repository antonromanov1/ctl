use std::collections::HashMap;

use crate::optimizer::ir::basic_block::{BasicBlock, BlockId, InstNode};
use crate::optimizer::ir::inst::{InstData, InstId, Value};

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

    pub fn insts(&self) -> &Vec<InstData> {
        &self.insts
    }

    pub fn insts_mut(&mut self) -> &mut Vec<InstData> {
        &mut self.insts
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

    pub fn layout_mut(&mut self) -> &mut Vec<InstNode> {
        &mut self.layout
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
        *self.layout[inst.0].block_mut() = Some(block);
        debug_assert!(
            block.0 < self.blocks.len(),
            "No block {} in Function",
            block.0
        );
        let bb = &mut self.blocks[block.0];

        if let None = bb.first() {
            debug_assert!(
                matches!(bb.last(), None),
                "BB: {}. First instruction is not set but last is",
                block.0
            );
            *bb.first_mut() = Some(inst);
            *bb.last_mut() = Some(inst);
            return;
        }

        if let None = bb.last() {
            debug_assert!(
                matches!(bb.first(), None),
                "BB: {}. First instruction is not set but last is",
                block.0
            );
        }

        let last_node = &mut self.layout[bb.last().unwrap().0];
        debug_assert!(
            matches!(last_node.next(), None),
            "Last instruction in BB {} has the next one",
            block.0
        );
        *last_node.next_mut() = Some(inst);
        *bb.last_mut() = Some(inst);
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
