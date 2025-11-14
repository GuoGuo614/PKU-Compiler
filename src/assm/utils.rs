use koopa::ir;

pub fn align_to_16(n: usize) -> usize {
    (n + 15) & !15
}

pub fn func_is_decl(func_data: &ir::FunctionData) -> bool {
    func_data.layout().bbs().is_empty()
}

pub fn calculate_type_size(ty: &ir::Type) -> usize {
    match ty.kind() {
        ir::TypeKind::Int32 => 4,
        ir::TypeKind::Array(base, len) => {
            let base_size = calculate_type_size(base);
            base_size * len
        }
        ir::TypeKind::Pointer(_) => 4,
        _ => panic!("Unsupported type for size calculation"),
    }
}
