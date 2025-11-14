use std::io::Write;
use koopa::ir::{self, ValueKind};
use super::{AsmCtx, utils::func_is_decl, utils::calculate_type_size};

pub fn emit_data_section<W: Write>(program: &ir::Program, w: &mut W) {
    let global_vars: Vec<_> = program.inst_layout()
        .iter()
        .filter(|&&val| {
            matches!(program.borrow_value(val).kind(), ValueKind::GlobalAlloc(_))
        })
        .collect();

    if global_vars.is_empty() {
        return;
    }

    writeln!(w, "\t.data").unwrap();
    for &val in &global_vars {
        emit_global_var(program, w, val);
    }
    writeln!(w).unwrap();
}

fn emit_global_var<W: Write>(program: &ir::Program, w: &mut W, val: &ir::Value) {
    let val_data = program.borrow_value(val.clone());
    let name = val_data.name().as_ref().unwrap().strip_prefix('@').unwrap();
    
    writeln!(w, "\t.globl {}", name).unwrap();
    writeln!(w, "{}:", name).unwrap();

    if let ValueKind::GlobalAlloc(alloc) = val_data.kind() {
        let init_data = program.borrow_value(alloc.init());
        match init_data.kind() {
            ValueKind::Integer(int) => {
                writeln!(w, "\t.word {}", int.value()).unwrap();
            },
            ValueKind::ZeroInit(_) => {
                let size = calculate_type_size(init_data.ty());
                writeln!(w, "\t.zero {}", size).unwrap();
            },
            ValueKind::Aggregate(aggregate) => {
                let vals = aggregate.elems();
                for val in vals {
                    let val_data = program.borrow_value(*val);
                    if let ValueKind::Integer(int) = val_data.kind() {
                        writeln!(w, "\t.word {}", int.value()).unwrap();
                    }
                }
            }
            _ => panic!("Unsupported global init type"),
        }
    }
}

pub fn emit_text_section<W: Write>(program: &ir::Program, w: &mut W) {
    let func_names: Vec<_> = program.func_layout()
        .iter()
        .filter_map(|&func| {
            let func_data = program.func(func);
            if !func_is_decl(func_data) {
                Some(func_data.name().strip_prefix('@').unwrap())
            } else {
                None
            }
        })
        .collect();

    writeln!(w, "\t.text").unwrap();
    if !func_names.is_empty() {
        writeln!(w, "\t.globl {}", func_names.join(", ")).unwrap();
    }

    for &func in program.func_layout() {
        let func_data = program.func(func);
        if !func_is_decl(func_data) {
            writeln!(w, "{}:", func_data.name().strip_prefix('@').unwrap()).unwrap();
            let mut ctx = AsmCtx::new(program, func_data, w);
            ctx.emit_function();
        }
    }
}
