use crate::symbol::SymbolTable;
use crate::ast::LVal;
use koopa::ir as ir;
use koopa::ir::builder::{LocalInstBuilder, ValueBuilder};

// 轻量 IR 上下文，内部方法只做“短借用”
// 我的天哪 GPT 大人
pub struct FuncCtx<'a> {
    pub func: &'a mut ir::FunctionData,
    pub bb: ir::BasicBlock,
    pub sym: &'a mut SymbolTable
}

impl<'a> FuncCtx<'a> {
    pub fn make_int(&mut self, v: i32) -> ir::Value {
        self.func.dfg_mut().new_value().integer(v)
    }

    pub fn make_val(&mut self, v: &LVal) -> ir::Value {
        let number = self.sym.get_symbol_value(&v.ident);
        self.func.dfg_mut().new_value().integer(*number.unwrap())
    }

    pub fn emit_binary(&mut self, op: ir::BinaryOp, lhs: ir::Value, rhs: ir::Value) -> ir::Value {
        let v = self.func.dfg_mut().new_value().binary(op, lhs, rhs);
        self.push_inst(v);
        v
    }

    pub fn emit_ret(&mut self, v: ir::Value) {
        let inst = self.func.dfg_mut().new_value().ret(Some(v));
        self.push_inst(inst);
    }

    pub fn push_inst(&mut self, v: ir::Value) {
        self.func
            .layout_mut()
            .bb_mut(self.bb)
            .insts_mut()
            .push_key_back(v)
            .expect("Failed to push instruction");
    }

    // 判断一个值是否已经是布尔（0/1）
    pub fn is_bool(&mut self, v: ir::Value) -> bool {
        use koopa::ir::ValueKind;
        let kind = self.func.dfg().value(v).kind();
        match kind {
            ValueKind::Integer(int) => int.value() == 0 || int.value() == 1,
            ValueKind::Binary(bin) => {
                matches!(bin.op(), ir::BinaryOp::Eq| ir::BinaryOp::Lt 
                | ir::BinaryOp::Le | ir::BinaryOp::Gt | ir::BinaryOp::Ge | ir::BinaryOp::NotEq)
            }
            _ => false,
        }
    }
}