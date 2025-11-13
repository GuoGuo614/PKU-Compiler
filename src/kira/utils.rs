// 工具函数

use koopa::*;
use crate::ast::*;
use super::const_eval::ArrayFlatten;

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

/// 处理数组多维度初始化
pub fn process_dimensions<'a, T>(
    sizes: &[usize], 
    init_vals: &'a [T]
) -> Vec<Option<&'a T::Scalar>> 
where T: ArrayFlatten + 'a
{
    let total_size = sizes.iter().fold(1, |acc, &x| acc * x);
    let mut result: Vec<Option<&'a T::Scalar>> = Vec::with_capacity(total_size);
    
    flatten(init_vals, sizes, &mut result);
    result
}

fn flatten<'a, T>(
    init_vals: &'a [T],
    sizes: &[usize],
    result: &mut Vec<Option<&'a T::Scalar>>,
) -> usize
where T: ArrayFlatten 
{
    if sizes.is_empty() {
        return 0;
    }
    // 计算各个维度的步长
    let mut steps = Vec::new();
    let mut s = 1;
    for i in (0..sizes.len()).rev() {
        s *= sizes[i];
        steps.push(s);
    }
    steps.reverse();

    let total_size = sizes.iter().fold(1, |acc, &x| acc * x);
    let mut current_num = 0;
    
    for init_val in init_vals {
        if init_val.is_scalar() {
            result.push(Some(init_val.as_scalar()));
            current_num += 1;
        } else if init_val.is_array() {
            let sub_array = init_val.as_array();
            let offset = result.len();
            println!("offset: {}", offset);
            // 找到对齐的维度
            let mut aligned_dim = None;
            for (dim, &step) in steps.iter().enumerate().skip(1) {
                if offset % step == 0 {
                    aligned_dim = Some(dim);
                    break;
                }
            }
            
            let aligned_dim = aligned_dim.expect(&format!(
                "Initializer list does not align with array dimensions at offset {}",
                offset
            ));
            
            current_num += flatten(sub_array, &sizes[aligned_dim..], result);
        }
    }

    for _ in current_num..(total_size) {
        result.push(None);
    }

    return total_size;
}