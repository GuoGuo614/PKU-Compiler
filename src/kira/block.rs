// 基本块和块项处理

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
                let v = c.const_val.const_exp.exp.eval(&ctx.sym);
                ctx.sym.insert_const(c.ident.clone(), v);
            }
        },
        BlockItem::Decl(Decl::Var(decl)) => {
            for c in &decl.var_defs {
                process_var_def(c, ctx);
            }
        }
    }
}

fn process_var_def(var_def: &VarDef, ctx: &mut FuncCtx) {
    match var_def {
        VarDef::Decl(name) => {
            let alloc = ctx.make_alloc(Some(format!("%{}", name)));
            ctx.sym.insert_var(name, alloc);
        },
        VarDef::Init(name, val) => {
            let alloc = ctx.make_alloc(Some(format!("%{}", name)));
            ctx.sym.insert_var(name, alloc);
            let rhs = emit_ast_exp(&val.exp, ctx);
            ctx.make_store(alloc, rhs);
        }
    }
}
