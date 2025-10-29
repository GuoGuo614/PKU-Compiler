use koopa::{ir::builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder, ValueInserter}, *};
use koopa::back::KoopaGenerator;
use crate::ast::*;

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

    parse_global_values(&mut glo_builder);
    parse_function_from_ast(comp, &mut program);

    program
}

fn parse_global_values(glo_builder: &mut ir::builder::GlobalBuilder<'_>) {
    let global_values = values_parse_from_ast();
    for global_value in global_values {
        glo_builder.insert_value(global_value);
    }
}

fn parse_function_from_ast(comp: &CompUnit, program: &mut ir::Program) {
    let func_type = functype_parse_from_ast(&comp.func_def.func_type);
    let func_name = format!("@{}", comp.func_def.ident);

    let func_data = ir::FunctionData::new(func_name.clone(), Vec::new(), func_type);
    let func_handle = program.new_func(func_data);

    // 解析函数体
    parse_function_body(&comp.func_def, program, func_handle);
}

// 为了避免借用检查写出了比较丑的代码，使用 GPT 重构下
fn parse_function_body(func_def: &FuncDef, program: &mut ir::Program, func_handle: ir::Function) {
    let func = program.func_mut(func_handle);

    // 1) 短借用 dfg 创建 entry 基本块
    let bb = {
        let dfg = func.dfg_mut();
        let bb = dfg.new_bb().basic_block(Some(String::from("%entry")));
        // 2) 短借用 layout 将 bb 放入布局
        {
            let layout = func.layout_mut();
            layout.bbs_mut().push_key_back(bb).expect("Failed to push basic block");
        }
        bb
    };

    // 3) 用上下文封装对 func 的操作，避免重叠借用
    let mut ctx = FuncCtx { func, bb };

    let Stmt::Return(exp) = &func_def.block.stmt;
    let ret_val = emit_ast_exp(exp, &mut ctx);
    ctx.emit_ret(ret_val);
}

// 轻量 IR 上下文，内部方法只做“短借用”
// 我的天哪 GPT 大人
struct FuncCtx<'a> {
    func: &'a mut ir::FunctionData,
    bb: ir::BasicBlock,
}

impl<'a> FuncCtx<'a> {
    fn make_int(&mut self, v: i32) -> ir::Value {
        self.func.dfg_mut().new_value().integer(v)
    }

    fn emit_binary(&mut self, op: ir::BinaryOp, lhs: ir::Value, rhs: ir::Value) -> ir::Value {
        let v = self.func.dfg_mut().new_value().binary(op, lhs, rhs);
        self.push_inst(v);
        v
    }

    fn emit_ret(&mut self, val: ir::Value) {
        let inst = self.func.dfg_mut().new_value().ret(Some(val));
        self.push_inst(inst);
    }

    fn push_inst(&mut self, v: ir::Value) {
        self.func
            .layout_mut()
            .bb_mut(self.bb)
            .insts_mut()
            .push_key_back(v)
            .expect("Failed to push instruction");
    }
}

// 表达式生成（基于上下文）
fn emit_ast_exp(exp: &Exp, ctx: &mut FuncCtx) -> ir::Value {
    emit_ast_add(&exp.add_exp, ctx)
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
    }
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
