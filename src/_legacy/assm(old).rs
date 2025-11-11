use core::panic;
use std::io::Write;
use koopa::ir::{self, ValueKind};
use crate::alloc::*;

static PARAM_REGS: [&str; 8] = ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

pub trait GenerateAsm {
    fn write_assm_to<W: Write>(&self, w: &mut W);
}

impl GenerateAsm for ir::Program {
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        // 解析全局变量
        let var_datas: Vec<_> = self.borrow_values()
            .keys()
            .filter_map(|&val| {
                let val_data = self.borrow_value(val);
                if matches!(val_data.kind(), ValueKind::GlobalAlloc(_)) {
                    Some(val_data)
                } else {
                    None
                }
            }).collect();

        writeln!(w, "\t.data").unwrap();
        for var_data in &var_datas {
            let name = var_data.name().as_ref().unwrap().strip_prefix('@').unwrap();
            writeln!(w, "\t.globl {}", name).unwrap();
            writeln!(w, "{}:", name).unwrap();

            if let ValueKind::GlobalAlloc(alloc) = var_data.kind() {
                let init = alloc.init();
                let init_data = self.borrow_value(init);
                
                match init_data.kind() {
                    ValueKind::Integer(int) => {
                        writeln!(w, "\t.word {}", int.value()).unwrap();
                    },
                    ValueKind::ZeroInit(_) => {
                        writeln!(w, "\t.zero 4").unwrap();
                    },
                    _ => panic!("Unsupported global init type"),
                }
            }
            writeln!(w).unwrap();
        }
        
        // 解析代码段
        let func_names: Vec<_> = self.func_layout()
            .iter()
            .filter_map(|&func| {
                let func_data = self.func(func);
                if !func_is_decl(func_data) {
                    Some(func_data.name().strip_prefix('@').unwrap())
                } else {
                    None
                }
            })
            .collect();
        writeln!(w, "\t.text").unwrap();
        writeln!(w, "\t.globl {}", func_names.join(", ")).unwrap();

        for &func in self.func_layout() {
            let func_data = self.func(func);
            if !func_is_decl(func_data) {
                writeln!(w, "{}:", func_data.name().strip_prefix('@').unwrap()).unwrap();
            }
            let mut ctx = AsmCtx::new(self, func_data, w);
            ctx.emit_function();
        }
    }
}

// 生成上下文：封装 writer / func / 分配器
struct AsmCtx<'a, W: Write> {
    w: &'a mut W,
    program: &'a ir::Program,
    func: &'a ir::FunctionData,
    ra: RegAlloc,
    sf: StackFrame,
    has_call: bool,
}

fn align_to_16(n: usize) -> usize {
    (n + 15) & !15
}

pub fn func_is_decl(func_data: &ir::FunctionData) -> bool {
    func_data.layout().bbs().is_empty()
}

impl<'a, W: Write> AsmCtx<'a, W> {
    fn new(program: &'a ir::Program, func: &'a ir::FunctionData, w: &'a mut W) -> Self {
        Self { 
            w, 
            program, 
            func, 
            ra: RegAlloc::new(), 
            sf: StackFrame::new(),
            has_call: false
        }
    }

    fn get_bb_name(&self, bb: ir::BasicBlock) -> String {
        self.func.dfg()
            .bb(bb)
            .name()
            .as_ref()
            .map(|s| s.trim_start_matches('%').to_string())
            .unwrap()
    }

    fn emit_function(&mut self) {
        // prologue
        let mut inst_count = 0usize;
        let mut has_call = 0;
        let mut max_call_params = 0isize;

        for (&_bb, node) in self.func.layout().bbs() {
            for &inst in node.insts().keys() {
                let v = self.func.dfg().value(inst);
                if !v.ty().is_unit() {
                    inst_count += 1;
                }
                if let ValueKind::Call(call) = v.kind() {
                    has_call = 1;
                    self.has_call = true;
                    max_call_params = 
                        std::cmp::max(max_call_params, call.args().len() as isize - 8);
                }
            }
        }

        self.sf.total = align_to_16((inst_count + has_call + max_call_params as usize) * 4);
        self.sf.params_base = max_call_params as usize * 4;

        if self.sf.total > 0 {
            writeln!(self.w, "\taddi sp, sp, -{}", self.sf.total).unwrap();
        }
        // Save ra and callee_registers
        if self.has_call {
            writeln!(self.w, "\tsw ra, {}(sp)", self.sf.total - 4).unwrap();
        }
        
        for (&bb, node) in self.func.layout().bbs() {
            let name  = self.get_bb_name(bb);
            if name.as_str() != "entry" {
                writeln!(self.w, "{}:", name).unwrap();
            }
            self.emit_block(node);
        }
    }

    fn emit_block(&mut self, bb_node: &ir::layout::BasicBlockNode) {
        for &inst in bb_node.insts().keys() {
            self.emit_inst(inst);
        }
    }

    fn is_global(&self, value: ir::Value) -> bool {
        self.func.dfg().values().get(&value).is_none()
    }
    
    fn get_global_name(&self, value: ir::Value) -> String {
        let data = self.program.borrow_value(value);
        data.name()
            .as_ref()
            .unwrap()
            .strip_prefix('@')
            .unwrap()
            .to_string()
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
                let v = ret.value();
                if v.is_some() {
                    self.specific_reg_for(v.unwrap(), "a0");
                }
                if self.has_call {
                    writeln!(self.w, "\tlw ra, {}(sp)", self.sf.total - 4).unwrap();
                }
                if self.sf.total > 0 {
                    writeln!(self.w, "\taddi sp, sp, {}", self.sf.total).unwrap();
                }
                writeln!(self.w, "\tret").unwrap();
            },
            ValueKind::Alloc(_alloc) => {
                self.sf.alloc_slot(inst);
            },
            ValueKind::Load(load) => {
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
            },
            ValueKind::Store(store) => {
                let val = store.value();
                let rval = self.reg_for(val);
                let dst = store.dest();

                if !self.is_global(dst) {
                    let dst_offset = self.sf.get_slot(&dst);
                    writeln!(self.w, "\tsw {}, {}(sp)", rval, dst_offset).unwrap();
                } else {
                    let addr_reg = self.ra.alloc();
                    let var_name = self.get_global_name(dst);
                    writeln!(self.w, "\tla {}, {}", addr_reg, var_name).unwrap();
                    writeln!(self.w, "\tsw {}, 0({})", rval, addr_reg).unwrap();
                }
                
                self.ra.dec_use(val);
            },
            ValueKind::Branch(branch) => {
                let cond = branch.cond();
                let rcond = self.reg_for(cond);
                let then_name = self.get_bb_name(branch.true_bb());
                writeln!(self.w, "\tbnez {}, {}", rcond, then_name).unwrap();

                let else_name = self.get_bb_name(branch.false_bb());
                writeln!(self.w, "\tj {}", else_name).unwrap();
                self.ra.dec_use(cond);
            },
            ValueKind::Jump(jump) => {
                let target = jump.target();
                let target_name = self.get_bb_name(target);
                writeln!(self.w, "\tj {}", target_name).unwrap();
            },
            ValueKind::Call(call) => {
                let callee = call.callee();
                let callee_name = self
                    .program
                    .func(callee)
                    .name()
                    .strip_prefix("@")
                    .unwrap();

                let params = call.args();
                for (i, &param) in params.iter().enumerate() {
                    if i < 8 {
                        self.specific_reg_for(param, PARAM_REGS[i]);
                    } else {
                        let rp = self.reg_for(param);
                        let offset = (i - 8) * 4;  // 栈上的偏移
                        writeln!(self.w, "\tsw {}, {}(sp)", rp, offset).unwrap();
                        self.ra.dec_use(param);
                    }
                }

                writeln!(self.w, "\tcall {}", callee_name).unwrap();
    
                // 处理一下返回值
                if !self.func.dfg().value(inst).ty().is_unit() {
                    let offset = self.sf.alloc_slot(inst);
                    writeln!(self.w, "\tsw a0, {}(sp)", offset).unwrap();
                }
            }
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
                writeln!(self.w, "\txor {}, {}, {}", rd, rl, rr).unwrap();
                writeln!(self.w, "\tseqz {}, {}", rd, rd)
            },
            ir::BinaryOp::NotEq => {
                writeln!(self.w, "\txor {}, {}, {}", rd, rl, rr).unwrap();
                writeln!(self.w, "\tsnez {}, {}", rd, rd)
            }
            ir::BinaryOp::Le => {
                writeln!(self.w, "\tslt {}, {}, {}", rd, rr, rl).unwrap();
                writeln!(self.w, "\txori {}, {}, 1", rd, rd)
            },
            ir::BinaryOp::Ge => {
                writeln!(self.w, "\tslt {}, {}, {}", rd, rl, rr).unwrap();
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
        }.unwrap();
    }

    fn load_from_stack(&mut self, v: ir::Value, offset: usize) -> &'static str {
        // let offset = self.sf.get_slot(&v);
        let rd = self.ra.alloc();
        writeln!(self.w, "\tlw {}, {}(sp)", rd, offset).expect("Write error");
        self.ra.bind(v, rd);
        rd
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
                if int.value() == 0 {
                    "x0";
                } else {
                    writeln!(self.w, "\tli {}, {}", reg, int.value()).unwrap();
                }
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
