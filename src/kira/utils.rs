// 工具函数

use koopa::*;
use crate::ast::*;

/// 将 AST 的函数类型转换为 Koopa IR 类型
pub fn functype_parse_from_ast(functype: &FuncType) -> ir::Type {
    match functype {
        FuncType::Int => ir::Type::get_i32(),
        FuncType::Void => ir::Type::get_unit(),
    }
}

/// 将 AST 的参数类型转换为 Koopa IR 类型
pub fn params_type_parse(param: &BType) -> ir::Type {
    match param._type.as_str() {
        "int" => ir::Type::get_i32(),
        _  => panic!("Unknown function types.")
    }
}