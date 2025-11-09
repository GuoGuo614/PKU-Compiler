use crate::symbol::SymbolTable;

use koopa::{ir::builder::{BasicBlockBuilder, ValueInserter}, *};
use koopa::back::KoopaGenerator;
use crate::ast::*;

use crate::const_eval::EvalConst;
use crate::ctx::FuncCtx;

pub trait GenerateIR {
    fn generate_ir(&self) -> ir::Program;
    fn write_ir(&self, output: String);
}

impl GenerateIR for CompUnit {
    fn write_ir(&self, output: String) {
        let program = program_parse_from_ast(self);
        let mut gen = KoopaGenerator::new(Vec::new());
        gen.generate_on(&program).expect("koopa serialize failed");
        std::fs::write(output, gen.writer()).expect("write ir file failed");
    }

    fn generate_ir(&self) -> ir::Program {
        program_parse_from_ast(self)
    }
}

pub fn program_parse_from_ast(comp: &CompUnit) -> ir::Program {
    let mut program = ir::Program::new();
    let mut glo_builder: ir::builder::GlobalBuilder<'_> = program.new_value();

    let mut sym = SymbolTable::new();

    parse_global_values(&mut glo_builder, &mut sym);
    parse_function_from_ast(comp, &mut program, &mut sym);

    program
}

fn parse_global_values(glo_builder: &mut ir::builder::GlobalBuilder<'_>, sym: &mut SymbolTable) {
    let global_values = values_parse_from_ast();
    for global_value in global_values {
        glo_builder.insert_value(global_value);
    }
}

fn parse_function_from_ast(comp: &CompUnit, program: &mut ir::Program, sym: &mut SymbolTable) {
    let func_type = functype_parse_from_ast(&comp.func_def.func_type);
    let func_name = format!("@{}", comp.func_def.ident);

    let func_data = ir::FunctionData::new(func_name.clone(), Vec::new(), func_type);
    let func_handle = program.new_func(func_data);

    // 解析函数体
    parse_function_body(&comp.func_def, program, func_handle, sym);
}

// 为了避免借用检查写出了比较丑的代码，使用 GPT 重构下
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
    let mut ctx = FuncCtx { func, bb, sym, bb_counter: 0 };

    process_block(&func_def.block, &mut ctx);
}

fn process_block(block: &Block, ctx: &mut FuncCtx) {
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
                match c {
                    VarDef::Decl(name) => {
                        let alloc = ctx.make_alloc();
                        ctx.sym.insert_var(name, alloc);
                    },
                    VarDef::Init(name, val) => {
                        let alloc = ctx.make_alloc();
                        ctx.sym.insert_var(name, alloc);
                        let rhs = emit_ast_exp(&val.exp, ctx);
                        ctx.make_store(alloc, rhs);
                    }
                }
            }
        }
    }
}

fn process_stmt(stmt: &Stmt, ctx: &mut FuncCtx) {
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
            // Do nothing. Is it OK?
            // Later I will use it.
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
    }
}

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
    ctx.make_jump(bb_end);

    if let Some((bb, stmt)) = bb_else.zip(else_stmt) {
        ctx.switch_to_bb(bb);
        process_stmt(stmt, ctx);
        ctx.make_jump(bb_end);
    }

    ctx.switch_to_bb(bb_end);
}

// 表达式生成（基于上下文）
fn emit_ast_exp(exp: &Exp, ctx: &mut FuncCtx) -> ir::Value {
    emit_ast_lor(&exp.lor_exp, ctx)
}

// 将整数转为布尔代数
fn to_bool(ctx: &mut FuncCtx, v: ir::Value) -> ir::Value {
    if ctx.is_bool(v) {
        return v;
    }
    let zero = ctx.make_int(0);
    let is_zero = ctx.emit_binary(ir::BinaryOp::Eq, v, zero); // 1 if v == 0
    ctx.emit_binary(ir::BinaryOp::Eq, is_zero, zero) // 1 if v != 0
}

fn emit_ast_lor(lor: &LOrExp, ctx: &mut FuncCtx) -> ir::Value {
    match lor {
        LOrExp::And(land) => emit_ast_land(land, ctx),
        LOrExp::Or(l, r) => {
            let l_raw = emit_ast_lor(l, ctx);
            let lv = to_bool(ctx, l_raw);
            let r_raw = emit_ast_land(r, ctx);
            let rv = to_bool(ctx, r_raw);

            ctx.emit_binary(ir::BinaryOp::Or, lv, rv)
        }
    }
}

fn emit_ast_land(land: &LAndExp, ctx: &mut FuncCtx) -> ir::Value {
    match land {
        LAndExp::Eq(eq) => {
            emit_ast_eq(eq, ctx)
        }
        LAndExp::And(l, r) => {
            let l_raw = emit_ast_land(l, ctx);
            let lv = to_bool(ctx, l_raw);
            let r_raw = emit_ast_eq(r, ctx);
            let rv = to_bool(ctx, r_raw);
            ctx.emit_binary(ir::BinaryOp::And, lv, rv)
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
        }
        UnaryExp::Not(u) => {
            let zero = ctx.make_int(0);
            let v = emit_ast_unary(u, ctx);
            ctx.emit_binary(ir::BinaryOp::Eq, v, zero)
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

pub fn values_parse_from_ast() -> Vec<ir::entities::ValueData> {
    let result = Vec::new();
    result
}

pub fn functype_parse_from_ast(functype: &FuncType) -> ir::Type {
    match functype._type.as_str() {
        "int" => ir::Type::get_i32(),
        _ => panic!("Unknown function types.")
    }
}
