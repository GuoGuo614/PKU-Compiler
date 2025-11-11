use std::io::Write;
use koopa::ir::ValueKind;
use super::AsmCtx;
use super::ir;

static PARAM_REGS: [&str; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

impl<'a, W: Write> AsmCtx<'a, W> {
    pub fn emit_inst(&mut self, inst: koopa::ir::Value) {
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
            _ => panic!("Unsupported value kind: {:?}", v.kind()),
        }
    }

    fn emit_binary_inst(&mut self, inst: koopa::ir::Value, bin: &koopa::ir::values::Binary) {
        let lhs = bin.lhs();
        let rhs = bin.rhs();
        let rl = self.reg_for(lhs);
        let rr = self.reg_for(rhs);
        let rd = self.ra.alloc();

        super::binary::emit_binary_op(self, bin.op(), rd, rl, rr);

        self.ra.dec_use(lhs);
        self.ra.dec_use(rhs);
        
        let offset = self.sf.alloc_slot(inst);
        writeln!(self.w, "\tsw {}, {}(sp)", rd, offset).unwrap();
        self.ra.release(rd);
    }

    fn emit_return(&mut self, ret: &koopa::ir::values::Return) {
        if let Some(v) = ret.value() {
            self.specific_reg_for(v, "a0");
        }
        if self.has_call {
            writeln!(self.w, "\tlw ra, {}(sp)", self.sf.total - 4).unwrap();
        }
        if self.sf.total > 0 {
            writeln!(self.w, "\taddi sp, sp, {}", self.sf.total).unwrap();
        }
        writeln!(self.w, "\tret").unwrap();
    }

    fn emit_alloc(&mut self, inst: koopa::ir::Value) {
        self.sf.alloc_slot(inst);
    }

    fn emit_load(&mut self, inst: koopa::ir::Value, load: &koopa::ir::values::Load) {
        let src = load.src();
        let rd = self.ra.alloc();
        
        if self.is_global(src) {
            let var_name = self.get_global_name(src);
            writeln!(self.w, "\tla {}, {}", rd, var_name).unwrap();
            writeln!(self.w, "\tlw {}, 0({})", rd, rd).unwrap();
        } else {
            let src_offset = self.sf.get_slot(&src);
            writeln!(self.w, "\tlw {}, {}(sp)", rd, src_offset).unwrap();
        }
        
        let dst_offset = self.sf.alloc_slot(inst);
        writeln!(self.w, "\tsw {}, {}(sp)", rd, dst_offset).unwrap();
        self.ra.release(rd);
    }

    fn emit_store(&mut self, store: &koopa::ir::values::Store) {
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
            let dst_offset = self.sf.get_slot(&dst);
            writeln!(self.w, "\tsw {}, {}(sp)", rval, dst_offset).unwrap();
        }
        
        self.ra.dec_use(val);
    }

    fn emit_branch(&mut self, branch: &koopa::ir::values::Branch) {
        let cond = branch.cond();
        let rcond = self.reg_for(cond);
        let then_name = self.get_bb_name(branch.true_bb());
        let else_name = self.get_bb_name(branch.false_bb());
        
        writeln!(self.w, "\tbnez {}, {}", rcond, then_name).unwrap();
        writeln!(self.w, "\tj {}", else_name).unwrap();
        self.ra.dec_use(cond);
    }

    fn emit_jump(&mut self, jump: &koopa::ir::values::Jump) {
        let target_name = self.get_bb_name(jump.target());
        writeln!(self.w, "\tj {}", target_name).unwrap();
    }

    fn emit_call(&mut self, inst: koopa::ir::Value, call: &koopa::ir::values::Call) {
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
            writeln!(self.w, "\tsw a0, {}(sp)", offset).unwrap();
        }
    }

    fn load_from_stack(&mut self, v: ir::Value, offset: usize) -> &'static str {
        let rd = self.ra.alloc();
        writeln!(self.w, "\tlw {}, {}(sp)", rd, offset).expect("Write error");
        self.ra.bind(v, rd);
        rd
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
            }
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
                writeln!(self.w, "\tlw {}, {}(sp)", reg, offset).expect("Write error");
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
                    writeln!(self.w, "\tlw {}, {}(sp)", reg, offset).unwrap();
                }
            }
            _ => panic!("Operand not in register and not an immediate"),
        }
    }
}