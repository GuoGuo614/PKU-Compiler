// 基本块和块项处理

use koopa::ir;

use crate::ast::*;
use super::ctx::FuncCtx;
use super::const_eval::EvalConst;
use super::stmt::process_stmt;
use super::expr::emit_ast_exp;

/// 处理块
pub fn process_block(block: &Block, ctx: &mut FuncCtx) {
    ctx.sym.enter_scope();

    for block_item in &block.block_items {
        process_block_item(block_item, ctx);
    }

    ctx.sym.exit_scope();
}

fn process_block_item(item: &BlockItem, ctx: &mut FuncCtx) {
    match item {
        BlockItem::Stmt(stmt) => {
            process_stmt(stmt, ctx);
        },
        BlockItem::Decl(Decl::Const(decl)) => {
            for c in &decl.const_defs {
                if c.size.is_none() {
                    let ConstInitVal::Var(const_exp) = &c.const_val else {
                        panic!("Excepted ConstInitVal::Var")
                    };
                    let v = const_exp.exp.eval(ctx.sym);
                    ctx.sym.insert_const(c.ident.clone(), v);
                } else {
                    let ConstInitVal::Array(ref exps) = c.const_val else {
                        panic!("Expected InitVal::Array")
                    };
                    // 局部数组，及其初始化
                    let size = c.size.as_ref().unwrap().exp.eval(&ctx.sym) as usize;
                    let array_name = format!("@{}", c.ident);
                    let alloc = ctx.make_array_init(Some(array_name), size, exps);
                    ctx.sym.insert_array(&c.ident, alloc); 
                }
            }
        },
        BlockItem::Decl(Decl::Var(decl)) => {
            for d in &decl.var_defs {
                process_var_def(d, ctx);
            }
        }
    }
}

fn process_var_def(var_def: &VarDef, ctx: &mut FuncCtx) {
    match var_def {
        VarDef::Decl(name, size) => {
            if size.is_none() {
                let alloc = ctx.make_alloc(ir::Type::get_i32(), Some(format!("%{}", name)));
                ctx.sym.insert_var(name, alloc);
            } else {
                let size = size.as_ref().unwrap().exp.eval(&ctx.sym) as usize;
                let array_name = format!("@{}", name);
                let alloc = ctx.make_array_alloc(Some(array_name), size);
                ctx.sym.insert_array(&name, alloc); 
            }
        },
        VarDef::Init(name, size, val) => {
            if size.is_none() {
                let InitVal::Var(exp) = &val else {
                    panic!("Excepted ConstInitVal::Var")
                };
                let alloc = ctx.make_alloc(ir::Type::get_i32(), Some(format!("%{}", name)));
                ctx.sym.insert_var(name, alloc);
                let rhs = emit_ast_exp(exp, ctx);
                ctx.make_store(alloc, rhs);
            } else {
                let InitVal::Array(ref exps) = val else {
                    panic!("Expected InitVal::Array")
                };
                // 局部数组，及其初始化
                let size = size.as_ref().unwrap().exp.eval(&ctx.sym) as usize;
                let array_name = format!("@{}", name);
                let alloc = ctx.make_array_init(Some(array_name), size, exps);
                ctx.sym.insert_array(&name, alloc); 
            }
        }
    }
}
