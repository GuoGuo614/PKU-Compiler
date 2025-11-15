// 函数定义处理

use std::collections::VecDeque;
use koopa::*;
use crate::ast::*;
use super::symbol::SymbolTable;
use super::ctx::FuncCtx;
use super::utils::{functype_parse_from_ast, param_type_parse};
use super::block::process_block;
use super::const_eval::EvalConst;

use koopa::{ir::builder::BasicBlockBuilder};

/// 处理函数定义
pub fn process_func_def(func_def: &FuncDef, program: &mut ir::Program, global_sym: &mut SymbolTable) {
    let params_type: Vec<ir::Type> = func_def.params.as_ref()
        .map(|f_params| f_params.params.iter()
            .map(|p| param_type_parse(p))
            .collect())
        .unwrap_or_default();

    let func_type = functype_parse_from_ast(&func_def.func_type);
    let func_name = format!("@{}", func_def.ident);

    let func_data = ir::FunctionData::new(func_name.clone(), params_type, func_type);
    let func_handle = program.new_func(func_data);

    global_sym.insert_func(&func_name, &func_handle);

    // 创建函数作用域，处理参数，解析函数体
    let mut sym = SymbolTable::with_parent(global_sym);
    parse_function_body(func_def, program, func_handle, &mut sym);
}

fn parse_function_body(
    func_def: &FuncDef, 
    program: &mut ir::Program, 
    func_handle: ir::Function,
    sym: &mut SymbolTable,
) {
    let func = program.func_mut(func_handle);

    // 1) 短借用 dfg 创建 entry 基本块
    let bb = {
        let dfg = func.dfg_mut();
        let bb = dfg.new_bb().basic_block(Some(String::from("%entry")));
        {
            let layout = func.layout_mut();
            layout.bbs_mut().push_key_back(bb).expect("Failed to push basic block");
        }
        bb
    };

    // 3) 用上下文封装对 func 的操作，避免重叠借用
    let mut ctx = FuncCtx { func, bb, sym, bb_counter: 0 , loop_stack: VecDeque::new()};
    
    if let Some(f_params) = &func_def.params {
        let param_values: Vec<_> = ctx.func.params().to_vec();

        for (param, &param_value) in f_params.params.iter().zip(&param_values) {
            let var_name = format!("%{}", param.ident);
            if let Some(ref index_list) = param.indexs {
                // 数组参数：需要 alloc **i32 并 store
                let mut sizes = vec![0];
                for const_exp in index_list {
                    let size = const_exp.exp.eval(ctx.sym) as usize;
                    sizes.push(size);
                }
                let alloc = ctx.make_alloc(ir::Type::get_pointer(ir::Type::get_i32()), Some(var_name));
                ctx.make_store(alloc, param_value);
                ctx.sym.insert_array_pointer(&param.ident, alloc, sizes);
            } else {
                let alloc = ctx.make_alloc(ir::Type::get_i32(), Some(var_name));
                ctx.make_store(alloc, param_value);
                ctx.sym.insert_var(&param.ident, alloc);
            }
        }
    }

    process_block(&func_def.block, &mut ctx);

    if !ctx.is_bb_terminated() {
        match &func_def.func_type {
            FuncType::Void => {
                ctx.emit_ret(None);
            },
            FuncType::Int => {
                let v = ctx.make_int(0);
                ctx.emit_ret(Some(v));
            }
            // _ => panic!("Miss a Return Statement!"),
        }
    }
}
