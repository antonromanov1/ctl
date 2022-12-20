use crate::optimizer::ir::inst::{InstData, InstId};

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

    pub fn block_mut(&mut self) -> &mut Option<BlockId> {
        &mut self.block
    }

    pub fn next(&self) -> &Option<InstId> {
        &self.next
    }

    pub fn next_mut(&mut self) -> &mut Option<InstId> {
        &mut self.next
    }
}

#[derive(Clone)]
pub struct BasicBlock {
    // Predecessors and successors
    preds: Vec<BlockId>,
    succs: Vec<BlockId>,

    // First and last instructions
    first: Option<InstId>,
    last: Option<InstId>,
}

impl BasicBlock {
    pub fn new() -> Self {
        Self {
            preds: Vec::new(),
            succs: Vec::new(),
            first: None,
            last: None,
        }
    }

    pub fn succs(&self) -> &[BlockId] {
        &self.succs
    }

    pub fn first(&self) -> &Option<InstId> {
        &self.first
    }

    pub fn first_mut(&mut self) -> &mut Option<InstId> {
        &mut self.first
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

    pub fn last_mut(&mut self) -> &mut Option<InstId> {
        &mut self.last
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

    pub fn dump(&self, insts: &[InstData], layout: &[InstNode]) -> String {
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
