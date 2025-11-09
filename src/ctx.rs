use crate::symbol::SymbolTable;
use crate::ast::LVal;
use koopa::ir::{self as ir, Type};
use koopa::ir::builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder};
use crate::symbol::Sym;

// 轻量 IR 上下文，内部方法只做短借用
pub struct FuncCtx<'a> {
    pub func: &'a mut ir::FunctionData,
    pub bb: ir::BasicBlock,
    pub sym: &'a mut SymbolTable,
    pub bb_counter: usize,
}

impl<'a> FuncCtx<'a> {
    pub fn gen_bb_name(&mut self, prefix: &str) -> String {
        let name = format!("{}{}", prefix, self.bb_counter);
        self.bb_counter += 1;
        name
    }
    
    pub fn new_bb(&mut self, prefix: &str) -> ir::BasicBlock {
        let name = self.gen_bb_name(prefix);
        let bb = self.func.dfg_mut().new_bb().basic_block(Some(name));
        self.func.layout_mut().bbs_mut()
            .push_key_back(bb)
            .expect("Failed to push basic block");
        bb
    }
    
    pub fn make_int(&mut self, v: i32) -> ir::Value {
        self.func.dfg_mut().new_value().integer(v)
    }

    pub fn make_val(&mut self, v: &LVal) -> Option<ir::Value> {
        match self.sym.get_const_var(&v.ident).expect("Symbol not found") {
            Sym::Const(number) => {
                Some(self.func.dfg_mut().new_value().integer(*number))
            },
            Sym::Var(alloc) => {
                Some(self.make_load(*alloc))
            }
        }
    }

    pub fn make_alloc(&mut self) -> ir::Value {
        let v = self.func.dfg_mut().new_value().alloc(Type::get_i32());
        self.push_inst(v);
        v
    }

    pub fn make_store(&mut self, alloc: ir::Value, val: ir::Value) {
        let v = self.func.dfg_mut().new_value().store(val, alloc);
        self.push_inst(v);
    }

    pub fn make_load(&mut self, src: ir::Value) -> ir::Value {
        let v = self.func.dfg_mut().new_value().load(src);
        self.push_inst(v);
        v
    }

    pub fn make_branch(
        &mut self, 
        cond: ir::Value, 
        true_bb: ir::BasicBlock, 
        false_bb: ir::BasicBlock
    ) {
        let v = self.func.dfg_mut().new_value().branch(cond, true_bb, false_bb);
        self.push_inst(v);
    }

    pub fn make_jump(&mut self, target: ir::BasicBlock) {
        let v = self.func.dfg_mut().new_value().jump(target);
        self.push_inst(v);
    }

    pub fn switch_to_bb(&mut self, new_bb: ir::BasicBlock) {
        self.bb = new_bb;
    }

    pub fn emit_binary(&mut self, op: ir::BinaryOp, lhs: ir::Value, rhs: ir::Value) -> ir::Value {
        let v = self.func.dfg_mut().new_value().binary(op, lhs, rhs);
        self.push_inst(v);
        v
    }

    pub fn emit_ret(&mut self, v: Option<ir::Value>) {
        let inst = self.func.dfg_mut().new_value().ret(v);
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