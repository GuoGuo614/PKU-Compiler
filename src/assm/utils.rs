use koopa::ir;

pub fn align_to_16(n: usize) -> usize {
    (n + 15) & !15
}

pub fn func_is_decl(func_data: &ir::FunctionData) -> bool {
    func_data.layout().bbs().is_empty()
}