use std::io::Write;
use koopa::ir::ValueKind;
use super::AsmCtx;
use super::ir;
use super::utils::calculate_type_size;

static PARAM_REGS: [&str; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

impl<'a, W: Write> AsmCtx<'a, W> {
    pub fn emit_inst(&mut self, inst: ir::Value) {
        let v = self.func.dfg().value(inst);
        match v.kind() {
            ValueKind::Binary(bin) => self.emit_binary_inst(inst, bin),
            ValueKind::Return(ret) => self.emit_return(ret),
            ValueKind::Alloc(_) => self.emit_alloc(inst),
            ValueKind::Load(load) => self.emit_load(inst, load),
            ValueKind::Store(store) => self.emit_store(store),
            ValueKind::Branch(branch) => self.emit_branch(branch),
            ValueKind::Jump(jump) => self.emit_jump(jump),
            ValueKind::Call(call) => self.emit_call(inst, call),
            ValueKind::GetElemPtr(get_ptr) => self.emit_get_elem_ptr(inst, get_ptr),
            ValueKind::GetPtr(get_ptr) => self.emit_get_ptr(inst, get_ptr),
            _ => panic!("Unsupported value kind: {:?}", v.kind()),
        }
    }

    fn emit_binary_inst(&mut self, inst: ir::Value, bin: &ir::values::Binary) {
        let lhs = bin.lhs();
        let rhs = bin.rhs();
        let rl = self.reg_for(lhs);
        let rr = self.reg_for(rhs);
        let rd = self.ra.alloc();

        super::binary::emit_binary_op(self, bin.op(), rd, rl, rr);

        self.ra.dec_use(lhs);
        self.ra.dec_use(rhs);
        
        let offset = self.sf.alloc_slot(inst);
        self.store_to_stack(rd, offset);
        self.ra.release(rd);
    }

    fn emit_return(&mut self, ret: &ir::values::Return) {
        if let Some(v) = ret.value() {
            self.specific_reg_for(v, "a0");
        }
        if self.has_call {
            if self.sf.total - 4 < 2048 {
                writeln!(self.w, "\tlw ra, {}(sp)", self.sf.total - 4).unwrap();
            } else {
                writeln!(self.w, "\tli t0, {}", self.sf.total - 4).unwrap();
                writeln!(self.w, "\tadd t0, sp, t0").unwrap();
                writeln!(self.w, "\tlw ra, 0(t0)",).unwrap();
            }
        }
        if self.sf.total > 0 && self.sf.total < 2048 {
            writeln!(self.w, "\taddi sp, sp, {}", self.sf.total).unwrap();
        } else if self.sf.total >= 2048 {
            writeln!(self.w, "\tli t0, -{}", self.sf.total).unwrap();
            writeln!(self.w, "\tadd sp, sp, t0").unwrap();
        }
        writeln!(self.w, "\tret").unwrap();
    }

    fn emit_alloc(&mut self, inst: ir::Value) {
        let alloc_data = self.func.dfg().value(inst);

        if let ValueKind::Alloc(_) = alloc_data.kind() {
            let ty = alloc_data.ty();
            
            if let ir::TypeKind::Pointer(base_ty) = ty.kind() {
                if base_ty.is_i32() {
                    self.sf.alloc_slot(inst);
                } else {
                    let size = calculate_type_size(base_ty); 
                    println!("alloc an array! size = {}", size);
                    self.sf.alloc_array(inst, size);
                }
            }
        } else {
            panic!("emit_alloc called on non-alloc instruction");
        }
    }

    fn emit_load(&mut self, inst: ir::Value, load: &ir::values::Load) {
        let src = load.src();
        let rd = self.ra.alloc();
        
        if self.is_global(src) {
            let var_name = self.get_global_name(src);
            writeln!(self.w, "\tla {}, {}", rd, var_name).unwrap();
            writeln!(self.w, "\tlw {}, 0({})", rd, rd).unwrap();
        } else {
            let src_kind = self.func.dfg().value(src).kind();
            match src_kind {
                ValueKind::GlobalAlloc(_) => {
                    panic!("A global alloc in function?");
                },
                ValueKind::Alloc(_) => {
                    let src_offset = self.sf.get_slot(&src);
                    self.load_from_stack_to(rd, src_offset);
                },
                ValueKind::GetElemPtr(_) | ValueKind::GetPtr(_) => {
                    let ptr_offset = self.sf.get_slot(&src);
                    let ptr_reg = self.ra.alloc();
                    self.load_from_stack_to(ptr_reg, ptr_offset);
                    writeln!(self.w, "\tlw {}, 0({})", rd, ptr_reg).unwrap();
                    self.ra.release(ptr_reg);
                },
                _ => panic!("Unexpected load source: {:?}", src_kind),
            }
        }
        
        let dst_offset = self.sf.alloc_slot(inst);
        self.store_to_stack(rd, dst_offset);
        self.ra.release(rd);
    }

   fn emit_store(&mut self, store: &ir::values::Store) {
        let val = store.value();
        let dst = store.dest();
        let rval = self.reg_for(val);

        if self.is_global(dst) {
            let var_name = self.get_global_name(dst);
            let addr_reg = self.ra.alloc();
            writeln!(self.w, "\tla {}, {}", addr_reg, var_name).unwrap();
            writeln!(self.w, "\tsw {}, 0({})", rval, addr_reg).unwrap();
            self.ra.release(addr_reg);
        } else {
            let dst_kind = self.func.dfg().value(dst).kind();
            match dst_kind {
                ValueKind::GlobalAlloc(_) => {
                    panic!("A global alloc in function?");
                },
                ValueKind::Alloc(_) => {
                    let dst_offset = self.sf.get_slot(&dst);
                    self.store_to_stack(rval, dst_offset);
                },
                ValueKind::GetElemPtr(_) | ValueKind::GetPtr(_) => {
                    let ptr_offset = self.sf.get_slot(&dst);
                    let ptr_reg = self.ra.alloc();
                    self.load_from_stack_to(ptr_reg, ptr_offset);
                    writeln!(self.w, "\tsw {}, 0({})", rval, ptr_reg).unwrap();
                    self.ra.release(ptr_reg);
                },
                _ => panic!("Unexpected store destination: {:?}", dst_kind),
            }
        }
        
        
        self.ra.dec_use(val);
    }

    fn emit_branch(&mut self, branch: &ir::values::Branch) {
        let cond = branch.cond();
        let rcond = self.reg_for(cond);
        let then_name = self.get_bb_name(branch.true_bb());
        let else_name = self.get_bb_name(branch.false_bb());
        
        writeln!(self.w, "\tbnez {}, {}", rcond, then_name).unwrap();
        writeln!(self.w, "\tj {}", else_name).unwrap();
        self.ra.dec_use(cond);
    }

    fn emit_jump(&mut self, jump: &ir::values::Jump) {
        let target_name = self.get_bb_name(jump.target());
        writeln!(self.w, "\tj {}", target_name).unwrap();
    }

    fn emit_call(&mut self, inst: ir::Value, call: &ir::values::Call) {
        let callee_name = self.program
            .func(call.callee())
            .name()
            .strip_prefix('@')
            .unwrap();

        // 传递参数
        for (i, &param) in call.args().iter().enumerate() {
            if i < 8 {
                self.specific_reg_for(param, PARAM_REGS[i]);
            } else {
                let rp = self.reg_for(param);
                let offset = (i - 8) * 4;
                writeln!(self.w, "\tsw {}, {}(sp)", rp, offset).unwrap();
                self.ra.dec_use(param);
            }
        }

        writeln!(self.w, "\tcall {}", callee_name).unwrap();

        // 处理返回值
        if !self.func.dfg().value(inst).ty().is_unit() {
            let offset = self.sf.alloc_slot(inst);
            self.store_to_stack("a0", offset);
        }
    }

    fn is_global(&self, src: ir::Value) -> bool {
        !self.func.dfg().values().contains_key(&src)
    }

    fn emit_get_elem_ptr(&mut self, inst: ir::Value, get_ptr: &ir::values::GetElemPtr) {
        let src = get_ptr.src();
        let index = get_ptr.index();

        let reg_base = self.ra.alloc();
        if !self.is_global(src) {
            let offset = self.sf.get_slot(&src);
            if offset <= 2047 {
                writeln!(self.w, "\taddi {}, sp, {}", reg_base, offset).unwrap();
            } else {
                writeln!(self.w, "\tli {}, {}", reg_base, offset).unwrap();
                writeln!(self.w, "\tadd {}, sp, {}", reg_base, reg_base).unwrap();
            }
        } else {
            let src_data = self.program.borrow_value(src);
            let name = src_data.name()
                .as_ref()
                .unwrap()
                .strip_prefix('@')
                .unwrap();
            writeln!(self.w, "\tla {}, {}", reg_base, name).unwrap();
        }

        let reg_index = self.reg_for(index);
        let reg_size = self.ra.alloc();
        writeln!(self.w, "\tli {}, 4", reg_size).unwrap();
        writeln!(self.w, "\tmul {}, {}, {}", reg_size, reg_index, reg_size).unwrap();
        let reg_offset = reg_size;

        writeln!(self.w, "\tadd {}, {}, {}", reg_base, reg_base, reg_offset).unwrap();
        let reg_result = reg_base;

        let dst_offset = self.sf.alloc_slot(inst);
        self.store_to_stack(reg_result, dst_offset);

        self.ra.release(reg_result);
        self.ra.release(reg_offset);
        self.ra.dec_use(index);
    }

    fn emit_get_ptr(&mut self, inst: ir::Value, get_ptr: &ir::values::GetPtr) {
        let src = get_ptr.src();
        let index = get_ptr.index();

        let src_offset = self.sf.get_slot(&src);
        let reg_base = self.ra.alloc();
        
        self.load_from_stack_to(reg_base, src_offset);
        
        let reg_index = self.reg_for(index);
        let reg_size = self.ra.alloc();
        writeln!(self.w, "\tli {}, 4", reg_size).unwrap();
        writeln!(self.w, "\tmul {}, {}, {}", reg_size, reg_index, reg_size).unwrap();
        let reg_offset = reg_size;

        writeln!(self.w, "\tadd {}, {}, {}", reg_base, reg_base, reg_offset).unwrap();
        let reg_result = reg_base;

        let dst_offset = self.sf.alloc_slot(inst);
        self.store_to_stack(reg_result, dst_offset);

        self.ra.release(reg_result);
        self.ra.release(reg_offset);
        self.ra.dec_use(index);
    }

    fn load_from_stack(&mut self, v: ir::Value, offset: usize) -> &'static str {
        let rd = self.ra.alloc();
        self.load_from_stack_to(rd, offset);
        self.ra.bind(v, rd);
        rd
    }

    // 辅助函数，加载栈上的值到指定寄存器
    fn load_from_stack_to(&mut self, reg: &str, offset: usize) {
        let offset = offset as i32;
        if offset >= -2048 && offset <= 2047 {
            writeln!(self.w, "\tlw {}, {}(sp)", reg, offset).unwrap();
        } else {
            let tmp = self.ra.alloc();
            writeln!(self.w, "\tli {}, {}", tmp, offset).unwrap();
            writeln!(self.w, "\tadd {}, sp, {}", tmp, tmp).unwrap();
            writeln!(self.w, "\tlw {}, 0({})", reg, tmp).unwrap();
            self.ra.release(tmp);
        }
    }

    // 辅助函数，将寄存器的值存储到栈上
    fn store_to_stack(&mut self, reg: &str, offset: usize) {
        let offset = offset as i32;
        if offset >= -2048 && offset <= 2047 {
            writeln!(self.w, "\tsw {}, {}(sp)", reg, offset).unwrap();
        } else {
            let tmp = self.ra.alloc();
            writeln!(self.w, "\tli {}, {}", tmp, offset).unwrap();
            writeln!(self.w, "\tadd {}, sp, {}", tmp, tmp).unwrap();
            writeln!(self.w, "\tsw {}, 0({})", reg, tmp).unwrap();
            self.ra.release(tmp);
        }
    }

    fn reg_for(&mut self, v: ir::Value) -> &'static str {
        if let Some(r) = self.ra.reg_of(v) {
            return r;
        }
        match self.func.dfg().value(v).kind() {
            ValueKind::Integer(int) => {
                if int.value() == 0 {
                    "x0"
                } else {
                    let rd = self.ra.alloc();
                    writeln!(self.w, "\tli {}, {}", rd, int.value()).unwrap();
                    self.ra.bind(v, rd);
                    rd
                }
            },
            ValueKind::Load(_) | ValueKind::Binary(_) | ValueKind::Call(_) => {
                let offset = self.sf.get_slot(&v);
                self.load_from_stack(v, offset)
            },
            ValueKind::Jump(_) => {
                panic!("jump value kind exist")
            },
            ValueKind::Store(_) => {
                panic!("store value kind exist")
            },
            ValueKind::FuncArgRef(arg_ref) => {
                let index = arg_ref.index();
                if index < 8 {
                    let param_reg = PARAM_REGS[index];
                    let rd = self.ra.alloc();
                    writeln!(self.w, "\tmv {}, {}", rd, param_reg).unwrap();
                    self.ra.bind(v, rd);
                    rd
                } else {
                    let offset = (index - 8) * 4 + self.sf.total;
                    self.load_from_stack(v, offset)
                }
            },
            ValueKind::GetElemPtr(_) | ValueKind::GetPtr(_) => {
                let offset = self.sf.get_slot(&v);
                self.load_from_stack(v, offset)
            },
            _ => panic!("Operand not in register and not an immediate"),
        }
    }

    fn specific_reg_for(&mut self, v: ir::Value, reg: &str) {
        match self.func.dfg().value(v).kind() {
            ValueKind::Integer(int) => {
                writeln!(self.w, "\tli {}, {}", reg, int.value()).unwrap();
            },
            ValueKind::Load(_) | ValueKind::Binary(_) | ValueKind::Call(_) => {
                let offset = self.sf.get_slot(&v);
                self.load_from_stack_to(reg, offset);
            },
            ValueKind::Jump(_) => {
                panic!("jump value kind exist")
            },
            ValueKind::Store(_) => {
                panic!("store value kind exist")
            },
            ValueKind::FuncArgRef(arg_ref) => {
                let index = arg_ref.index();
                if index < 8 {
                    let param_reg = PARAM_REGS[index];
                    writeln!(self.w, "\tmv {}, {}", reg, param_reg).unwrap();
                } else {
                    let offset = (index - 8) * 4 + self.sf.total;
                    self.load_from_stack_to(reg, offset);
                }
            },
            ValueKind::GetElemPtr(_) | ValueKind::GetPtr(_) => {
                let offset = self.sf.get_slot(&v);
                self.load_from_stack_to(reg, offset);
            },
            _ => panic!("Operand not in register and not an immediate"),
        }
    }
}