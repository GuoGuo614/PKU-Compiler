// 语句处理

use crate::ast::*;
use super::ctx::FuncCtx;
use super::block::process_block;
use super::expr::emit_ast_exp;

/// 处理语句
pub fn process_stmt(stmt: &Stmt, ctx: &mut FuncCtx) {
    // 当前块已经结束，无需处理语句
    if ctx.is_bb_terminated() {
        return;
    }
    match stmt {
        Stmt::Return(exp) => {
            let v = exp.as_ref()
                .map(|e| emit_ast_exp(e, ctx));
            ctx.emit_ret(v);
        },
        Stmt::Assign(lval, exp) => {
            // 可优化：先尝试常量求值
            let rhs = emit_ast_exp(exp, ctx);
            let alloc = ctx.sym.get_var(&lval.ident)
                .expect("Variable not found");
            ctx.make_store(*alloc, rhs);
        },
        Stmt::Exp(Some(_exp)) => {
            // Do nothing currently. Later I will use it.
            emit_ast_exp(_exp, ctx);
        },
        Stmt::Exp(None) => {
            // Do nothing.
        },
        Stmt::Block(block) => {
            process_block(block, ctx);
        },
        Stmt::If(cond, body, else_body) => {
            process_if(cond, body, else_body.as_deref(), ctx);
        },
        Stmt::While(cond, body) => {
            process_while(cond, body, ctx);
        },
        Stmt::Break => {
            let while_end = ctx.current_loop_end();
            ctx.make_jump(while_end);
        },
        Stmt::Continue => {
            let while_entry = ctx.current_loop_entry();
            ctx.make_jump(while_entry);
        }
    }
}

/// 处理 if 语句
fn process_if(
    cond: &Exp,
    then_stmt: &Stmt,
    else_stmt: Option<&Stmt>,
    ctx: &mut FuncCtx,
) {
    let bb_then = ctx.new_bb("%then");
    let bb_else = else_stmt.as_ref().map(|_| ctx.new_bb("%else"));
    let bb_end = ctx.new_bb("%end");

    let exp_val = emit_ast_exp(cond, ctx);
    // let bool_val = to_bool(ctx, exp_val);

    let target_false = bb_else.unwrap_or(bb_end);
    ctx.make_branch(exp_val, bb_then, target_false);

    ctx.switch_to_bb(bb_then);
    process_stmt(then_stmt, ctx);
    if !ctx.is_bb_terminated() {
        ctx.make_jump(bb_end);
    }

    if let Some((bb, stmt)) = bb_else.zip(else_stmt) {
        ctx.switch_to_bb(bb);
        process_stmt(stmt, ctx);
        if !ctx.is_bb_terminated() {
            ctx.make_jump(bb_end);
        }
    }

    ctx.switch_to_bb(bb_end);
}

/// 处理 while 语句
fn process_while(cond: &Exp, body_stmt: &Stmt, ctx: &mut FuncCtx) {
    let bb_entry = ctx.new_bb("%while_entry");
    let bb_body = ctx.new_bb("%while_body");
    let bb_jump_body = ctx.new_bb("%while_body");
    let bb_end = ctx.new_bb("%end");
    
    ctx.enter_loop(bb_entry, bb_end);
    ctx.make_jump(bb_entry);

    ctx.switch_to_bb(bb_entry);
    let cond_val = emit_ast_exp(cond, ctx);
    ctx.make_branch(cond_val, bb_body, bb_end);

    ctx.switch_to_bb(bb_jump_body);
    ctx.make_jump(bb_entry);

    ctx.switch_to_bb(bb_body);
    process_stmt(body_stmt, ctx);

    if !ctx.is_bb_terminated() {
        ctx.make_jump(bb_jump_body);
    }

    ctx.switch_to_bb(bb_end);
    ctx.exit_loop();
}
