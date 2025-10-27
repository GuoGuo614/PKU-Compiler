use std::io::Write;
use koopa::ir::{self, ValueKind};

pub trait GenerateAsm {
    fn write_assm_to<W: Write>(&self, w: &mut W);
}

impl GenerateAsm for ir::Program {
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        writeln!(w, "\t.text");
        write!(w, "\t.globl ");
        for &func in self.func_layout() {
            let func_data = self.func(func);
            write!(w, "{} ", func_data.name().strip_prefix('@').unwrap());
        }
        writeln!(w, "");

        for &func in self.func_layout() {
            let func_data = self.func(func);
            writeln!(w, "{}:", func_data.name().strip_prefix('@').unwrap());
            func_data.write_assm_to(w);
        }
    }
}

impl GenerateAsm for ir::FunctionData {
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        for (&bb, node) in self.layout().bbs() {
            // let bb_data = self.dfg().bb(bb);
            // let _ = writeln!(w, "{}:", bb_data.name().as_ref().unwrap());

            // 遍历指令列表
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                match value_data.kind() {
                    ValueKind::Integer(int) => {
                        // 处理 integer 指令
                        writeln!(w, "\tli a0, {}", int.value());
                    }
                    ValueKind::Return(ret) => {
                        // 处理 ret 指令
                        let value = ret.value().unwrap();
                        self.dfg().value(value).write_assm_to(w);
                        writeln!(w, "\tret");
                    }
                    // 其他种类暂时遇不到
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl GenerateAsm for ir::entities::ValueData {
    // 姑且为嵌入其他指令的指令实现一下
    fn write_assm_to<W: Write>(&self, w: &mut W) {
        match self.kind() {
            ValueKind::Integer(int) => {
                // 处理 integer 指令
                writeln!(w, "\tli a0, {}", int.value());
            }
            // 其他种类暂时遇不到
            _ => unreachable!(),
        }
    }
}
