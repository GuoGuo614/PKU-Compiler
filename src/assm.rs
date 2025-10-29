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

struct RegAlloc {
    free: Vec<&'static str>,
    reg_map: HashMap<ir::Value, &'static str>,
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

impl GenerateAsm for ir::FunctionData {
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        let mut reg_alloc = RegAlloc::new();
        
        for (&_bb, node) in self.layout().bbs() {
            // 遍历指令列表
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                match value_data.kind() {
                    // 指令通常不存在 Integer 类型
                    ValueKind::Integer(int) => {
                        let reg = reg_alloc.alloc();
                        writeln!(w, "\tli {}, {}", reg, int.value()).expect("Write error");
                        reg_alloc.bind(inst, reg);
                    },
                    ValueKind::Binary(bin) => {
                        let lhs = bin.lhs();
                        let rhs = bin.rhs();
                        let regl = reg_or_materialize(&mut reg_alloc, self, w, lhs);
                        let regr = reg_or_materialize(&mut reg_alloc, self, w, rhs);
                        let rd = reg_alloc.alloc();
                        
                        match bin.op() {
                            ir::BinaryOp::Eq => {
                                writeln!(w, "\txor {}, {}, {}", rd, regl, regr).expect("Write error");
                                writeln!(w, "\tseqz {}, {}", rd, rd).expect("Write error");
                            },
                            ir::BinaryOp::Sub => {
                                writeln!(w, "\tsub {}, {}, {}", rd, regl, regr).expect("Write error");
                            }
                            _ => panic!("Unsupported BinaryOp")
                        }
                        reg_alloc.bind(inst, rd);
                        reg_alloc.dec_use(lhs);
                        reg_alloc.dec_use(rhs);
                    }
                    ValueKind::Return(ret) => {
                        // 处理 ret 指令
                        let val = ret.value().expect("Return without value");
                        let rv = reg_or_materialize(&mut reg_alloc, self, w, val);
                        
                        writeln!(w, "\tmv a0, {}", rv).expect("Write error");
                        writeln!(w, "\tret").expect("Write error");
                        reg_alloc.dec_use(val);
                    }
                    // 其他种类暂时遇不到
                    _ => unreachable!(),
                }
            }
        }
    }
}

fn reg_or_materialize<W: Write>(
    ra: &mut RegAlloc,
    func: &ir::FunctionData,
    w: &mut W,
    v: ir::Value,
) -> &'static str {
    if let Some(r) = ra.reg_of(v) {
        return r;
    }
    match func.dfg().value(v).kind() {
        ValueKind::Integer(int) => {
            if int.value() == 0 {
                "x0"
            } else {
                let rd = ra.alloc();
                writeln!(w, "\tli {}, {}", rd, int.value()).expect("Write error");
                ra.bind(v, rd);
                rd
            }
        }
        _ => panic!("Operand not in register."),
    }
}
