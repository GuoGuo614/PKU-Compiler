use std::io::Write;
use koopa::ir::ValueKind;
use super::{AsmCtx, utils::align_to_16};

impl<'a, W: Write> AsmCtx<'a, W> {
    pub fn emit_function(&mut self) {
        self.compute_stack_frame();
        self.emit_prologue();
        self.emit_body();
    }

    fn compute_stack_frame(&mut self) {
        let mut inst_count = 0usize;
        let mut has_call = 0;
        let mut max_call_params = 0isize;

        for (&_bb, node) in self.func.layout().bbs() {
            for &inst in node.insts().keys() {
                let v = self.func.dfg().value(inst);
                if !v.ty().is_unit() {
                    inst_count += 1;
                }
                if let ValueKind::Call(call) = v.kind() {
                    has_call = 1;
                    self.has_call = true;
                    max_call_params = std::cmp::max(
                        max_call_params, 
                        call.args().len() as isize - 8
                    );
                }
            }
        }

        self.sf.total = align_to_16((inst_count + has_call + max_call_params as usize) * 4);
        self.sf.params_base = max_call_params as usize * 4;
    }

    fn emit_prologue(&mut self) {
        if self.sf.total > 0 {
            writeln!(self.w, "\taddi sp, sp, -{}", self.sf.total).unwrap();
        }
        if self.has_call {
            writeln!(self.w, "\tsw ra, {}(sp)", self.sf.total - 4).unwrap();
        }
    }

    fn emit_body(&mut self) {
        for (&bb, node) in self.func.layout().bbs() {
            self.emit_block(bb, node);
        }
    }

    fn emit_block(&mut self, bb: koopa::ir::BasicBlock, node: &koopa::ir::layout::BasicBlockNode) {
        let name = self.get_bb_name(bb);
        if name.as_str() != "entry" {
            writeln!(self.w, "{}:", name).unwrap();
        }
        for &inst in node.insts().keys() {
            self.emit_inst(inst);
        }
    }
}