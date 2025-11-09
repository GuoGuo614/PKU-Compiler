use std::{collections::HashMap, io::Write};
use koopa::ir::{self, ValueKind};

static AVAILABLE_REGS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];

pub trait GenerateAsm {
    fn write_assm_to<W: Write>(&self, w: &mut W);
}

impl GenerateAsm for ir::Program {
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        writeln!(w, "\t.text").expect("Write error");
        write!(w, "\t.globl ").expect("Write error");
        for &func in self.func_layout() {
            let func_data = self.func(func);
            write!(w, "{} ", func_data.name().strip_prefix('@').unwrap()).expect("Write error");
        }
        writeln!(w, "").expect("Write error");

        for &func in self.func_layout() {
            let func_data = self.func(func);
            writeln!(w, "{}:", func_data.name().strip_prefix('@').unwrap()).expect("Write error");
            func_data.write_assm_to(w);
        }
    }
}

// 建立 AsmCtx，并派发任务
impl GenerateAsm for ir::FunctionData {
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        let mut ctx = AsmCtx::new(self, w);
        ctx.emit_function();
    }
}

struct RegAlloc {
    free: Vec<&'static str>,
    reg_map: HashMap<ir::Value, &'static str>,
}

struct StackFrame {
    pub total: usize,
    allocs: HashMap<ir::Value, usize>,
}

impl RegAlloc {
    fn new() -> Self {
        let mut free = AVAILABLE_REGS.to_vec();
        free.reverse();
        Self { 
            free, 
            reg_map: HashMap::new(),
        }
    }

    fn alloc(&mut self) -> &'static str {
        self.free.pop().expect("No Free Registers left")
    }

    fn release(&mut self, reg: &'static str) {
        self.free.push(reg);
        // 保证一直按顺序分配，不是那么必要
        // self.free.sort_unstable();
        // self.free.reverse();
    }

    fn bind(&mut self, v: ir::Value, reg: &'static str) {
        self.reg_map.insert(v, reg);
    }

    fn reg_of(&self, v: ir::Value) -> Option<&'static str> {
        self.reg_map.get(&v).copied()
    }

    fn dec_use(&mut self, v: ir::Value) {
        if let Some(reg) = self.reg_map.remove(&v) {
            self.release(reg);
        }
    }
}

impl StackFrame {
    fn new() -> Self {
        Self {
            total: 0,
            allocs: HashMap::new(),
        }
    }

    fn alloc_slot(&mut self, v: ir::Value) -> usize {
        let offset = self.allocs.len() * 4;
        self.allocs.insert(v, offset);
        offset
    }

    fn get_slot(&mut self, v: &ir::Value) -> usize {
        *self.allocs.get(&v).unwrap()
    }
}

// 生成上下文：封装 writer / func / 分配器
struct AsmCtx<'a, W: Write> {
    w: &'a mut W,
    func: &'a ir::FunctionData,
    ra: RegAlloc,
    sf: StackFrame,
}

impl<'a, W: Write> AsmCtx<'a, W> {
    fn new(func: &'a ir::FunctionData, w: &'a mut W) -> Self {
        Self { w, func, ra: RegAlloc::new(), sf: StackFrame::new() }
    }

    fn get_bb_name(&self, bb: ir::BasicBlock) -> &str {
        let name = {
            let bb_data = self.func.dfg().bbs().get(&bb);
            bb_data.unwrap().name()
        };
        name.as_ref().unwrap().strip_prefix('%').unwrap()
    }

    fn emit_function(&mut self) {
        // prologue
        let mut inst_count = 0usize;
        for (&_bb, node) in self.func.layout().bbs() {
            for &inst in node.insts().keys() {
                let v = self.func.dfg().value(inst);
                if !v.ty().is_unit() {
                    inst_count += 1;
                }
            }
        }
        self.sf.total = inst_count * 4;
        writeln!(self.w, "\taddi sp, sp, -{}", inst_count * 4).expect("Write error");
        
        for (&bb, node) in self.func.layout().bbs() {
            let name  = self.get_bb_name(bb).to_string();
            writeln!(self.w, "{}:", name).expect("Write error");
            
            self.emit_block(node);
        }
    }

    fn emit_block(&mut self, bb_node: &ir::layout::BasicBlockNode) {
        for &inst in bb_node.insts().keys() {
            self.emit_inst(inst);
        }
    }

    fn emit_inst(&mut self, inst: ir::Value) {
        let v = self.func.dfg().value(inst);
        match v.kind() {
            ValueKind::Integer(_int) => {
                // 常量一般不会直接作为指令出现
                panic!("Const value appear as inst");
            },
            ValueKind::Binary(bin) => {
                let lhs = bin.lhs();
                let rhs = bin.rhs();
                let rl = self.reg_for(lhs);
                let rr = self.reg_for(rhs);
                let rd = self.ra.alloc();

                self.emit_binary(bin.op(), rd, rl, rr);

                self.ra.dec_use(lhs);
                self.ra.dec_use(rhs);
                
                let offset = self.sf.alloc_slot(inst);
                writeln!(self.w, "\tsw {}, {}(sp)", rd, offset).unwrap();
                
                self.ra.release(rd);
            },
            ValueKind::Return(ret) => {
                let v = ret.value().expect("Return without value");
                let rv = self.reg_for(v);
                writeln!(self.w, "\tmv a0, {}", rv).expect("Write error");
                writeln!(self.w, "\taddi sp, sp, {}", self.sf.total).unwrap();
                writeln!(self.w, "\tret").expect("Write error");
                self.ra.dec_use(v);
            },
            ValueKind::Alloc(_alloc) => {
                self.sf.alloc_slot(inst);
            },
            ValueKind::Load(load) => {
                let src = load.src();
                let src_offset = self.sf.get_slot(&src);
                let rd = self.ra.alloc();
                writeln!(self.w, "\tlw {}, {}(sp)", rd, src_offset).unwrap();

                let dst_offset = self.sf.alloc_slot(inst);
                writeln!(self.w, "\tsw {}, {}(sp)", rd, dst_offset).unwrap();
                self.ra.release(rd);
            },
            ValueKind::Store(store) => {
                let val = store.value();
                let rval = self.reg_for(val);

                let dst = store.dest();
                let dst_offset = self.sf.get_slot(&dst);
                
                writeln!(self.w, "\tsw {}, {}(sp)", rval, dst_offset).unwrap();
                self.ra.dec_use(val);
            },
            ValueKind::Branch(branch) => {
                let cond = branch.cond();
                let rcond = self.reg_for(cond);
                let then_name = self.get_bb_name(branch.true_bb()).to_string();
                writeln!(self.w, "\tbnez {}, {}", rcond, then_name).unwrap();

                let else_name = self.get_bb_name(branch.false_bb()).to_string();
                writeln!(self.w, "\tj {}", else_name).unwrap();
            },
            ValueKind::Jump(jump) => {
                let target = jump.target();
                let target_name = self.get_bb_name(target).to_string();
                writeln!(self.w, "\tj {}", target_name).expect("Write error");
            },
            _ => unreachable!("Unsupported value kind"),
        }
    }

    fn emit_binary(&mut self, op: ir::BinaryOp, rd: &str, rl: &str, rr: &str) {
        match op {
            ir::BinaryOp::Add => writeln!(self.w, "\tadd {}, {}, {}", rd, rl, rr),
            ir::BinaryOp::Sub => writeln!(self.w, "\tsub {}, {}, {}", rd, rl, rr),
            ir::BinaryOp::Mul => writeln!(self.w, "\tmul {}, {}, {}", rd, rl, rr),
            ir::BinaryOp::Div => writeln!(self.w, "\tdiv {}, {}, {}", rd, rl, rr),
            ir::BinaryOp::Mod => writeln!(self.w, "\trem {}, {}, {}", rd, rl, rr),
            ir::BinaryOp::Eq  => {
                writeln!(self.w, "\txor {}, {}, {}", rd, rl, rr).expect("Write error");
                writeln!(self.w, "\tseqz {}, {}", rd, rd)
            },
            ir::BinaryOp::NotEq => {
                writeln!(self.w, "\txor {}, {}, {}", rd, rl, rr).expect("Write error");
                writeln!(self.w, "\tsnez {}, {}", rd, rd)
            }
            ir::BinaryOp::Le => {
                writeln!(self.w, "\tslt {}, {}, {}", rd, rr, rl).expect("Write error");
                writeln!(self.w, "\txori {}, {}, 1", rd, rd)
            },
            ir::BinaryOp::Ge => {
                writeln!(self.w, "\tslt {}, {}, {}", rd, rl, rr).expect("Write error");
                writeln!(self.w, "\txori {}, {}, 1", rd, rd)
            },
            ir::BinaryOp::Lt => {
                writeln!(self.w, "\tslt {}, {}, {}", rd, rl, rr)
            },
            ir::BinaryOp::Gt => {
                writeln!(self.w, "\tslt {}, {}, {}", rd, rr, rl)
            },
            ir::BinaryOp::And => {
                writeln!(self.w, "\tand {}, {}, {}", rd, rl, rr)
            },
            ir::BinaryOp::Or => {
                writeln!(self.w, "\tor {}, {}, {}", rd, rl, rr)
            },
            _ => panic!("Unsupported BinaryOp"),
        }.expect("Write error");
    }

    // 获取操作数寄存器
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
                    writeln!(self.w, "\tli {}, {}", rd, int.value()).expect("Write error");
                    self.ra.bind(v, rd);
                    rd
                }
            },
            ValueKind::Load(_) => {
                // let src = load.src();
                let src_offset = self.sf.get_slot(&v);
                let rd = self.ra.alloc();
                writeln!(self.w, "\tlw {}, {}(sp)", rd, src_offset).expect("Write error");
                self.ra.bind(v, rd);
                rd
            },
            ValueKind::Binary(_) => {
                let offset = self.sf.get_slot(&v);
                let rd = self.ra.alloc();
                writeln!(self.w, "\tlw {}, {}(sp)", rd, offset).expect("Write error");
                self.ra.bind(v, rd);
                rd
            },
            ValueKind::Store(_) => {
                panic!("store value kind exist")
            },
            _ => panic!("Operand not in register and not an immediate"),
        }
    }
}
