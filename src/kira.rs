use koopa::{ir::builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder, ValueInserter}, *};
use koopa::back::KoopaGenerator;
use crate::ast::*;

pub fn generate_ir(comp: &CompUnit, output: String) {
    let program = program_parse_from_ast(comp);
    let mut gen = KoopaGenerator::new(Vec::new());
    gen.generate_on(&program).expect("koopa serialize failed");
    std::fs::write(output, gen.writer()).expect("write ir file failed");
}

pub fn program_parse_from_ast(comp: &CompUnit) -> ir::Program {
    let mut program = ir::Program::new();
    let mut glo_builder: ir::builder::GlobalBuilder<'_> = program.new_value();

    let global_values = values_parse_from_ast();
    for global_value in global_values {
        glo_builder.insert_value(global_value);
    }

    let func_type = functype_parse_from_ast(&comp.func_def.func_type);
    let func_name = format!("@{}", comp.func_def.ident);

    let func_data = ir::FunctionData::new(func_name.clone(), Vec::new(), func_type);
    let func_handle = program.new_func(func_data);

    {
        let func = program.func_mut(func_handle);
        let dfg = func.dfg_mut();

        // create entry basic block
        let bb_builder = dfg.new_bb();
        let bb = bb_builder.basic_block(Some(String::from("%entry")));

        // create constant 0 and a ret using dfg (globals now available if needed)
        let zero = dfg.new_value().integer(0);
        let ret = dfg.new_value().ret(Some(zero));

        // update layout: push bb, then push ret into bb's inst list
        let layout = func.layout_mut();
        layout.bbs_mut().push_key_back(bb);
        layout.bb_mut(bb).insts_mut().push_key_back(ret);
    }

    program
}

pub fn values_parse_from_ast() -> Vec<ir::entities::ValueData> {
    let result = Vec::new();
    result
}

pub fn functype_parse_from_ast(functype: &FuncType) -> ir::Type {
    match functype._type.as_str() {
        "int" => ir::Type::get_i32(),
        _ => ir::Type::get_i32()
    }
}
