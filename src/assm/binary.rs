use std::io::Write;
use koopa::ir::BinaryOp;
use super::AsmCtx;

pub fn emit_binary_op<W: Write>(ctx: &mut AsmCtx<W>, op: BinaryOp, rd: &str, rl: &str, rr: &str) {
    match op {
        BinaryOp::Add => writeln!(ctx.w, "\tadd {}, {}, {}", rd, rl, rr),
        BinaryOp::Sub => writeln!(ctx.w, "\tsub {}, {}, {}", rd, rl, rr),
        BinaryOp::Mul => writeln!(ctx.w, "\tmul {}, {}, {}", rd, rl, rr),
        BinaryOp::Div => writeln!(ctx.w, "\tdiv {}, {}, {}", rd, rl, rr),
        BinaryOp::Mod => writeln!(ctx.w, "\trem {}, {}, {}", rd, rl, rr),
        BinaryOp::Eq => {
            writeln!(ctx.w, "\txor {}, {}, {}", rd, rl, rr).unwrap();
            writeln!(ctx.w, "\tseqz {}, {}", rd, rd)
        },
        BinaryOp::NotEq => {
            writeln!(ctx.w, "\txor {}, {}, {}", rd, rl, rr).unwrap();
            writeln!(ctx.w, "\tsnez {}, {}", rd, rd)
        },
        BinaryOp::Lt => writeln!(ctx.w, "\tslt {}, {}, {}", rd, rl, rr),
        BinaryOp::Gt => writeln!(ctx.w, "\tslt {}, {}, {}", rd, rr, rl),
        BinaryOp::Le => {
            writeln!(ctx.w, "\tslt {}, {}, {}", rd, rr, rl).unwrap();
            writeln!(ctx.w, "\txori {}, {}, 1", rd, rd)
        },
        BinaryOp::Ge => {
            writeln!(ctx.w, "\tslt {}, {}, {}", rd, rl, rr).unwrap();
            writeln!(ctx.w, "\txori {}, {}, 1", rd, rd)
        },
        BinaryOp::And => writeln!(ctx.w, "\tand {}, {}, {}", rd, rl, rr),
        BinaryOp::Or => writeln!(ctx.w, "\tor {}, {}, {}", rd, rl, rr),
        _ => panic!("Unsupported BinaryOp: {:?}", op),
    }.unwrap();
}