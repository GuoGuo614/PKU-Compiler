// 全局变量和库函数处理

use koopa::*;
use crate::ast::*;
use super::symbol::SymbolTable;
use super::const_eval::EvalConst;
use super::utils::process_dimensions;
use super::function::process_func_def;

use koopa::{ir::builder::{GlobalInstBuilder, ValueBuilder}};

/// 插入 SysY 库函数声明
pub fn insert_library_functions(program: &mut ir::Program, symbol_table: &mut SymbolTable) {
    let lib_funcs = [
        ("@getint", vec![], ir::Type::get_i32()),
        ("@getch", vec![], ir::Type::get_i32()),
        ("@getarray", vec![ir::Type::get_pointer(ir::Type::get_i32())], ir::Type::get_i32()),
        ("@putint", vec![ir::Type::get_i32()], ir::Type::get_unit()),
        ("@putch", vec![ir::Type::get_i32()], ir::Type::get_unit()),
        ("@putarray", vec![ir::Type::get_i32(), ir::Type::get_pointer(ir::Type::get_i32())], ir::Type::get_unit()),
        ("@starttime", vec![], ir::Type::get_unit()),
        ("@stoptime", vec![], ir::Type::get_unit()),
    ];

    for (name, params, ret_ty) in lib_funcs {
        let func = program.new_func(ir::FunctionData::new_decl(
            name.to_string(),
            params,
            ret_ty,
        ));
        symbol_table.insert_func(name, &func);
    }
}

/// 处理编译单元
pub fn process_comp_unit(
    comp: &CompUnit, 
    program: &mut ir::Program, 
    global_sym: &mut SymbolTable
) {
    // 递归处理多个函数和全局变量
    if let Some(prev) = comp.comp_unit.as_ref() {
        process_comp_unit(prev, program, global_sym);
    }

    match &comp.body {
        CompBody::FuncDef(func_def) => {
            process_func_def(func_def, program, global_sym);
        },
        CompBody::Decl(decl) => {
            process_global_decl(decl, program, global_sym);
        }
    }
}

/// 创建全局分配
pub fn make_global_alloc(program: &mut ir::Program, init: Option<ir::Value>) -> ir::Value {
    if let Some(val) = init {
        program.new_value().global_alloc(val)
    } else {
        let zero_init = program.new_value().zero_init(ir::Type::get_i32());
        program.new_value().global_alloc(zero_init)
    }
}

fn make_array_init<T: EvalConst>(
    program: &mut ir::Program,
    exps: &[Option<T>],
    size: usize,
    global_sym: &SymbolTable,
) -> ir::Value {
    if exps.is_empty() || exps.iter().all(|exp| exp.is_none()) {
        let array_type = ir::Type::get_array(ir::Type::get_i32(), size);
        program.new_value().zero_init(array_type)
    } else {
        let zero = program.new_value().integer(0);
        let inits: Vec<ir::Value> = exps.iter()
            .map(|exp_opt| {
                exp_opt.as_ref()
                    .map(|exp| {
                        let val = exp.eval(global_sym);
                        program.new_value().integer(val)
                    })
                    .unwrap_or(zero)
            })
            .collect();
        program.new_value().aggregate(inits)
    }
}

/// 处理全局声明（变量和常量）
fn process_global_decl(decl: &Decl, program: &mut ir::Program, global_sym: &mut SymbolTable) {
    match decl {
        Decl::Var(decl) => {
            for c in &decl.var_defs {
                process_var_def(c, program, global_sym);
            }
        },
        Decl::Const(decl) => {
            for c in &decl.const_defs {
                if c.size.is_empty() {
                    let ConstInitVal::Var(const_exp) = &c.const_val else {
                        panic!("Excepted ConstInitVal::Var")
                    };
                    let v = const_exp.exp.eval(&global_sym);
                    global_sym.insert_const(c.ident.clone(), v);
                } else {
                    // 多维数组
                    let sizes: Vec<usize> = c.size.iter()
                        .map(|const_exp| const_exp.exp.eval(&global_sym) as usize)
                        .collect();
                    let total_size = sizes.iter().fold(1, |acc, &x| acc * x);
                    let var_name = format!("@{}", c.ident);
                    
                    let ConstInitVal::Array(ref init_vals) = c.const_val else {
                        panic!("Expected InitVal::Array")
                    };
                    
                    // 展平初始化列表
                    let flattened = process_dimensions(&sizes, init_vals);
                    let init = make_array_init(program, &flattened, total_size, global_sym);
                    let global_var = make_global_alloc(program, Some(init));
                    program.set_value_name(global_var, Some(var_name));
                    global_sym.insert_array(&c.ident, global_var, sizes); 
                }
            }
        }
    }
}

fn process_var_def(c: &VarDef, program: &mut ir::Program, global_sym: &mut SymbolTable) {
    match c {
        VarDef::Decl(name, size) => {
            let var_name = format!("@{}", name);
            let global_var;
            if size.is_empty() {
                global_var = make_global_alloc(program, None);
                global_sym.insert_var(&name, global_var);
            } else {
                let sizes: Vec<_> = size.iter().map(|const_exp| {
                    const_exp.exp.eval(&global_sym) as usize
                }).collect();
                let size = sizes.iter().fold(1, |acc, &x| acc * x);
                let array_type = ir::Type::get_array(ir::Type::get_i32(), size);
                let zero_init = program.new_value().zero_init(array_type);
                global_var = make_global_alloc(program, Some(zero_init));
                global_sym.insert_array(&name, global_var, sizes);
            }
            // 设置变量名，便于调试
            program.set_value_name(global_var, Some(var_name));
        },
        VarDef::Init(name, size, val) => {
            let var_name = format!("@{}", name);
            let global_var;
            if size.is_empty() {
                let InitVal::Var(exp) = val else {
                    panic!("Expected InitVal::Var")
                };
                let const_val = exp.eval(&global_sym);
                let init = program.new_value().integer(const_val);
                global_var = make_global_alloc(program, Some(init));
                global_sym.insert_var(&name, global_var);
            } else {
                let sizes: Vec<_> = size.iter()
                    .map(|const_exp| { const_exp.exp.eval(&global_sym) as usize })
                    .collect();
                let total_size = sizes.iter().fold(1, |acc, &x| acc * x) as usize;
                let InitVal::Array(exps) = val else {
                    panic!("Expected InitVal::Array")
                };
                
                let flattened = process_dimensions(&sizes, exps);
                let init = make_array_init(program, &flattened, total_size, global_sym);
                global_var = make_global_alloc(program, Some(init));
                global_sym.insert_array(&name, global_var, sizes);
            }
            // 设置变量名，便于调试
            program.set_value_name(global_var, Some(var_name));
        }
    }
}
