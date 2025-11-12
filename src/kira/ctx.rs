use core::panic;
use std::collections::VecDeque;

use super::symbol::SymbolTable;
use crate::ast::LVal;
use koopa::ir::{self as ir, Type, Value};
use koopa::ir::builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder};
use super::symbol::Sym;
use super::expr::emit_ast_exp;

// 轻量 IR 上下文，内部方法只做短借用
pub struct FuncCtx<'a> {
    pub func: &'a mut ir::FunctionData,
    pub bb: ir::BasicBlock,
    pub sym: &'a mut SymbolTable,
    pub bb_counter: usize,
    pub loop_stack: VecDeque<LoopContext>,
}

pub struct LoopContext {
    pub entry_bb: ir::BasicBlock,
    pub end_bb: ir::BasicBlock,
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
        // Evaluate index expression first (if any) to avoid borrowing self.sym
        // immutably while we later need a mutable borrow for emitting code.
        let index_val_opt = if let Some(index) = &v.index.as_ref() {
            Some(emit_ast_exp(index, self))
        } else {
            None
        };

        let sym = self.sym.get_all_sym(&v.ident).expect("Symbol not found");
        match sym {
            Sym::Const(number) => {
                Some(self.func.dfg_mut().new_value().integer(*number))
            },
            Sym::Var(alloc) => {
                Some(self.make_load(*alloc))
            },
            Sym::Array(alloc) => {
                let index_val = index_val_opt.expect("Array access without index");
                let target = self.make_getelemptr(*alloc, index_val);
                Some(self.make_load(target))
            }
            Sym::Func(_) => {
                panic!("Call a function without '()'!")
            }
        }
    }

    // 可以顺便设置一下变量名
    pub fn make_alloc(&mut self, ty: Type, var_name: Option<String>) -> ir::Value {
        let v = self.func.dfg_mut().new_value().alloc(ty);
        self.func.dfg_mut().set_value_name(v, var_name);
        self.push_inst(v);
        v
    }

    pub fn make_array_alloc(&mut self, array_name: Option<String>, size: usize) -> ir::Value {
        let array_type = ir::Type::get_array(ir::Type::get_i32(), size);
        self.make_alloc(array_type, array_name)
    }

    pub fn make_array_init<T: super::const_eval::EvalConst>(
        &mut self, 
        array_name: Option<String>, 
        size: usize,
        exps: &[T],
    ) -> ir::Value {
        let array = self.make_array_alloc(array_name, size);
        for (index, exp) in exps.iter().enumerate() {
            let index = self.make_int(index as i32);
            let index_alloc = self.make_getelemptr(array, index);
            let value = exp.eval(self.sym);
            let val = self.make_int(value);
            self.make_store(index_alloc, val);
        }
        if exps.len() < size {
            let zero = self.make_int(0);
            for i in exps.len()..size {
                let index_val = self.make_int(i as i32);
                let index_alloc = self.make_getelemptr(array, index_val);
                self.make_store(index_alloc, zero);
            }
        }
        array
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

    pub fn make_getelemptr(&mut self, src: ir::Value, index: ir::Value) -> ir::Value {
        let v = self.func.dfg_mut().new_value().get_elem_ptr(src, index);
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

    pub fn make_call(&mut self, callee: ir::Function, args: Vec<ir::Value>) -> Value {
        let v = self.func.dfg_mut().new_value().call(callee, args);
        self.push_inst(v);
        v
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

    pub fn enter_loop(&mut self, entry: ir::BasicBlock, end: ir::BasicBlock) {
        self.loop_stack.push_back(LoopContext {
            entry_bb: entry,
            end_bb: end,
        });
    }

    pub fn exit_loop(&mut self) {
        self.loop_stack.pop_back()
            .expect("exit_loop called outside loop");
    }

    pub fn current_loop_entry(&self) -> ir::BasicBlock {
        self.loop_stack.back()
            .expect("break/continue outside loop")
            .entry_bb
    }

    pub fn current_loop_end(&self) -> ir::BasicBlock {
        self.loop_stack.back()
            .expect("break/continue outside loop")
            .end_bb
    }

    // 判断一个值是否已经是 bool，好像没用啊
    pub fn _is_bool(&mut self, v: ir::Value) -> bool {
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

    // 检查当前块是否已结束，Provided by Claude.
    pub fn is_bb_terminated(&self) -> bool {
        self.func.layout()
            .bbs()
            .node(&self.bb)
            .and_then(|node| {
                node.insts().back_key().map(|&inst| {
                    let kind = self.func.dfg().value(inst).kind();
                    matches!(kind,
                        ir::ValueKind::Return(_) |
                        ir::ValueKind::Jump(_) |
                        ir::ValueKind::Branch(_)
                    )
                })
            })
            .unwrap_or(false)
    }
}