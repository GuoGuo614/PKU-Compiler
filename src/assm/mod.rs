use std::io::Write;
use koopa::ir;

mod context;
mod inst;
mod binary;
mod function;
mod global;
mod utils;
mod alloc;

pub use context::AsmCtx;
pub use utils::{align_to_16, func_is_decl};

pub trait GenerateAsm {
    fn write_assm_to<W: Write>(&self, w: &mut W);
}

impl GenerateAsm for ir::Program {
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        // 生成全局变量段
        global::emit_data_section(self, w);
        
        // 生成代码段
        global::emit_text_section(self, w);
    }
}