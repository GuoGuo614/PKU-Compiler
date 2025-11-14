use std::io::Write;
use koopa::ir::{self, ValueKind};
use super::{AsmCtx, utils::align_to_16};
use super::utils::calculate_type_size;

impl<'a, W: Write> AsmCtx<'a, W> {
    pub fn emit_function(&mut self) {
        self.compute_stack_frame();
        self.emit_prologue();
        self.emit_body();
    }

    fn compute_stack_frame(&mut self) {
        let mut inst_count = 0usize;
        let mut alloc_size = 0usize;
        let mut has_call = 0;
        let mut max_call_params = 0isize;

        for (&_bb, node) in self.func.layout().bbs() {
            for &inst in node.insts().keys() {
                let v = self.func.dfg().value(inst);
                if let ValueKind::Alloc(_) = v.kind() {
                    let ptr_ty = v.ty();
                    if let ir::TypeKind::Pointer(base_ty) = ptr_ty.kind() {
                        if base_ty.is_i32() {
                            alloc_size += 4;
                        } else {
                            alloc_size += calculate_type_size(base_ty);
                        }
                    }
                } else if !v.ty().is_unit() {
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

        let call_params_size = max_call_params as usize * 4;
        self.sf.total = align_to_16((inst_count + has_call) * 4 + call_params_size + alloc_size);
        self.sf.params_base = call_params_size;
    }

    fn emit_prologue(&mut self) {
        if self.sf.total > 0 && self.sf.total < 2048 {
            writeln!(self.w, "\taddi sp, sp, -{}", self.sf.total).unwrap();
        } else if self.sf.total >= 2048 {
            writeln!(self.w, "\tli t0, -{}", self.sf.total).unwrap();
            writeln!(self.w, "\tadd sp, sp, t0").unwrap();
        }
        
        if self.has_call && self.sf.total - 4 < 2048 {
            writeln!(self.w, "\tsw ra, {}(sp)", self.sf.total - 4).unwrap();
        } else if self.has_call && self.sf.total - 4 >= 2048 {
            writeln!(self.w, "\tli t0, {}", self.sf.total - 4).unwrap();
            writeln!(self.w, "\tadd t0, sp, t0").unwrap();
            writeln!(self.w, "\tsw ra, 0(t0)").unwrap();
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