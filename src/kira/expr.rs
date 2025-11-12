// 表达式生成

use koopa::*;
use crate::ast::*;
use super::ctx::FuncCtx;

/// 表达式生成（基于上下文）
pub fn emit_ast_exp(exp: &Exp, ctx: &mut FuncCtx) -> ir::Value {
    emit_ast_lor(&exp.lor_exp, ctx)
}

/// 将整数转为布尔代数，好像没用啊
fn _to_bool(ctx: &mut FuncCtx, v: ir::Value) -> ir::Value {
    if ctx._is_bool(v) {
        return v;
    }
    let zero = ctx.make_int(0);
    let is_zero = ctx.emit_binary(ir::BinaryOp::Eq, v, zero); // 1 if v == 0
    ctx.emit_binary(ir::BinaryOp::Eq, is_zero, zero) // 1 if v != 0
}

// lor 和 land 实现了短路求值
fn emit_ast_lor(lor: &LOrExp, ctx: &mut FuncCtx) -> ir::Value {
    match lor {
        LOrExp::And(land) => emit_ast_land(land, ctx),
        LOrExp::Or(l, r) => {
            let result_alloc = ctx.make_alloc(ir::Type::get_i32(), Some("%result".to_string()));
            let bb_true = ctx.new_bb("%or_true");
            let bb_false = ctx.new_bb("%or_false");
            let bb_end = ctx.new_bb("%or_end");
            
            let lv = emit_ast_lor(l, ctx);
            ctx.make_branch(lv, bb_true, bb_false);

            ctx.switch_to_bb(bb_true);
            let one = ctx.make_int(1);
            ctx.make_store(result_alloc, one);
            ctx.make_jump(bb_end);
            
            ctx.switch_to_bb(bb_false);
            let rv = emit_ast_land(r, ctx);
            // let rv_bool = to_bool(ctx, rv);
            ctx.make_store(result_alloc, rv);
            ctx.make_jump(bb_end);

            ctx.switch_to_bb(bb_end);
            ctx.make_load(result_alloc)
        }
    }
}

fn emit_ast_land(land: &LAndExp, ctx: &mut FuncCtx) -> ir::Value {
    match land {
        LAndExp::Eq(eq) => {
            emit_ast_eq(eq, ctx)
        }
        LAndExp::And(l, r) => {
            let result_alloc = ctx.make_alloc(ir::Type::get_i32(), Some("%result".to_string()));
            
            let bb_true = ctx.new_bb("%and_true");
            let bb_false = ctx.new_bb("%and_false");
            let bb_end = ctx.new_bb("%and_end");
            
            let lv = emit_ast_land(l, ctx);
            ctx.make_branch(lv, bb_true, bb_false);
            
            ctx.switch_to_bb(bb_true);
            let rv = emit_ast_eq(r, ctx);
            // let rv_bool = to_bool(ctx, rv);
            ctx.make_store(result_alloc, rv);
            ctx.make_jump(bb_end);
            
            ctx.switch_to_bb(bb_false);
            let zero = ctx.make_int(0);
            ctx.make_store(result_alloc, zero);
            ctx.make_jump(bb_end);
            
            ctx.switch_to_bb(bb_end);
            ctx.make_load(result_alloc)
        }
    }
}

fn emit_ast_eq(eq: &EqExp, ctx: &mut FuncCtx) -> ir::Value {
    match eq {
        EqExp::Rel(rel) => emit_ast_rel(rel, ctx),
        EqExp::Eq(l, r) => {
            let lv = emit_ast_eq(l, ctx);
            let rv = emit_ast_rel(r, ctx);
            ctx.emit_binary(ir::BinaryOp::Eq, lv, rv)
        }
        EqExp::Neq(l, r) => {
            let lv = emit_ast_eq(l, ctx);
            let rv = emit_ast_rel(r, ctx);
            ctx.emit_binary(ir::BinaryOp::NotEq, lv, rv)
        }
    }
}

fn emit_ast_rel(rel: &RelExp, ctx: &mut FuncCtx) -> ir::Value {
    match rel {
        RelExp::Add(add) => emit_ast_add(add, ctx),
        RelExp::Lt(l, r) => {
            let lv = emit_ast_rel(l, ctx);
            let rv = emit_ast_add(r, ctx);
            ctx.emit_binary(ir::BinaryOp::Lt, lv, rv)
        }
        RelExp::Gt(l, r) => {
            let lv = emit_ast_rel(l, ctx);
            let rv = emit_ast_add(r, ctx);
            ctx.emit_binary(ir::BinaryOp::Gt, lv, rv)
        }
        RelExp::Ge(l, r) => {
            let lv = emit_ast_rel(l, ctx);
            let rv = emit_ast_add(r, ctx);
            ctx.emit_binary(ir::BinaryOp::Ge, lv, rv)
        }
        RelExp::Le(l, r) => {
            let lv = emit_ast_rel(l, ctx);
            let rv = emit_ast_add(r, ctx);
            ctx.emit_binary(ir::BinaryOp::Le, lv, rv)
        }
    }
}

fn emit_ast_add(add: &AddExp, ctx: &mut FuncCtx) -> ir::Value {
    match add {
        AddExp::Mul(mul_exp) => {
            emit_ast_mul(mul_exp, ctx)
        },
        AddExp::Sub(add_exp, mul_exp) => {
            let left = emit_ast_add(add_exp, ctx);
            let right = emit_ast_mul(mul_exp, ctx);
            ctx.emit_binary(ir::BinaryOp::Sub, left, right)
        },
        AddExp::Add(add_exp, mul_exp) => {
            let left = emit_ast_add(add_exp, ctx);
            let right = emit_ast_mul(mul_exp, ctx);
            ctx.emit_binary(ir::BinaryOp::Add, left, right)
        },
    }
}

fn emit_ast_mul(mul: &MulExp, ctx: &mut FuncCtx) -> ir::Value {
    match mul {
        MulExp::Unary(unary) => {
            emit_ast_unary(unary, ctx)
        },
        MulExp::Mul(mul_exp, unary) => {
            let left = emit_ast_mul(mul_exp, ctx);
            let right = emit_ast_unary(unary, ctx);
            ctx.emit_binary(ir::BinaryOp::Mul, left, right)
        },
        MulExp::Div(mul_exp, unary) => {
            let left = emit_ast_mul(mul_exp, ctx);
            let right = emit_ast_unary(unary, ctx);
            ctx.emit_binary(ir::BinaryOp::Div, left, right)
        },
        MulExp::Mod(mul_exp, unary) => {
            let left = emit_ast_mul(mul_exp, ctx);
            let right = emit_ast_unary(unary, ctx);
            ctx.emit_binary(ir::BinaryOp::Mod, left, right)
        },
    }
}

fn emit_ast_unary(unary: &UnaryExp, ctx: &mut FuncCtx) -> ir::Value {
    match unary {
        UnaryExp::Primary(p) => emit_ast_primary(p, ctx),
        UnaryExp::Positive(u) => emit_ast_unary(u, ctx),
        UnaryExp::Negative(u) => {
            let zero = ctx.make_int(0);
            let v = emit_ast_unary(u, ctx);
            ctx.emit_binary(ir::BinaryOp::Sub, zero, v)
        },
        UnaryExp::Not(u) => {
            let zero = ctx.make_int(0);
            let v = emit_ast_unary(u, ctx);
            ctx.emit_binary(ir::BinaryOp::Eq, v, zero)
        },
        UnaryExp::FuncCall(ident, r_params) => {
            let func_name = format!("@{}", ident);
            let func = ctx.sym.get_func(&func_name)
                .expect("Function {} not Found")
                .clone();
            let params_values: Vec<_> = r_params.as_ref()
                .into_iter()
                .flat_map(|p| &p.params)
                .map(|exp| emit_ast_exp(exp, ctx))
                .collect();

            ctx.make_call(func, params_values)
        }
    }
}

fn emit_ast_primary(p: &PrimaryExp, ctx: &mut FuncCtx) -> ir::Value {
    match p {
        PrimaryExp::Number(n) => ctx.make_int(*n),
        PrimaryExp::Paren(e) => emit_ast_exp(e, ctx),
        PrimaryExp::LVal(lval) => ctx.make_val(lval).unwrap()
    }
}
