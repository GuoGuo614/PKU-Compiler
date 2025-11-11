use koopa::*;
use koopa::back::KoopaGenerator;
use crate::ast::*;

// 导出子模块
mod global;
mod function;
mod block;
mod stmt;
mod expr;
mod ctx;
mod symbol;
mod const_eval;
pub mod utils;

// 导出必要的函数
pub use global::{insert_library_functions, process_comp_unit};
pub use function::process_func_def;
use symbol::SymbolTable;

pub trait GenerateIR {
    fn generate_ir(&self) -> ir::Program;
    fn write_ir(&self, output: String);
}

impl GenerateIR for CompUnit {
    fn write_ir(&self, output: String) {
        let program = self.generate_ir();
        let mut gen_ir = KoopaGenerator::new(Vec::new());
        gen_ir.generate_on(&program).expect("koopa serialize failed");
        std::fs::write(output, gen_ir.writer()).expect("write ir file failed");
    }

    fn generate_ir(&self) -> ir::Program {
        let mut program = ir::Program::new();

        // 为 Program 插入库函数声明
        let mut global_sym = SymbolTable::new();
        insert_library_functions(&mut program, &mut global_sym);
        
        // 从头开始处理
        process_comp_unit(self, &mut program, &mut global_sym);
        program
    }
}
