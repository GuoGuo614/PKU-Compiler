use std::io::Write;
use koopa::ir;
use super::alloc::{RegAlloc, StackFrame};

pub struct AsmCtx<'a, W: Write> {
    pub w: &'a mut W,
    pub program: &'a ir::Program,
    pub func: &'a ir::FunctionData,
    pub ra: RegAlloc,
    pub sf: StackFrame,
    pub has_call: bool,
}

impl<'a, W: Write> AsmCtx<'a, W> {
    pub fn new(program: &'a ir::Program, func: &'a ir::FunctionData, w: &'a mut W) -> Self {
        Self { 
            w, 
            program, 
            func, 
            ra: RegAlloc::new(), 
            sf: StackFrame::new(),
            has_call: false,
        }
    }

    pub fn get_bb_name(&self, bb: ir::BasicBlock) -> String {
        self.func.dfg()
            .bb(bb)
            .name()
            .as_ref()
            .map(|s| s.trim_start_matches('%').to_string())
            .unwrap()
    }

    // pub fn is_global(&self, value: ir::Value) -> bool {
    //     self.func.dfg().values().get(&value).is_none()
    // }
    
    pub fn get_global_name(&self, value: ir::Value) -> String {
        let data = self.program.borrow_value(value);
        data.name()
            .as_ref()
            .unwrap()
            .strip_prefix('@')
            .unwrap()
            .to_string()
    }
}